use palette::{Lab, Lch};
use std::collections::HashMap;
use stats::{stddev, mean};
use std::f32::MAX;
use std::f32::MIN;
use genetic::ProblemDescription;
use colorscheme::ColorScheme;

#[derive(Clone)]
pub struct ColorSchemeProblemDescription {
    pub free_color_count: usize,
    pub fixed_colors: Vec<Lab>,
    pub fitness_targets: Vec<Target>,
}

impl ProblemDescription<ColorScheme, ColorSchemeProblemDescription> for ColorSchemeProblemDescription {
    fn calculate_fitness(&self, scheme: &ColorScheme) -> f32 {
        let data = scheme.fitness_data(&self);
        self.fitness_targets.iter().map(|target| target.calculate(&data)).sum::<f32>()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Parameter {
    Chroma,
    Luminance,
    FixedDistance,
    FreeDistance,
}

pub type FitnessData = HashMap<Parameter, StatValues>;


#[derive(Clone)]
pub struct Target {
    direction: TargetDirection,
    stat: Stat,
    parameter: Parameter,
    strength: Strength,
}

#[derive(Clone)]
pub enum TargetDirection {
    Maximize,
    Minimize,
    MaximizeDifference(f32),
    MinimizeDifference(f32),
}

impl Target {
    pub fn new(direction: TargetDirection,
               stat: Stat,
               parameter: Parameter,
               strength: Strength)
               -> Target {
        Target {
            direction: direction,
            stat: stat,
            parameter: parameter,
            strength: strength,
        }
    }
}

impl Target {
    fn calculate(&self, data: &FitnessData) -> f32 {
        let value = data.get(&self.parameter).unwrap().get(&self.stat);
        match self.direction {
            TargetDirection::Maximize => self.strength.calculate(value),
            TargetDirection::Minimize => -self.strength.calculate(value),
            TargetDirection::MaximizeDifference(against) => {
                self.strength.calculate((against - value).abs())
            }
            TargetDirection::MinimizeDifference(against) => {
                -self.strength.calculate((against - value).abs())
            }
        }

    }
}

#[derive(Clone)]
pub struct Strength {
    pub factor: f32,
    pub exponent: i32,
}

impl Strength {
    fn calculate(&self, value: f32) -> f32 {
        (self.factor * value).powi(self.exponent)
    }
}

#[derive(Clone)]
pub enum Stat {
    Mean,
    StdDev,
    Min,
    Max,
}

pub struct StatValues {
    mean: f32,
    stddev: f32,
    min: f32,
    max: f32,
}

impl StatValues {
    pub fn from(data: &Vec<f32>) -> StatValues {
        StatValues {
            mean: mean(data.iter().map(|&x| x)) as f32,
            stddev: stddev(data.iter().map(|&x| x)) as f32,
            min: data.iter().fold(MAX, |min, &x| min.min(x)),
            max: data.iter().fold(MIN, |max, &x| max.max(x)),
        }
    }

    fn get(&self, prop: &Stat) -> f32 {
        match prop {
            &Stat::Mean => self.mean,
            &Stat::StdDev => self.stddev,
            &Stat::Min => self.min,
            &Stat::Max => self.max,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Stat::*;
    use super::Parameter::*;
    use super::TargetDirection::*;


    #[test]
    fn target_calculate() {
        let mut data = FitnessData::new();
        data.insert(Chroma,
                    StatValues {
                        mean: 4.0,
                        stddev: 4.0,
                        min: 4.0,
                        max: 4.0,
                    });
        fn t(target: TargetDirection) -> Target {
            Target::new(target,
                        Mean,
                        Chroma,
                        Strength {
                            factor: 3.0,
                            exponent: 2,
                        })
        }
        assert_eq!(t(Maximize).calculate(&data), 144.0);
        assert_eq!(t(Minimize).calculate(&data), -144.0);
        assert_eq!(t(MaximizeDifference(6.0)).calculate(&data), 36.0);
        assert_eq!(t(MinimizeDifference(6.0)).calculate(&data), -36.0);
    }

}
