use std::cmp::Ordering;

pub trait Genotype<G:Genotype<G> + Clone> {
    fn fitness(&self) -> f64;
    fn mutated(&self, heat: f64) -> G;
    fn create_random_population(n: usize) -> Population<G>;
}

pub struct Population<G: Genotype<G> + Clone> {
    pub genotypes: Vec<G>,
}

impl<G: Genotype<G> + Clone> Population<G> {
    pub fn iterate(&mut self, heat: f64, elitism: usize) -> G {
        let mut fitnesses: Vec<(G, f64)> = self.genotypes
                                               .iter()
                                               .map(|g| (g.clone(), g.fitness()))
                                               .collect();
        fitnesses.sort_by(|&(_, fa), &(_, fb)| {
            if fa < fb {
                Ordering::Less
            } else if fa > fb {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        fitnesses.reverse();


        // normalization
        let &(_, max_fitness) = fitnesses.first().unwrap();
        let &(_, min_fitness) = fitnesses.last().unwrap();
        let interval = max_fitness - min_fitness;
        // sum of all shifted values
        let sum = fitnesses.iter().fold(0.0, |sum, &(_, x)| sum + x - min_fitness);

        let probs: Vec<f64> = fitnesses.iter()
                                       .map(|&(_, fitness)| (fitness - min_fitness) / sum)
                                       .collect();
        // (sum of all probabilities == 1)

        for genotype in self.genotypes.iter_mut() {
            // let parent_a = random_parent();
            // let parent_b = random_parent();
            // let crossed = parent_a.cross(parent_b);
            // let mutated = cross.mutated();
            // *genotype = mutated;
        }


        let (best, _) = fitnesses.iter().nth(0).unwrap().clone();
        best
    }
}
