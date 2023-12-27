use ordered_float::OrderedFloat;

mod candlestick_chart;
mod symbols;
mod utils;

pub use candlestick_chart::Candle;
pub use candlestick_chart::CandleStickChart;

pub(crate) type Float = OrderedFloat<f64>;
