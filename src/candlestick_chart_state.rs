use std::cmp::{max, min};

use crate::Interval;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandleStikcChartInfo {
    first_timestamp: i64,
    last_timestamp: i64,
    interval: Interval,
}

impl CandleStikcChartInfo {
    pub(crate) fn new(first_timestamp: i64, last_timestamp: i64, interval: Interval) -> Self {
        Self {
            first_timestamp,
            last_timestamp,
            interval,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CandleStickChartState {
    pub(crate) info: Option<CandleStikcChartInfo>,

    pub(crate) cursor_timestamp: Option<i64>,
}

impl CandleStickChartState {
    pub(crate) fn set_info(&mut self, info: CandleStikcChartInfo) {
        if let Some(cursor_timestamp) = self.cursor_timestamp {
            if cursor_timestamp == info.last_timestamp {
                self.cursor_timestamp = None;
            } else {
                self.cursor_timestamp =
                    Some(cursor_timestamp.clamp(info.first_timestamp, info.last_timestamp));
            }
        }
        self.info = Some(info);
    }

    pub fn try_move_backward(&mut self) {
        if let Some(info) = &self.info {
            let cursor = if let Some(cursor_timestamp) = self.cursor_timestamp {
                cursor_timestamp - info.interval as i64 * 1000
            } else {
                info.last_timestamp - info.interval as i64 * 1000
            };

            self.cursor_timestamp = Some(max(cursor, info.first_timestamp));
        }
    }

    pub fn try_move_forward(&mut self) {
        if let Some(info) = &self.info {
            if let Some(cursor_timestamp) = self.cursor_timestamp {
                self.cursor_timestamp = Some(min(
                    cursor_timestamp + info.interval as i64 * 1000,
                    info.last_timestamp,
                ));
            }
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_timestamp = None;
    }
}
