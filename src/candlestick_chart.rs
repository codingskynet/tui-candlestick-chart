use std::cmp::{max, min};

use ordered_float::OrderedFloat;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Styled},
    widgets::Widget,
};

use crate::{symbols::*, utils::YAxis, Float};

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

    fn render(&self, y_axis: &YAxis) -> Vec<&str> {
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

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleStickChart {
    /// Widget style
    style: Style,
    /// Candle data
    candles: Vec<Candle>,
}

impl Default for CandleStickChart {
    fn default() -> Self {
        Self {
            style: Style::default(),
            candles: Vec::new(),
        }
    }
}

impl CandleStickChart {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn candles(mut self, candles: Vec<Candle>) -> Self {
        self.candles = candles;
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

impl Widget for CandleStickChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.candles.is_empty() {
            return;
        }

        let max = self.candles.iter().map(|c| c.high).max().unwrap();
        let min = self.candles.iter().map(|c| c.low).min().unwrap();

        let y_axis = YAxis::new(area.height, min, max);

        for (x, candle) in self.candles.iter().take(area.width as usize).enumerate() {
            let rendered = candle.render(&y_axis);

            for (y, char) in rendered.iter().enumerate() {
                buf.get_mut(x as u16, y as u16).set_symbol(char);
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
        widgets::Widget,
    };

    use crate::{Candle, CandleStickChart};

    fn render(widget: CandleStickChart, height: u16) -> Buffer {
        let area = Rect::new(0, 0, 1, height);
        let mut cell = Cell::default();
        cell.set_symbol("x");
        let mut buffer = Buffer::filled(area, &cell);
        widget.render(area, &mut buffer);
        buffer
    }

    #[test]
    fn simple_candle() {
        let widget =
            CandleStickChart::default().candles(vec![Candle::new(1, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 4);
        #[rustfmt::skip]
        assert_buffer_eq!(buffer, Buffer::with_lines(vec![
            "│",
            "┃",
            "┃",
            "│"
        ]));
    }
}
