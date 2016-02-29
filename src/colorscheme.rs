use palette::{Lab, Lch, Rgb};
use fitness::{ColorSchemeProblemDescription, FitnessData, StatValues};
use fitness::Parameter::*;
use color::*;
use genetic::Genotype;
use rand::Rng;
use rand::distributions::{Normal, IndependentSample};

#[derive(Debug, Clone, Default)]
pub struct ColorScheme {
    pub free_colors: Vec<Lab>,
    fitness: f32,
}

impl ColorScheme {
    pub fn preview(&self, descr: &ColorSchemeProblemDescription) {
        for color in descr.fixed_colors.iter() {
            print_color(color);
        }
        println!("");
        let mut sorted = self.free_colors.clone();
        sorted.sort_by_key(|&col| {
            let lch: Lch = col.into();
            (lch.hue.to_positive_degrees() * 100.0) as usize + (lch.l * 1000.0) as usize
        });
        for color in sorted.iter() {
            print_color(color);
        }

        println!("");
        for bg in descr.fixed_colors.iter() {
            for fg in self.free_colors.iter() {
                print_colored_text(bg, fg, "          ");
            }
            println!("");
            for fg in self.free_colors.iter() {
                print_colored_text(bg, fg, " delgmpgl ");
            }
            println!("");
            for fg in self.free_colors.iter() {
                print_colored_text(bg, fg, "          ");
            }
            println!("");
        }

        println!("");
        // for fg in descr.fixed_colors.iter() {
        //     for bg in self.free_colors.iter() {
        //         print_colored_text(bg, fg, "delgmpgl ");
        //     }
        //     println!("");
        // }

        // println!("");
        // for bg in self.free_colors.iter() {
        //     for fg in self.free_colors.iter() {
        //         print_colored_text(bg, fg, "Spiegelei ");
        //     }
        //     println!("");
        // }
        // println!("");
    }

    pub fn fitness_data(&self, descr: &ColorSchemeProblemDescription) -> FitnessData {
        let chroma: Vec<f32> = self.free_colors
                                   .iter()
                                   .map(|&col| {
                                       let lch: Lch = col.into();
                                       lch.chroma * 128.0
                                   })
                                   .collect();

        let luminance: Vec<f32> = self.free_colors.iter().map(|&col| col.l * 100.0).collect();

        let fixed_dist: Vec<f32> = descr.fixed_colors
                                        .iter()
                                        .flat_map(|col1| {
                                            self.free_colors
                                                .iter()
                                                .map(move |col2| distance(col1, col2))
                                        })
                                        .collect();

        let free_dist: Vec<f32> = self.free_colors
                                      .iter()
                                      .enumerate()
                                      .flat_map(|(i, col1)| {
                                          self.free_colors
                                              .iter()
                                              .skip(i + 1)
                                              .map(move |col2| distance(col1, col2))
                                      })
                                      .collect();


        let mut data = FitnessData::new();

        data.insert(Chroma, StatValues::from(&chroma));
        data.insert(Luminance, StatValues::from(&luminance));
        data.insert(FixedDistance, StatValues::from(&fixed_dist));
        data.insert(FreeDistance, StatValues::from(&free_dist));

        data
    }

    pub fn print_fitness(&self, descr: &ColorSchemeProblemDescription) {
        let fixed_dist: Vec<(&Lab, &Lab, f32)> = descr.fixed_colors
                                                      .iter()
                                                      .flat_map(|col1| {
                                                          self.free_colors
                                                              .iter()
                                                              .map(move |col2| {
                                                                  (col1, col2, distance(col1, col2))
                                                              })
                                                      })
                                                      .collect();

        let free_dist: Vec<(&Lab, &Lab, f32)> = self.free_colors
                                                    .iter()
                                                    .enumerate()
                                                    .flat_map(|(i, col1)| {
                                                        self.free_colors
                                                            .iter()
                                                            .skip(i + 1)
                                                            .map(move |col2| {
                                                                (col1, col2, distance(col1, col2))
                                                            })
                                                    })
                                                    .collect();

        // for &coldist in fixed_dist.iter() {
        //     print_col_dist(coldist);
        // }

        // for &coldist in free_dist.iter() {
        //     print_col_dist(coldist);
        // }

        let data = self.fitness_data(&descr);
        for t in descr.fitness_targets.values() {
            println!("{: <23} {: <6} {: <13} ( {:8.3} *{})^{} = {:11.3}",
                     format!("{:?}",t.direction),
                     format!("{:?}",t.stat),
                     format!("{:?}", t.parameter),
                     t.value(&data),
                     t.strength.factor,
                     t.strength.exponent,
                     t.calculate(&data),
                     );
        }
    }
}


impl Genotype<ColorScheme, ColorSchemeProblemDescription> for ColorScheme {
    fn rand<R: Rng>(descr: &ColorSchemeProblemDescription, rng: &mut R) -> ColorScheme {
        let free_colors: Vec<Lab> = (0..descr.free_color_count)
                                        .map(|_| {
                                            Rgb::new(rng.gen_range(0.0, 1.0),
                                                     rng.gen_range(0.0, 1.0),
                                                     rng.gen_range(0.0, 1.0))
                                                .into()
                                        })
                                        .collect();

        ColorScheme { free_colors: free_colors, ..Default::default() }
    }

    fn mutated<R: Rng>(&self, strength: f32, mut rng: &mut R) -> ColorScheme {
        let normal_distribution = Normal::new(0.0, 0.02 * strength as f64);
        let mut mutate = |x: f32, lower, upper| -> f32 {
            let diff = normal_distribution.ind_sample(&mut rng) as f32;
            (x + diff).min(upper).max(lower)
        };


        let mutated_free = self.free_colors
                               .iter()
                               .map(|old| {
                                   let new = Lab::new(mutate(old.l, 0.0, 1.0),
                                                      mutate(old.a, -1.0, 1.0),
                                                      mutate(old.b, -1.0, 1.0));
                                   assert!(new.l >= 0.0 && new.l <= 1.0);
                                   assert!(new.a >= -1.0 && new.a <= 1.0);
                                   assert!(new.b >= -1.0 && new.b <= 1.0);
                                   new
                               })
                               .collect();

        ColorScheme { free_colors: mutated_free, ..Default::default() }
    }

    fn crossover<R: Rng>(&self, other: &ColorScheme, rng: &mut R) -> ColorScheme {
        let mut sorted_a = self.free_colors.clone();
        sorted_a.sort_by_key(|&col| {
            let lch: Lch = col.into();
            (lch.hue.to_positive_degrees() * 100.0) as usize
        });
        let mut sorted_b = other.free_colors.clone();
        sorted_b.sort_by_key(|&col| {
            let lch: Lch = col.into();
            (lch.hue.to_positive_degrees() * 100.0) as usize
        });
        let free = sorted_a.iter()
            .zip(sorted_b.iter())
            .map(|(a, b)| (*a + *b) / 2.0)
            // .map(|(a, b)| if rng.gen::<bool>() {*a} else {*b})
            .collect();

        ColorScheme { free_colors: free, ..Default::default() }
    }

    fn set_fitness(&mut self, fitness: f32) {
        self.fitness = fitness;
    }
    fn get_fitness(&self) -> f32 {
        self.fitness
    }
}
