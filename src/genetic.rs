use rand::{Rng, Rand};
use std::cmp::Ordering;
use rand::distributions::IndependentSample;
use stats::{stddev, mean};

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
        let mean_fitness = mean(self.genotypes.iter().map(|g| g.fitness())) as f32;
        let sd_fitness = stddev(self.genotypes.iter().map(|g| g.fitness())) as f32;

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


        (best, mean_fitness, sd_fitness)
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
