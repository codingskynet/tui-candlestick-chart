use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use ratatui_candlestick_chart::Candle;
use ratatui_candlestick_chart::CandleStickChart;

struct App {
    candles: Vec<Candle>,
}

impl App {
    fn new() -> Self {
        Self {
            candles: vec![
                Candle::new(1703656020000, 42366.00, 42391.10, 42366.00, 42391.10).unwrap(),
                Candle::new(1703656080000, 42391.10, 42420.00, 42391.09, 42419.99).unwrap(),
                Candle::new(1703656140000, 42420.00, 42429.02, 42414.12, 42429.02).unwrap(),
                Candle::new(1703656200000, 42429.01, 42441.49, 42424.52, 42426.01).unwrap(),
                Candle::new(1703656260000, 42426.01, 42426.01, 42414.36, 42417.98).unwrap(),
                Candle::new(1703656320000, 42417.99, 42441.10, 42415.00, 42441.10).unwrap(),
                Candle::new(1703656380000, 42441.09, 42448.07, 42440.00, 42441.24).unwrap(),
                Candle::new(1703656440000, 42441.24, 42448.07, 42441.23, 42446.62).unwrap(),
                Candle::new(1703656500000, 42446.61, 42449.99, 42432.00, 42432.00).unwrap(),
                Candle::new(1703656560000, 42432.00, 42432.01, 42411.10, 42413.33).unwrap(),
                Candle::new(1703656620000, 42413.33, 42441.67, 42406.01, 42436.01).unwrap(),
                Candle::new(1703656680000, 42436.01, 42436.01, 42425.58, 42427.64).unwrap(),
                Candle::new(1703656740000, 42427.64, 42458.24, 42427.63, 42454.27).unwrap(),
                Candle::new(1703656800000, 42454.28, 42461.65, 42453.04, 42458.83).unwrap(),
                Candle::new(1703656860000, 42458.83, 42470.01, 42458.83, 42470.01).unwrap(),
                Candle::new(1703656920000, 42470.01, 42485.00, 42470.00, 42474.71).unwrap(),
            ],
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chart = CandleStickChart::default().candles(app.candles.clone());
    f.render_widget(chart, f.size());
}
