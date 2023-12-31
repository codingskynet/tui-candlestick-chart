use std::cmp::max;

use ordered_float::OrderedFloat;

use crate::Float;

pub(crate) struct Numeric {
    precision: usize,
    scale: usize,
}

impl Default for Numeric {
    fn default() -> Self {
        Self::new(8, 3)
    }
}

impl Numeric {
    pub fn new(precision: usize, scale: usize) -> Self {
        Self { precision, scale }
    }

    pub fn format(&self, value: Float) -> String {
        let precision = self.precision;
        let scale = self.scale;
        format!("{0:>precision$.scale$}", value)
    }
}

pub(crate) struct YAxis {
    numeric: Numeric,
    height: u16,
    min: Float,
    max: Float,
    unit: Float,
}

impl YAxis {
    pub fn new(numeric: Numeric, height: u16, min: Float, max: Float) -> Self {
        assert!(min < max);
        let unit = (max - min) / OrderedFloat::from(height as f64);

        Self {
            numeric,
            height,
            min,
            max,
            unit,
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn calc_y(&self, value: Float) -> Float {
        (value - self.min) / self.unit
    }

    pub fn render(&self) -> Vec<String> {
        let mut result = Vec::new();
        let max_chars = max(
            self.numeric.format(self.max).len(),
            self.numeric.format(self.min).len(),
        );
        for i in 0..self.height {
            let rendered = if i % 4 == 0 {
                let value = self.max - self.unit * OrderedFloat::from(i);
                format!(" {} │┈ ", self.numeric.format(value))
            } else {
                format!(" {} │  ", " ".repeat(max_chars))
            };

            result.push(rendered);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use crate::{
        y_axis::{Numeric, YAxis},
        Float,
    };

    #[test]
    fn test_format() {
        let numeric = Numeric::new(10, 2);
        assert_eq!(numeric.format(Float::from(3.1415926535)), "      3.14");
        assert_eq!(numeric.format(Float::from(99991)), "  99991.00");
    }

    #[test]
    fn test_calc() {
        let y_axis = YAxis::new(Numeric::default(), 40, 100.into(), 200.into());
        assert_eq!(y_axis.calc_y(130.into()), OrderedFloat::from(28));
    }
}
