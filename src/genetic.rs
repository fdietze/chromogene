use rand::{Rng, Rand};
use std::cmp::Ordering;

pub trait Genotype<G:Genotype<G> + Clone + Rand> {
    fn fitness(&self) -> f32;
    fn calculate_fitness(&mut self);
    fn mutated<R: Rng>(&self, rng: &mut R, heat: f32) -> G;
    fn crossover<R: Rng>(&self, rng: &mut R, other: &G) -> G;
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
            mutation_index: 0.25,
            elitism: 1,
        }
    }
}

impl<G: Genotype<G> + Clone + Rand> Population<G> {
    pub fn new<R: Rng>(size: usize, rng: &mut R) -> Population<G> {
        let genotypes = (0..size).map(|_| rng.gen::<G>()).collect();
        Population { genotypes: genotypes, ..Default::default() }
    }
    pub fn next_generation<R: Rng>(&mut self, rng: &mut R, heat: f32) -> G {
        for genotype in self.genotypes.iter_mut() {
            genotype.calculate_fitness();
        }

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
        for genotype in self.genotypes.iter_mut().skip(self.elitism) {
            let parent_a = tournament_selection(&old, 4, rng);
            let parent_b = tournament_selection(&old, 4, rng);
            let child = parent_a.crossover(rng, &parent_b);

            *genotype = if rng.gen_range(0.0, 1.0) < self.mutation_index {
                child.mutated(rng, heat)
            } else {
                child
            };
        }


        let best = old[0].clone();
        best
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
