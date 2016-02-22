extern crate palette;
extern crate rand;
#[macro_use]
extern crate lazy_static;

use palette::{Lab, Lch, Rgb};
use std::ops::Add;
use palette::Limited;
use palette::pixel::Srgb;

use rand::{thread_rng, Rng, Rand};
use rand::distributions::{Normal, IndependentSample};

mod genetic;
use genetic::{Population, Genotype};

#[macro_use]
mod color;
#[allow(unused_imports)]
use color::*;

#[derive(Debug, Clone, Default)]
struct ColorScheme {
    free_colors: Vec<Lab>,
    fitness: f32,
}

lazy_static! {
    static ref FIXED_COLORS: [Lab; 2] = [srgb!(0, 43, 54), srgb!(253, 246, 227)];
}
const FREE_COLOR_COUNT: usize = 8;

impl ColorScheme {
    pub fn preview(&self) {
        for color in FIXED_COLORS.iter() {
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
        for bg in FIXED_COLORS.iter() {
            for fg in self.free_colors.iter() {
                print_colored_text(bg, fg, "delgmpgl ");
            }
            println!("");
        }
        println!("");
    }
    fn fitness_print(&self, print: bool) -> f32 {
        // fn convert_to_desired
        fn distance(col1: &Lab, col2: &Lab) -> f32 {
            // euclidean_distance(col1, col2)
            ciede2000(col1, col2)
        };

        let fixed_free_dist: Vec<(&Lab, &Lab, f32)> = FIXED_COLORS.iter()
                                                                  .flat_map(|col1| {
                                                                      self.free_colors
                                                                          .iter()
                                                                          .map(move |col2| {
                                                                              (col1,
                                                                               col2,
                                                                               distance(col1, col2))
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

        let avg_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| dist)
                                     .fold(0.0, |sum, x| sum + x) /
                            free_dist.len() as f32;

        let avg_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| dist)
                                            .fold(0.0, |sum, x| sum + x) /
                             fixed_free_dist.len() as f32;

        let var_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| (dist - avg_free_dist).powi(2))
                                     .fold(0.0, |sum, x| sum + x) /
                            free_dist.len() as f32;

        let var_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| (dist - avg_fixed_dist).powi(2))
                                            .fold(0.0, |sum, x| sum + x) /
                             fixed_free_dist.len() as f32;

        let min_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| dist)
                                     .fold(std::f32::MAX, |min, x| min.min(x));

        let min_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| dist)
                                            .fold(std::f32::MAX, |min, x| min.min(x));

        let avg_chroma = self.free_colors
                             .iter()
                             .map(|&col| {
                                 let lch: Lch = col.into();
                                 lch.chroma * 128.0
                             })
                             .fold(0.0, |sum, x| sum + x) /
                         self.free_colors.len() as f32;
        let var_chroma = self.free_colors
                             .iter()
                             .map(|&col| {
                                 let lch: Lch = col.into();
                                 (lch.chroma * 128.0 - avg_chroma).powi(2)
                             })
                             .fold(0.0, |sum, x| sum + x) /
                         self.free_colors.len() as f32;

        let avg_luminance = self.free_colors
                                .iter()
                                .map(|&col| col.l * 100.0)
                                .fold(0.0, |sum, x| sum + x) /
                            self.free_colors.len() as f32;
        let var_luminance = self.free_colors
                                .iter()
                                .map(|&col| (col.l * 100.0 - avg_luminance).powi(2))
                                .fold(0.0, |sum, x| sum + x) /
                            self.free_colors.len() as f32;

        if print {
            for &coldist in fixed_free_dist.iter() {
                print_col_dist(coldist);
            }
            println!("min fixed distance: {:7.2}", min_fixed_dist);
            println!("avg fixed distance: {:7.2}", avg_fixed_dist);
            println!("var fixed distance: {:7.2}", var_fixed_dist);
            for &coldist in free_dist.iter() {
                print_col_dist(coldist);
            }
            println!("min free distance : {:7.2}", min_free_dist);
            println!("avg free distance : {:7.2}", avg_free_dist);
            println!("var free distance : {:7.2}", var_free_dist);
            println!("avg free chroma   : {:7.2}", avg_chroma);
            println!("var free chroma   : {:7.2}", var_chroma);
            println!("avg free luminance: {:7.2}", avg_luminance);
            println!("var free luminance: {:7.2}", var_luminance);
        }

        // fitness
        -var_fixed_dist * 2.0 - var_luminance.powi(2) - var_chroma + min_free_dist.powi(2)
    }
}

impl Rand for ColorScheme {
    fn rand<R: Rng>(rng: &mut R) -> ColorScheme {
        let free_colors: Vec<Lab> = (0..FREE_COLOR_COUNT)
                                        .map(|_| {
                                            Rgb::new(rng.gen_range(0.0, 1.0),
                                                     rng.gen_range(0.0, 1.0),
                                                     rng.gen_range(0.0, 1.0))
                                                .into()
                                        })
                                        .collect();

        ColorScheme { free_colors: free_colors, ..Default::default() }
    }
}

impl Genotype<ColorScheme> for ColorScheme {
    fn calculate_fitness(&mut self) {
        self.fitness = self.fitness_print(false);
    }
    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn mutated<R: Rng>(&self, mut rng: &mut R, heat: f32) -> ColorScheme {
        let distribution = Normal::new(0.0, 0.15);
        let mut mutated_free = self.free_colors.clone();
        for _ in 0..mutated_free.len() / 4 {
            let index = rng.gen_range(0, mutated_free.len());
            let diff = Lab::new(distribution.ind_sample(&mut rng) as f32 * heat,
                                distribution.ind_sample(&mut rng) as f32 * heat,
                                distribution.ind_sample(&mut rng) as f32 * heat);
            let mut lab = mutated_free[index] + diff;
            lab.clamp_self();
            let mut rgb: Rgb = lab.into();
            rgb.clamp_self();
            mutated_free[index] = rgb.into();
        }

        ColorScheme { free_colors: mutated_free, ..Default::default() }
    }

    fn crossover<R: Rng>(&self, rng: &mut R, other: &ColorScheme) -> ColorScheme {
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
}

fn main() {
    let generations = 4000;
    let population_size = 1000;

    let mut rng = thread_rng();
    let mut p: Population<ColorScheme> = Population::new(population_size, &mut rng);
    let mut latest: Option<ColorScheme> = None;
    for i in 0..generations {
        let heat = 0.5; //(1.0 - i as f32 / generations as f32).powi(2);
        let best = p.next_generation(&mut rng, heat);
        best.preview();
        println!("{:04}: best fitness: {:11.5}, heat: {:5.3}\n",
                 i,
                 best.fitness_print(false),
                 heat);
        latest = Some(best);
    }
    // latest.unwrap().fitness_print(true);
}
