use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::BTreeMap,
    error::Error,
    io,
    rc::Rc,
    time::{Duration, Instant},
};

use actix_rt::time::sleep;
use awc::{ws, Client};
use chrono::{Offset, TimeZone, Utc};
use chrono_tz::Asia;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{prelude::stream::StreamExt, SinkExt};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use ratatui::prelude::*;
use tui_candlestick_chart::{
    Candle, CandleStickChart, CandleStickChartState, Grid, Interval, Numeric,
};

const SYMBOL: &str = "BTCUSDT";

struct App {
    is_loading_previous_candles: Rc<RefCell<bool>>,
    candles: Rc<RefCell<BTreeMap<i64, Candle>>>,
    state: CandleStickChartState,
}

impl App {
    fn new() -> Self {
        Self {
            is_loading_previous_candles: Rc::new(RefCell::new(false)),
            candles: Rc::new(RefCell::new(BTreeMap::new())),
            state: CandleStickChartState::default(),
        }
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();

    actix_rt::spawn(binance_perp_agg_trade(app.candles.clone()));

    let tick_rate = Duration::from_millis(200);
    let res = run_app(&mut terminal, app, tick_rate).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if !*app.is_loading_previous_candles.borrow() {
            let first_timestamp = app.candles.borrow().keys().next().cloned();
            if app.state.is_needed_previous_candles() {
                if let Some(first_timestamp) = first_timestamp {
                    *app.is_loading_previous_candles.borrow_mut() = true;
                    actix_rt::spawn(binance_perp_klines(
                        app.is_loading_previous_candles.clone(),
                        first_timestamp,
                        app.candles.clone(),
                    ));
                }
            }
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.state.try_move_backward(),
                    KeyCode::Right => app.state.try_move_forward(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
        sleep(Duration::from_millis(10)).await;
    }
}

async fn binance_perp_agg_trade(candles: Rc<RefCell<BTreeMap<i64, Candle>>>) {
    let client = awc::Client::builder()
        .max_http_version(awc::http::Version::HTTP_11)
        .finish();

    let (_, mut connection) = client
        .ws(format!(
            "wss://fstream.binance.com/ws/{}@aggTrade",
            SYMBOL.to_lowercase()
        ))
        .connect()
        .await
        .unwrap();

    loop {
        let response = connection.next().await.unwrap().unwrap();
        let ws::Frame::Text(bytes) = response else {
            if let ws::Frame::Ping(_) = response {
                let _ = connection
                    .send(ws::Message::Pong(([0x0A].as_slice()).into()))
                    .await
                    .unwrap();
            }
            continue;
        };
        let json: serde_json::Value =
            serde_json::from_str(&std::str::from_utf8(&bytes.to_vec()).unwrap()).unwrap();

        let t = json["T"].as_i64().unwrap() / 60_000 * 60_000;
        let p = OrderedFloat::from(json["p"].as_str().unwrap().parse::<f64>().unwrap());
        candles
            .borrow_mut()
            .entry(t)
            .and_modify(|c| {
                c.low = min(c.low, p);
                c.high = max(c.high, p);
                c.close = p;
            })
            .or_insert(Candle::new(t, *p, *p, *p, *p).unwrap());
    }
}

async fn binance_perp_klines(
    is_loading_previous_candles: Rc<RefCell<bool>>,
    first_timestamp: i64,
    candles: Rc<RefCell<BTreeMap<i64, Candle>>>,
) {
    let bytes = Client::new()
        .get(format!(
            "https://fapi.binance.com/fapi/v1/klines?symbol={}&interval=1m&endTime={}",
            SYMBOL, first_timestamp
        ))
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&std::str::from_utf8(&bytes.to_vec()).unwrap()).unwrap();

    let mut candles = candles.borrow_mut();
    for kline in json.as_array().unwrap() {
        let data = kline.as_array().unwrap();
        let timestamp = data[0].as_i64().unwrap();
        let candle = Candle::new(
            timestamp,
            data[1].as_str().unwrap().parse::<f64>().unwrap(),
            data[2].as_str().unwrap().parse::<f64>().unwrap(),
            data[3].as_str().unwrap().parse::<f64>().unwrap(),
            data[4].as_str().unwrap().parse::<f64>().unwrap(),
        )
        .unwrap();
        candles.insert(timestamp, candle);
    }
    *is_loading_previous_candles.borrow_mut() = false;
}

fn ui(f: &mut Frame, app: &mut App) {
    let chart = CandleStickChart::new(Interval::OneMinute)
        .candles(
            app.candles
                .borrow()
                .values()
                .cloned()
                .sorted_by_key(|c| c.timestamp)
                .collect_vec(),
        )
        .y_grid(Grid::Readable)
        .display_timezone(
            Asia::Seoul
                .offset_from_utc_date(&Utc::now().naive_utc().date())
                .fix(),
        );
    f.render_stateful_widget(chart, f.size(), &mut app.state);
}
