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
    pub fitness_targets: HashMap<(Stat, Parameter), Target>,
}

impl ColorSchemeProblemDescription {
    pub fn set(&mut self, target: Target) {
        self.fitness_targets.insert((target.stat, target.parameter), target);
    }
}

impl ProblemDescription<ColorScheme, ColorSchemeProblemDescription> for ColorSchemeProblemDescription {
    fn calculate_fitness(&self, scheme: &ColorScheme) -> f32 {
        let data = scheme.fitness_data(&self);
        self.fitness_targets.values().map(|target| target.calculate(&data)).sum::<f32>()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Parameter {
    Chroma,
    Luminance,
    FixedDistance,
    FreeDistance,
}

pub type FitnessData = HashMap<Parameter, StatValues>;


#[derive(Clone)]
pub struct Target {
    pub direction: TargetDirection,
    pub stat: Stat,
    pub parameter: Parameter,
    pub strength: Strength,
}

#[derive(Clone, Debug)]
pub enum TargetDirection {
    Maximize,
    Minimize,
    Approximate(f32),
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
    pub fn value(&self, data: &FitnessData) -> f32 {
        data.get(&self.parameter).unwrap().get(&self.stat)
    }
    pub fn calculate(&self, data: &FitnessData) -> f32 {
        let value = self.value(&data);
        match self.direction {
            TargetDirection::Maximize => self.strength.calculate(value),
            TargetDirection::Minimize => -self.strength.calculate(value),
            TargetDirection::Approximate(against) => {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
        assert_eq!(t(Approximate(6.0)).calculate(&data), -36.0);
    }

}
