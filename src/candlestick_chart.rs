use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Styled},
    widgets::Widget,
};

#[derive(Debug, Clone)]
pub struct CandleStickChart {
    /// Widget style
    style: Style,
}

impl Styled for CandleStickChart {
    type Item = CandleStickChart;

    fn style(&self) -> Style {
        todo!()
    }

    fn set_style(self, style: Style) -> Self::Item {
        todo!()
    }
}

impl Widget for CandleStickChart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        todo!()
    }
}

#[cfg(tests)]
mod tests {}
