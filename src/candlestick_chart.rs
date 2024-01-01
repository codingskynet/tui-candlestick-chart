use std::cmp::{max, min};

use itertools::Itertools;
use ordered_float::OrderedFloat;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled},
    widgets::StatefulWidget,
};

use crate::{
    candlestick_chart_state::CandleStikcChartInfo,
    symbols::*,
    x_axis::{Interval, XAxis},
    y_axis::{Numeric, YAxis},
    CandleStickChartState, Float,
};

enum CandleType {
    Bearish,
    Bullish,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    pub timestamp: i64,
    pub open: Float,
    pub high: Float,
    pub low: Float,
    pub close: Float,
}

impl Candle {
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64) -> Option<Self> {
        if high >= low {
            Some(Self {
                timestamp,
                open: OrderedFloat::from(open),
                high: OrderedFloat::from(high),
                low: OrderedFloat::from(low),
                close: OrderedFloat::from(close),
            })
        } else {
            None
        }
    }

    fn render(&self, y_axis: &YAxis) -> (CandleType, Vec<&str>) {
        let open = y_axis.calc_y(self.open);
        let close = y_axis.calc_y(self.close);

        let min = *min(open, close);
        let high = *y_axis.calc_y(self.high);
        let low = *y_axis.calc_y(self.low);
        let max = *max(open, close);

        let mut result = Vec::new();
        for y in (0..y_axis.height()).rev() {
            let y = y as f64;

            let char = if high.ceil() >= y && y >= max.floor() {
                if max - y > 0.75 {
                    UNICODE_BODY
                } else if (max - y) > 0.25 {
                    if (high - y) > 0.75 {
                        UNICODE_TOP
                    } else {
                        UNICODE_HALF_BODY_BOTTOM
                    }
                } else if (high - y) > 0.75 {
                    UNICODE_WICK
                } else if (high - y) > 0.25 {
                    UNICODE_UPPER_WICK
                } else {
                    UNICODE_VOID
                }
            } else if max.floor() >= y && y >= min.ceil() {
                UNICODE_BODY
            } else if min.ceil() >= y && y >= low.floor() {
                if (min - y) < 0.25 {
                    UNICODE_BODY
                } else if (min - y) < 0.75 {
                    if (low - y) < 0.25 {
                        UNICODE_BOTTOM
                    } else {
                        UNICODE_HALF_BODY_TOP
                    }
                } else if low - y < 0.25 {
                    UNICODE_WICK
                } else if low - y < 0.75 {
                    UNICODE_LOWER_WICK
                } else {
                    UNICODE_VOID
                }
            } else {
                UNICODE_VOID
            };

            result.push(char);
        }

        let candle_type = if open <= close {
            CandleType::Bullish
        } else {
            CandleType::Bearish
        };

        (candle_type, result)
    }
}

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
        let chart_width = area.width - y_axis_width;
        let candles_len = chart_width as usize;

        let first_timestamp_cursor = if self.candles.len() > candles_len {
            self.candles[candles_len - 1].timestamp
        } else {
            self.candles.last().unwrap().timestamp
        };

        state.set_info(CandleStikcChartInfo::new(
            first_timestamp_cursor,
            self.candles.last().unwrap().timestamp,
            self.interval,
        ));

        let skipped_candles_len = if let Some(cursor_timestamp) = state.cursor_timestamp {
            let count = self
                .candles
                .iter()
                .filter(|c| c.timestamp <= cursor_timestamp)
                .count();

            if count > candles_len {
                count - candles_len
            } else {
                0
            }
        } else if self.candles.len() > candles_len {
            self.candles.len() - candles_len
        } else {
            0
        };

        let rendered_candles = self
            .candles
            .iter()
            .skip(skipped_candles_len)
            .take(candles_len)
            .collect_vec();

        let min = rendered_candles.iter().map(|c| c.low).min().unwrap();
        let max = rendered_candles.iter().map(|c| c.high).max().unwrap();

        let y_axis = YAxis::new(Numeric::default(), area.height - 3, min, max);
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

        // TODO: if chart_width is negative
        for (x, candle) in rendered_candles.iter().enumerate() {
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
        let buffer = render(widget, 13, 8);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
            "xxxxxxxxxxxxx",
        ]));
    }

    #[test]
    fn simple_candle() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 13, 8);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "    3.000 ├ │",
            "          │ ╽",
            "          │ ┃",
            "          │ ╿",
            "    0.600 ├ │",
            "xxxxxxxxxx└─┴",
            "xxxxxxxxxxxx ",
            "xxxxxxxxxxxxx",
        ]));
    }

    #[test]
    fn simple_candle_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 28, 8);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "    3.000 ├ │xxxxxxxxxxxxxxx",
            "          │ ╽xxxxxxxxxxxxxxx",
            "          │ ┃xxxxxxxxxxxxxxx",
            "          │ ╿xxxxxxxxxxxxxxx",
            "    0.600 ├ │xxxxxxxxxxxxxxx",
            "xxxxxxxxxx└─┴───────────────",
            "xxxxxxxxxxxx1970/01/01 00:00",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ]));
    }

    #[test]
    fn simple_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 3.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
        ]);
        let buffer = render(widget, 17, 8);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "    4.200 ├  ╽╽xx",
            "          │ ╷┃┃xx",
            "          │ ╽ ╹xx",
            "          │ ┃  xx",
            "    0.840 ├ │  xx",
            "xxxxxxxxxx└───┴──",
            "xxxxxxxxxxxx00:02",
            "xxxxxxxxxxxxxxxxx",
        ]));
    }

    #[test]
    fn simple_full_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 3.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
            Candle::new(180000, 2.3, 3.9, 1.3, 2.0).unwrap(),
            Candle::new(240000, 2.0, 5.2, 0.9, 3.9).unwrap(),
        ]);
        let buffer = render(widget, 17, 8);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "    5.200 ├     │",
            "          │  ╽╽╷╽",
            "          │ │┃┃│┃",
            "          │ ┃  ╵│",
            "    1.040 ├ │    ",
            "xxxxxxxxxx└─────┴",
            "xxxxxxxxxxxx00:04",
            "xxxxxxxxxxxxxxxxx",
        ]));
    }

    // TODO: fix if not changing OHLC, it cannot be shown on graph
    // #[test]
    // fn simple_candle_with_not_changing() {
    //     let widget = CandleStickChart::default().candles(vec![
    //         Candle::new(1, 0.0, 100.0, 0.0, 50.0).unwrap(),
    //         Candle::new(1, 50.0, 50.0, 50.0, 50.0).unwrap(),
    //     ]);
    //     let buffer = render(widget, 2, 4);
    //     #[rustfmt::skip]
    //     assert_buffer_eq!(buffer, Buffer::with_lines(vec![
    //         "│ ",
    //         "┃╻",
    //         "┃ ",
    //         "│ ",
    //     ]));
    // }
}
