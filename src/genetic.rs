use rand::{Rng, Rand};
use std::cmp::Ordering;
use rand_power::Power;
use rand::distributions::IndependentSample;

pub trait Genotype<G:Genotype<G> + Clone + Rand> {
    fn fitness(&self) -> f32;
    fn calculate_fitness(&mut self);
    fn mutated<R: Rng>(&self, strength: f32, rng: &mut R) -> G;
    fn crossover<R: Rng>(&self, other: &G, rng: &mut R) -> G;
}

pub struct Population<G: Genotype<G> + Clone + Rand> {
    pub genotypes: Vec<G>,
    pub mutation_index: f32,
    pub elitism: usize,
}

impl<G: Genotype<G> + Clone + Rand> Default for Population<G> {
    fn default() -> Population<G> {
        Population {
            genotypes: vec![],
            mutation_index: 1.0,
            elitism: 1,
        }
    }
}

impl<G: Genotype<G> + Clone + Rand> Population<G> {
    pub fn new<R: Rng>(size: usize, rng: &mut R) -> Population<G> {
        let genotypes = (0..size).map(|_| rng.gen::<G>()).collect();
        Population { genotypes: genotypes, ..Default::default() }
    }
    pub fn next_generation<R: Rng>(&mut self,
                                   mutation_strength: f32,
                                   rng: &mut R)
                                   -> (G, f32, f32) {
        for genotype in self.genotypes.iter_mut() {
            genotype.calculate_fitness();
        }

        let fitnesses = self.genotypes.iter().map(|g| g.fitness()).collect();
        let (avg, var) = avg_var(&fitnesses);

        self.genotypes.sort_by(|geno_a, geno_b| {
            if geno_a.fitness() > geno_b.fitness() {
                Ordering::Less
            } else if geno_a.fitness() < geno_b.fitness() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let old = self.genotypes.clone();
        let best = old[0].clone();

        let mutation_count = (self.mutation_index * self.genotypes.len() as f32).ceil() as usize;

        for (i, genotype) in self.genotypes.iter_mut().skip(self.elitism).enumerate() {
            let parent_a = tournament_selection(&old, 4, rng);
            let parent_b = tournament_selection(&old, 4, rng);
            let child = parent_a.crossover(&parent_b, rng);
            // let child = parent_a.clone();

            let child = if i < mutation_count {
                child.mutated(mutation_strength, rng)
            } else {
                child
            };

            *genotype = child;
        }


        (best, avg, var)
    }
}

#[allow(dead_code)]
fn tournament_selection<'a, G: Genotype<G> + Clone + Rand, R: Rng>(genotypes: &'a Vec<G>,
                                                                   size: usize,
                                                                   rng: &mut R)
                                                                   -> &'a G {
    assert!(size >= 1);
    (0..size).fold(rng.choose(genotypes).unwrap(), |best, _| {
        let competitor = rng.choose(genotypes).unwrap();
        if competitor.fitness() > best.fitness() {
            competitor
        } else {
            best
        }
    })
}

pub fn power_mutation<R: Rng>(power_distribution: &Power,
                              current_value: f64,
                              lower_bound: f64,
                              upper_bound: f64,
                              mut rng: &mut R)
                              -> f64 {
    // from: A new mutation operator for real coded genetic algorithms (Deep, Thakur)
    let t = (current_value - lower_bound) / (upper_bound - lower_bound);
    // println!("current: {}", current_value);
    // println!("lower: {}", lower_bound);
    // println!("upper: {}", upper_bound);
    let r = rng.gen_range(0.0, 1.0);
    let s = power_distribution.ind_sample(&mut rng);

    // println!("t: {}", t);
    // println!("r: {}, s: {}", r, s);

    let y = if t < r {
        current_value - s * (current_value - lower_bound)
    } else {
        current_value + s * (upper_bound - current_value)
    };

    // println!("y: {}", y);

    y
}

#[allow(dead_code)]
fn roulette_wheel_selection(cumulative_fitness: &Vec<f32>, rand: f32) -> usize {
    cumulative_fitness.binary_search_by(|probe| {
                          if probe > &rand {
                              Ordering::Greater
                          } else {
                              Ordering::Less
                          }
                      })
                      .unwrap_err()
}


pub fn avg_var(values: &Vec<f32>) -> (f32, f32) {
    let avg = values.iter()
                    .fold(0.0, |sum, x| sum + x) / values.len() as f32;

    let var = values.iter()
                    .map(|&value| (value - avg).powi(2))
                    .fold(0.0, |sum, x| sum + x) / values.len() as f32;

    (avg, var)
}


#[cfg(test)]
mod test {
    use super::*;
    use super::roulette_wheel_selection;

    #[test]
    fn roulette_wheel() {
        let wheel = vec![13.0, 20.0, 21.0, 30.0, 35.0];
        assert_eq!(roulette_wheel_selection(&wheel, 0.0), 0);
        assert_eq!(roulette_wheel_selection(&wheel, 10.0), 0);
        assert_eq!(roulette_wheel_selection(&wheel, 14.0), 1);
        assert_eq!(roulette_wheel_selection(&wheel, 20.0), 2);
        assert_eq!(roulette_wheel_selection(&wheel, 22.0), 3);
        assert_eq!(roulette_wheel_selection(&wheel, 34.0), 4);
        assert_eq!(roulette_wheel_selection(&wheel, 35.0), 5);
    }
}
