use std::cmp::Ordering;

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

        let elitism = 1;
        let elite = fitnesses.iter()
                             .take(elitism);

        for (old, x) in self.genotypes.iter_mut().zip(elite.clone()) {
            let (best, _) = x.clone();
            *old = best.clone();
        }
        for (old, x) in self.genotypes
                            .iter_mut()
                            .skip(elitism)
                            .zip(fitnesses.iter()
                                          .take(if elitism == 0 {
                                              1
                                          } else {
                                              0
                                          })
                                          .chain(elite.clone())
                                          .cycle()) {
            let (best, _) = x.clone();
            *old = best.mutated();
        }
        let (best, _) = fitnesses.iter().nth(0).unwrap().clone();
        best
    }
}
