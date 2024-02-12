use itertools::Itertools;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled},
    widgets::StatefulWidget,
};

use crate::{
    candle::{Candle, CandleType},
    candlestick_chart_state::CandleStikcChartInfo,
    x_axis::{Interval, XAxis},
    y_axis::{Numeric, YAxis},
    CandleStickChartState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleStickChart {
    /// Candle interval
    interval: Interval,
    /// Candle data
    candles: Vec<Candle>,
    /// Widget style
    style: Style,
    /// Candle style,
    bearish_color: Color,
    bullish_color: Color,
}

impl CandleStickChart {
    pub fn new(interval: Interval) -> Self {
        Self {
            interval,
            candles: Vec::default(),
            style: Style::default(),
            bearish_color: Color::Rgb(234, 74, 90),
            bullish_color: Color::Rgb(52, 208, 88),
        }
    }

    pub fn candles(mut self, candles: Vec<Candle>) -> Self {
        self.candles = candles;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn bearish_color(mut self, color: Color) -> Self {
        self.bearish_color = color;
        self
    }

    pub fn bullish_color(mut self, color: Color) -> Self {
        self.bullish_color = color;
        self
    }
}

impl Styled for CandleStickChart {
    type Item = CandleStickChart;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

impl StatefulWidget for CandleStickChart {
    type State = CandleStickChartState;

    /// render like:
    /// |---|-----------------------|
    /// | y |                       |
    /// |   |                       |
    /// | a |                       |
    /// | x |                       |
    /// | i |                       |
    /// | s |       chart data      |
    /// |   |                       |
    /// | a |                       |
    /// | r |                       |
    /// | e |                       |
    /// | a |                       |
    /// |---|-----------------------|
    ///     |      x axis area      |
    ///     |-----------------------|
    ///
    ///
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if self.candles.is_empty() {
            return;
        }

        let global_min = self.candles.iter().map(|c| c.low).min().unwrap();
        let global_max = self.candles.iter().map(|c| c.high).max().unwrap();

        let y_axis_width: u16 = YAxis::estimated_width(Numeric::default(), global_min, global_max);
        if area.width <= y_axis_width || area.height <= 3 {
            return;
        }

        let chart_width = area.width - y_axis_width;
        let chart_width_usize = chart_width as usize;

        // with first/last dummies
        let first_timestamp = self.candles.first().unwrap().timestamp;
        let last_timestamp = self.candles.last().unwrap().timestamp;
        let y_min = self.candles.iter().map(|c| c.low).min().unwrap();
        let y_max = self.candles.iter().map(|c| c.high).max().unwrap();

        let mut candles = Vec::new();
        for i in (1..=(chart_width as i64 - 1)).rev() {
            candles.push(
                Candle::new(
                    first_timestamp - i * self.interval as i64 * 1000,
                    0.,
                    0.,
                    0.,
                    0.,
                )
                .unwrap(),
            );
        }
        candles.extend(self.candles.clone());
        for i in 1..=(chart_width as i64 - 1) {
            candles.push(
                Candle::new(
                    last_timestamp + i * self.interval as i64 * 1000,
                    0.,
                    0.,
                    0.,
                    0.,
                )
                .unwrap(),
            );
        }

        let first_timestamp_cursor = candles[chart_width_usize - 1].timestamp;

        state.set_info(CandleStikcChartInfo::new(
            first_timestamp_cursor,
            candles.last().unwrap().timestamp,
            self.interval,
            last_timestamp,
        ));

        let skipped_candles_len = {
            let count = candles
                .iter()
                .filter(|c| c.timestamp <= state.cursor_timestamp.unwrap_or(last_timestamp))
                .count();

            if count >= chart_width_usize {
                count - chart_width_usize
            } else {
                0
            }
        };

        let rendered_candles = candles
            .iter()
            .skip(skipped_candles_len)
            .take(chart_width_usize)
            .collect_vec();

        let y_axis = YAxis::new(Numeric::default(), area.height - 3, y_min, y_max);
        let rendered_y_axis = y_axis.render();
        for (y, string) in rendered_y_axis.iter().enumerate() {
            buf.set_string(0, y as u16, string, Style::default());
        }

        let timestamp_min = rendered_candles.first().unwrap().timestamp;
        let timestamp_max = rendered_candles.last().unwrap().timestamp;

        let x_axis = XAxis::new(chart_width, timestamp_min, timestamp_max, self.interval);
        let rendered_x_axis = x_axis.render();
        buf.set_string(y_axis_width - 2, area.height - 3, "└──", Style::default());
        for (y, string) in rendered_x_axis.iter().enumerate() {
            buf.set_string(
                y_axis_width,
                area.height - 3 + y as u16,
                string,
                Style::default(),
            );
        }

        for (x, candle) in rendered_candles.iter().enumerate() {
            if candle.timestamp < first_timestamp || candle.timestamp > last_timestamp {
                continue;
            }
            let (candle_type, rendered) = candle.render(&y_axis);

            let color = match candle_type {
                CandleType::Bearish => self.bearish_color,
                CandleType::Bullish => self.bullish_color,
            };

            for (y, char) in rendered.iter().enumerate() {
                buf.get_mut(x as u16 + y_axis_width, y as u16)
                    .set_symbol(char)
                    .set_style(Style::default().fg(color));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        assert_buffer_eq,
        buffer::{Buffer, Cell},
        layout::Rect,
        style::{Style, Stylize},
        widgets::StatefulWidget,
    };

    use crate::{Candle, CandleStickChart, CandleStickChartState, Interval};

    fn render(widget: CandleStickChart, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut cell = Cell::default();
        cell.set_symbol("x");
        let mut buffer = Buffer::filled(area, &cell);
        widget.render(area, &mut buffer, &mut CandleStickChartState::default());
        buffer.set_style(area, Style::default().reset());
        buffer
    }

    #[test]
    fn empty_candle() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![]);
        let buffer = render(widget, 14, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 14, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     3.000 ├ │",
                "           │ │",
                "           │ ┃",
                "           │ │",
                "     0.600 ├ │",
                "xxxxxxxxxxx└──",
                "xxxxxxxxxxxxx ",
                "xxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 29, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     3.000 ├ xxxxxxxxxxxxxxx│",
                "           │ xxxxxxxxxxxxxxx│",
                "           │ xxxxxxxxxxxxxxx┃",
                "           │ xxxxxxxxxxxxxxx│",
                "     0.600 ├ xxxxxxxxxxxxxxx│",
                "xxxxxxxxxxx└────────────────┴",
                "xxxxxxxxxxxxx1970/01/01 00:00",
                "xxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 2.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
        ]);
        let buffer = render(widget, 18, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     4.200 ├ xx ╽┃",
                "           │ xx│┃┃",
                "           │ xx│╹╿",
                "           │ xx│  ",
                "     0.840 ├ xx│  ",
                "xxxxxxxxxxx└─────┴",
                "xxxxxxxxxxxxx00:02",
                "xxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_full_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 2.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
            Candle::new(180000, 2.3, 3.9, 1.3, 2.0).unwrap(),
            Candle::new(240000, 2.0, 5.2, 0.9, 3.9).unwrap(),
        ]);
        let buffer = render(widget, 18, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     5.200 ├  ╷  │",
                "           │  ╽┃││",
                "           │ │┃╿│┃",
                "           │ ┃ ╵││",
                "     1.040 ├ │   ╵",
                "xxxxxxxxxxx└─────┴",
                "xxxxxxxxxxxxx00:04",
                "xxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_not_changing() {
        let widget = CandleStickChart::new(Interval::OneSecond).candles(vec![
            Candle::new(0, 0.0, 1000.0, 0.0, 50.0).unwrap(),
            Candle::new(1, 50.0, 50.0, 50.0, 50.0).unwrap(),
            Candle::new(2, 500.0, 500.0, 500.0, 500.0).unwrap(),
        ]);
        let buffer = render(widget, 16, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "  1000.000 ├ │  ",
                "           │ │  ",
                "           │ │ ╻",
                "           │ │  ",
                "   200.000 ├ │╻ ",
                "xxxxxxxxxxx└────",
                "xxxxxxxxxxxxx   ",
                "xxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_small_candle() {
        let widget = CandleStickChart::new(Interval::OneSecond).candles(vec![
            Candle::new(0, 0.0, 1000.0, 0.0, 50.0).unwrap(),
            Candle::new(1, 450.0, 580.0, 320.0, 450.0).unwrap(),
            Candle::new(1, 580.0, 580.0, 320.0, 320.0).unwrap(),
        ]);
        let buffer = render(widget, 16, 8);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "  1000.000 ├ │  ",
                "           │ │  ",
                "           │ │╽┃",
                "           │ │╵╹",
                "   200.000 ├ │  ",
                "xxxxxxxxxxx└────",
                "xxxxxxxxxxxxx   ",
                "xxxxxxxxxxxxxxxx",
            ])
        );
    }
}
