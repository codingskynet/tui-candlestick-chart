use ordered_float::OrderedFloat;

use crate::Float;

pub(crate) struct YAxis {
    height: u16,
    min: Float,
    max: Float,
    unit: Float,
}

impl YAxis {
    pub fn new(height: u16, min: Float, max: Float) -> Self {
        let unit = (max - min) / OrderedFloat::from(height as f64);

        Self {
            height,
            min,
            max,
            unit,
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn unit(&self) -> Float {
        self.unit
    }

    pub fn calc_y(&self, value: Float) -> Float {
        (value - self.min) / self.unit
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use crate::utils::YAxis;

    #[test]
    fn test_calc() {
        let y_axis = YAxis::new(40, 100.into(), 200.into());
        assert_eq!(y_axis.calc_y(130.into()), OrderedFloat::from(28));
    }
}
