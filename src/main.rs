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

#[derive(Debug, Clone)]
struct ColorScheme {
    free_colors: Vec<Lab<f64>>,
}

lazy_static! {
    static ref FIXED_COLORS: [Lab<f64>; 2] = [srgb!(0, 43, 54), srgb!(253, 246, 227)];
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
            let lch: Lch<f64> = col.into();
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
    fn fitness_print(&self, print: bool) -> f64 {
        // fn convert_to_desired
        fn distance(col1: &Lab<f64>, col2: &Lab<f64>) -> f64 {
            // euclidean_distance(col1, col2)
            ciede2000(col1, col2)
        };

        let fixed_free_dist: Vec<(&Lab<f64>, &Lab<f64>, f64)> = FIXED_COLORS.iter()
                                                                            .flat_map(|col1| {
                                                                                self.free_colors
                    .iter()
                    .map(move |col2| {
                        (col1,
                         col2,
                         distance(col1,
                                  col2))
                    })
                                                                            })
                                                                            .collect();

        let free_dist: Vec<(&Lab<f64>, &Lab<f64>, f64)> = self.free_colors
                                                              .iter()
                                                              .enumerate()
                                                              .flat_map(|(i, col1)| {
                                                                  self.free_colors
                                                                      .iter()
                                                                      .skip(i + 1)
                                                                      .map(move |col2| {
                                                                          (col1,
                                                                           col2,
                                                                           distance(col1, col2))
                                                                      })
                                                              })
                                                              .collect();

        let avg_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| dist)
                                     .fold(0.0, |sum, x| sum + x) /
                            free_dist.len() as f64;

        let avg_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| dist)
                                            .fold(0.0, |sum, x| sum + x) /
                             fixed_free_dist.len() as f64;

        let var_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| (dist - avg_free_dist).powi(2))
                                     .fold(0.0, |sum, x| sum + x) /
                            free_dist.len() as f64;

        let var_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| (dist - avg_fixed_dist).powi(2))
                                            .fold(0.0, |sum, x| sum + x) /
                             fixed_free_dist.len() as f64;

        let min_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| dist)
                                     .fold(std::f64::MAX, |min, x| min.min(x));

        let min_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| dist)
                                            .fold(std::f64::MAX, |min, x| min.min(x));

        let avg_chroma = self.free_colors
                             .iter()
                             .map(|&col| {
                                 let lch: Lch<f64> = col.into();
                                 lch.chroma * 128.0
                             })
                             .fold(0.0, |sum, x| sum + x) /
                         self.free_colors.len() as f64;
        let var_chroma = self.free_colors
                             .iter()
                             .map(|&col| {
                                 let lch: Lch<f64> = col.into();
                                 (lch.chroma * 128.0 - avg_chroma).powi(2)
                             })
                             .fold(0.0, |sum, x| sum + x) /
                         self.free_colors.len() as f64;

        let avg_luminance = self.free_colors
                                .iter()
                                .map(|&col| col.l * 100.0)
                                .fold(0.0, |sum, x| sum + x) /
                            self.free_colors.len() as f64;
        let var_luminance = self.free_colors
                                .iter()
                                .map(|&col| (col.l * 100.0 - avg_luminance).powi(2))
                                .fold(0.0, |sum, x| sum + x) /
                            self.free_colors.len() as f64;

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
        let free_colors: Vec<Lab<f64>> = (0..FREE_COLOR_COUNT)
                                             .map(|_| {
                                                 Rgb::<f64>::new(rng.gen_range(0f64, 1f64),
                                                                 rng.gen_range(0f64, 1f64),
                                                                 rng.gen_range(0f64, 1f64))
                                                     .into()
                                             })
                                             .collect();

        ColorScheme { free_colors: free_colors }
    }
}

impl Genotype<ColorScheme> for ColorScheme {
    fn fitness(&self) -> f64 {
        self.fitness_print(false)
    }
    fn mutated<R: Rng>(&self, mut rng: &mut R, heat: f64) -> ColorScheme {
        let distribution = Normal::new(0.0, 0.15);
        let mutated_free = self.free_colors
                               .iter()
                               .map(|color| {
                                   let diff = Lab::<f64>::new(distribution.ind_sample(&mut rng) *
                                                              heat,
                                                              distribution.ind_sample(&mut rng) *
                                                              heat,
                                                              distribution.ind_sample(&mut rng) *
                                                              heat);
                                   let mut lab = *color + diff;
                                   lab.clamp_self();
                                   let mut rgb: Rgb<f64> = lab.into();
                                   rgb.clamp_self();
                                   let lab = rgb.into();
                                   lab
                               })
                               .collect();
        ColorScheme { free_colors: mutated_free }
    }

    fn crossover<R: Rng>(&self, rng: &mut R, other: &ColorScheme) -> ColorScheme {
        let free = self.free_colors
                       .iter()
                       .zip(other.free_colors.iter())
                       .map(|(a, b)| {
                           if rng.gen::<bool>() {
                               *a
                           } else {
                               *b
                           }
                       })
                       .collect();

        ColorScheme { free_colors: free }
    }
}

fn main() {
    let generations = 100;
    let population_size = 1000;

    let mut rng = thread_rng();
    let mut p: Population<ColorScheme> = Population::new(population_size, &mut rng);
    let mut latest: Option<ColorScheme> = None;
    for i in 0..generations {
        let heat = 0.04;//(1.0 - i as f64 / generations as f64).powi(2);
        let best = p.iterate(&mut rng, heat);
        best.preview();
        println!("{:04}: best fitness: {:11.5}, heat: {:5.3}\n",
                 i,
                 best.fitness_print(false),
                 heat);
        latest = Some(best);
    }
    // latest.unwrap().fitness_print(true);
}
