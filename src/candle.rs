use std::cmp::{max, min};

use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::{symbols::*, y_axis::YAxis, Float};

pub(crate) enum CandleType {
    Bearish,
    Bullish,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    pub(crate) timestamp: i64,
    pub(crate) open: Float,
    pub(crate) high: Float,
    pub(crate) low: Float,
    pub(crate) close: Float,
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

    pub(crate) fn render(&self, y_axis: &YAxis) -> (CandleType, Vec<&str>) {
        let open = y_axis.calc_y(self.open);
        let close = y_axis.calc_y(self.close);

        let min = *min(open, close);
        let high = *y_axis.calc_y(self.high);
        let low = *y_axis.calc_y(self.low);
        let max = *max(open, close);

        let mut is_body = false;
        let mut result = Vec::new();
        for y in (0..y_axis.height()).rev() {
            let y = y as f64;

            let high_max_diff = high - max;
            let min_low_diff = min - low;
            debug_assert!(high_max_diff >= 0.);
            debug_assert!(min_low_diff >= 0.);

            let char = if high.ceil() >= y && y >= max.floor() {
                if high - y > 0.5 {
                    if high_max_diff < 0.25 {
                        is_body = true;
                        UNICODE_BODY
                    } else if high_max_diff < 0.75 {
                        if is_body {
                            is_body = true;
                            UNICODE_BODY
                        } else {
                            is_body = true;
                            UNICODE_UP
                        }
                    } else {
                        UNICODE_WICK
                    }
                } else if high - y >= 0. {
                    if high_max_diff < 0.25 {
                        UNICODE_HALF_BODY_BOTTOM
                    } else {
                        UNICODE_HALF_WICK_BOTTOM
                    }
                } else {
                    UNICODE_VOID
                }
            } else if max.floor() >= y && y >= min.ceil() {
                is_body = true;
                UNICODE_BODY
            } else if min.ceil() >= y && y >= low.floor() {
                if low - y < 0.5 {
                    if min_low_diff < 0.25 {
                        is_body = true;
                        UNICODE_BODY
                    } else if min_low_diff < 0.75 {
                        if is_body {
                            is_body = false;
                            UNICODE_DOWN
                        } else {
                            UNICODE_WICK
                        }
                    } else {
                        UNICODE_WICK
                    }
                } else if low - y <= 1.0 {
                    if min_low_diff < 0.25 {
                        UNICODE_HALF_BODY_TOP
                    } else {
                        UNICODE_HALF_WICK_TOP
                    }
                } else {
                    UNICODE_VOID
                }
            } else {
                UNICODE_VOID
            };

            result.push(char);
        }

        #[cfg(debug_assertions)]
        if !test_continuous_graph(result.clone()) {
            tracing::error!("The result of candle rendering is broken. Please report it.")
        }

        let candle_type = if open <= close {
            CandleType::Bullish
        } else {
            CandleType::Bearish
        };

        (candle_type, result)
    }
}

fn test_continuous_graph(mut chars: Vec<&str>) -> bool {
    if chars.iter().all(|&c| c == UNICODE_VOID) {
        return false;
    }

    if chars.len() <= 1 {
        return true;
    }

    chars.push(UNICODE_VOID);

    // check if there is VOID between chars
    {
        let mut graphs = 0;
        for (a, b) in chars.clone().into_iter().tuple_windows() {
            if a != UNICODE_VOID && b == UNICODE_VOID {
                graphs += 1;
            }
        }

        if graphs > 1 {
            return false;
        }
    }

    for (a, b) in chars.clone().into_iter().tuple_windows() {
        match (a, b) {
            (UNICODE_VOID, UNICODE_VOID) => {}
            (UNICODE_VOID, _) => {}
            (_, UNICODE_VOID) => {}
            (UNICODE_BODY, UNICODE_UP | UNICODE_HALF_BODY_BOTTOM | UNICODE_HALF_WICK_BOTTOM) => {
                return false
            }
            (UNICODE_DOWN | UNICODE_HALF_BODY_TOP | UNICODE_HALF_WICK_TOP, UNICODE_BODY) => {
                return false
            }
            (UNICODE_WICK, UNICODE_HALF_BODY_BOTTOM | UNICODE_HALF_WICK_BOTTOM) => return false,
            (UNICODE_HALF_BODY_TOP | UNICODE_HALF_WICK_TOP, UNICODE_WICK) => return false,
            _ => {}
        }
    }

    true
}
