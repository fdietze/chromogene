use rand::{Rng, Rand};
use rand::distributions::{Sample, IndependentSample};
use rand::distributions::exponential::Exp1;


#[derive(Clone, Copy)]
pub struct Power {
    p: f64,
}

impl Power {
    pub fn new(p: f64) -> Power {
        assert!(p > 0.0, "Power::new called with `p` <= 0");
        assert!(p <= 1.0, "Power::new called with `p` > 1");
        Power { p: p }
    }
}

impl Sample<f64> for Power {
    fn sample<R: Rng>(&mut self, rng: &mut R) -> f64 {
        self.ind_sample(rng)
    }
}
impl IndependentSample<f64> for Power {
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> f64 {
        let Exp1(e) = rng.gen::<Exp1>();
        (1.0 - (-e).exp()).powf(1.0 / self.p) //TODO: store 1 / p in struct
    }
}
