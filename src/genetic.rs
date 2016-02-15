pub trait Genotype<G:Genotype<G> + Clone> {
    fn fitness(&self) -> f64;
    fn mutated(&self) -> G;
    fn create_random_population(n: usize) -> Population<G>;
}

// #[derive(Ord)]
// pub struct PhenotypeWithFitness<G: Genotype<G>> {
//     fitness: f64,
//     genotype: G,
// }

pub struct Population<G: Genotype<G> + Clone> {
    pub genotypes: Vec<G>,
}

impl<G: Genotype<G> + Clone> Population<G> {
    pub fn iterate(&mut self) -> G {
        let first = self.genotypes.first().unwrap().clone();
        let (mut best, mut best_fitness) = (first.clone(), first.fitness());
        {
            let fitnesses = self.genotypes.iter().skip(1).map(|g| (g, g.fitness()));
            for (g, fitness) in fitnesses {
                // println!("fitness: {} best: {} fitness > best: {}",
                //          fitness,
                //          best_fitness,
                //          fitness > best_fitness);
                if fitness > best_fitness {
                    best = g.clone();
                    best_fitness = fitness;
                }
            }
        }
        println!("best fitness: {}", best_fitness);

        for old in self.genotypes.iter_mut().take(1) {
            *old = best.clone();
        }
        for old in self.genotypes.iter_mut().skip(1) {
            *old = best.mutated();
        }
        best
    }
}
