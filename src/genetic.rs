use rand::{Rng, Rand};
use std::cmp::Ordering;
use rand::distributions::IndependentSample;
use stats::{stddev, mean};

pub trait Genotype<G:Genotype<G, P> + Clone, P: ProblemDescription<G, P>> {
    fn rand<R: Rng>(descr: &P, rng: &mut R) -> G;
    fn mutated<R: Rng>(&self, strength: f32, rng: &mut R) -> G;
    fn crossover<R: Rng>(&self, other: &G, rng: &mut R) -> G;

    fn get_fitness(&self) -> f32;
    fn set_fitness(&mut self, fitness: f32);
}

pub trait ProblemDescription<G:Genotype<G, P> + Clone, P: ProblemDescription<G, P>> {
    fn calculate_fitness(&self, genotype: &G) -> f32;
}

pub struct Population<G: Genotype<G, P> + Clone, P: ProblemDescription<G, P>> {
    pub genotypes: Vec<G>,
    pub mutation_index: f32,
    pub elitism: usize,
    pub problem_description: P,
}

impl<G: Genotype<G, P> + Clone, P: ProblemDescription<G, P>> Population<G, P> {
    pub fn new<R: Rng>(size: usize, problem_description: P, mut rng: &mut R) -> Population<G, P> {
        let genotypes = (0..size).map(|_| G::rand(&problem_description, &mut rng)).collect();
        Population {
            genotypes: genotypes,
            problem_description: problem_description,
            mutation_index: 1.0,
            elitism: 1,
        }
    }
    pub fn next_generation<R: Rng>(&mut self,
                                   mutation_strength: f32,
                                   rng: &mut R)
                                   -> (G, f32, f32) {
        for genotype in self.genotypes.iter_mut() {
            let fitness = self.problem_description.calculate_fitness(&genotype);
            genotype.set_fitness(fitness);
        }

        let mean_fitness = mean(self.genotypes.iter().map(|g| g.get_fitness())) as f32;
        let sd_fitness = stddev(self.genotypes.iter().map(|g| g.get_fitness())) as f32;

        self.genotypes.sort_by(|geno_a, geno_b| {
            if geno_a.get_fitness() > geno_b.get_fitness() {
                Ordering::Less
            } else if geno_a.get_fitness() < geno_b.get_fitness() {
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
fn tournament_selection<'a, G: Genotype<G, P> + Clone, P: ProblemDescription<G, P>, R: Rng>
    (genotypes: &'a Vec<G>,
     size: usize,
     rng: &mut R)
     -> &'a G {
    assert!(size >= 1);
    (0..size).fold(rng.choose(genotypes).unwrap(), |best, _| {
        let competitor = rng.choose(genotypes).unwrap();
        if competitor.get_fitness() > best.get_fitness() {
            competitor
        } else {
            best
        }
    })
}
