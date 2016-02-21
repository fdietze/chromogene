use rand::{thread_rng, Rng, Rand};
use std::cmp::Ordering;
use std::f64::{MIN, MAX};

pub trait Genotype<G:Genotype<G> + Clone + Rand> {
    fn fitness(&self) -> f64;
    fn mutated<R: Rng>(&self, rng: &mut R, heat: f64) -> G;
    fn crossover<R: Rng>(&self, rng: &mut R, other: &G) -> G;
}

pub struct Population<G: Genotype<G> + Clone + Rand> {
    pub genotypes: Vec<G>,
    pub mutation_index: f64, // pub elitism: f64,
}

impl<G: Genotype<G> + Clone + Rand> Default for Population<G> {
    fn default() -> Population<G> {
        Population {
            genotypes: vec![],
            mutation_index: 0.25,
        }
    }
}

impl<G: Genotype<G> + Clone + Rand> Population<G> {
    pub fn new<R: Rng>(size: usize, rng: &mut R) -> Population<G> {
        let genotypes = (0..size).map(|_| rng.gen::<G>()).collect();
        Population { genotypes: genotypes, ..Default::default() }
    }
    pub fn iterate<R: Rng>(&mut self, rng: &mut R, heat: f64) -> G {
        let fitnesses: Vec<f64> = self.genotypes.iter().map(|g| g.fitness()).collect();

        let best = self.genotypes
                       .iter()
                       .zip(fitnesses.iter())
                       .fold((self.genotypes.first().unwrap(), MIN),
                             |(mg, mf), (g, &f)| {
                                 if f > mf {
                                     (g, f)
                                 } else {
                                     (mg, mf)
                                 }
                             })
                       .0
                       .clone();

        let min_fitness = fitnesses.iter().fold(MAX, |min, &x| min.min(x));
        let mut cumulative_fitnesses: Vec<f64> = Vec::with_capacity(self.genotypes.len());
        let mut sum = 0.0;
        for &fitness in fitnesses.iter() {
            let current = fitness - min_fitness; // shift lowest fitness to zero
            sum += current;
            cumulative_fitnesses.push(sum);
        }
        let sum = sum;
        let cumulative_fitnesses = cumulative_fitnesses;

        let old = self.genotypes.clone();
        for genotype in self.genotypes.iter_mut() {
            let parent_index_a = roulette_wheel_selection(&cumulative_fitnesses,
                                                          rng.gen_range(0.0, sum)); // fails when sum == 0.0 (all individuals have the same fitness)
            let parent_index_b = roulette_wheel_selection(&cumulative_fitnesses,
                                                          rng.gen_range(0.0, sum));
            let child = old[parent_index_a].crossover(rng, &old[parent_index_b]);
            // println!("selected {} {:5.3} -mutate-> {:5.3}",
            //          index,
            //          fitnesses[index],
            //          child.fitness());
            *genotype = if rng.gen_range(0.0, 1.0) < self.mutation_index {
                child.mutated(rng, heat)
            } else {
                child
            };
        }


        best
    }
}

fn roulette_wheel_selection(cumulative_fitness: &Vec<f64>, rand: f64) -> usize {
    cumulative_fitness.binary_search_by(|probe| {
                          if probe > &rand {
                              Ordering::Greater
                          } else {
                              Ordering::Less
                          }
                      })
                      .unwrap_err()
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
