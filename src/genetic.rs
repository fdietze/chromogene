use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::f64::{MIN, MAX};

pub trait Genotype<G:Genotype<G> + Clone> {
    // fn random() -> G;
    fn fitness(&self) -> f64;
    fn mutated(&self, heat: f64) -> G;
    fn crossover(&self, other: &G) -> G;
    fn create_random_population(n: usize) -> Population<G>;
}

pub struct Population<G: Genotype<G> + Clone> {
    pub genotypes: Vec<G>,
    pub mutation_index: f64,
}

impl<G: Genotype<G> + Clone> Population<G> {
    pub fn iterate(&mut self, heat: f64, elitism: usize) -> G {
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

        let max_fitness = fitnesses.iter().fold(MIN, |max, &x| max.max(x));
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

        let mut rng = thread_rng();

        // println!("fitnesses: {:#?}", fitnesses);
        // println!("min: {:5.3} max: {:5.3}", min_fitness, max_fitness);
        // println!("cumulative: {:#?}", cumulative_fitnesses);
        let old = self.genotypes.clone();
        for genotype in self.genotypes.iter_mut() {
            let parent_index_a = roulette_wheel_selection(&cumulative_fitnesses,
                                                          rng.gen_range(0.0, sum)); // fails when sum == 0.0 (all individuals have the same fitness)
            let parent_index_b = roulette_wheel_selection(&cumulative_fitnesses,
                                                          rng.gen_range(0.0, sum));
            let child = old[parent_index_a].crossover(&old[parent_index_b]);
            // println!("selected {} {:5.3} -mutate-> {:5.3}",
            //          index,
            //          fitnesses[index],
            //          child.fitness());
            *genotype = if rng.gen_range(0.0, 1.0) < self.mutation_index {
                child.mutated(heat)
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
