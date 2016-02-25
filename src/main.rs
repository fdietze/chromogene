#![allow(warnings)]

extern crate palette;
extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate stats;

use palette::{Lab, Lch, Rgb};
use std::ops::Add;
use palette::Limited;
use palette::pixel::Srgb;

use rand::{thread_rng, Rng, Rand};
use rand::distributions::{Normal, IndependentSample};
use rand::distributions::exponential::Exp1;

use stats::{stddev, mean};

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

fn simulate_color_space(col: &Lab) -> Lab {
    let mut rgb: Rgb = (*col).into();
    rgb.clamp_self();
    rgb.into()
}

// fn convert_to_desired
fn distance(col1: &Lab, col2: &Lab) -> f32 {
    let col1 = simulate_color_space(&col1);
    let col2 = simulate_color_space(&col2);
    // euclidean_distance(col1, col2)
    ciede2000(&col1, &col2)
}

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

        let mean_free_dist = mean(free_dist.iter().map(|&(_, _, dist)| dist)) as f32;
        let sd_free_dist = stddev(free_dist.iter().map(|&(_, _, dist)| dist)) as f32;

        let mean_fixed_dist = mean(fixed_free_dist.iter().map(|&(_, _, dist)| dist)) as f32;
        let sd_fixed_dist = stddev(fixed_free_dist.iter().map(|&(_, _, dist)| dist)) as f32;

        let min_free_dist = free_dist.iter()
                                     .map(|&(_, _, dist)| dist)
                                     .fold(std::f32::MAX, |min, x| min.min(x));

        let min_fixed_dist = fixed_free_dist.iter()
                                            .map(|&(_, _, dist)| dist)
                                            .fold(std::f32::MAX, |min, x| min.min(x));

        let mean_chroma = mean(self.free_colors.iter().map(|&col| {
            let lch: Lch = col.into();
            lch.chroma * 128.0
        })) as f32;
        let sd_chroma = stddev(self.free_colors.iter().map(|&col| {
            let lch: Lch = col.into();
            lch.chroma * 128.0
        })) as f32;

        let mean_luminance = mean(self.free_colors.iter().map(|&col| col.l * 100.0)) as f32;
        let sd_luminance = stddev(self.free_colors.iter().map(|&col| col.l * 100.0)) as f32;
        if print {
            for &coldist in fixed_free_dist.iter() {
                print_col_dist(coldist);
            }
            println!("min  fixed distance: {:7.2}", min_fixed_dist);
            println!("mean fixed distance: {:7.2}", mean_fixed_dist);
            println!("sd   fixed distance: {:7.2}", sd_fixed_dist);
            for &coldist in free_dist.iter() {
                print_col_dist(coldist);
            }
            println!("min  free distance : {:7.2}", min_free_dist);
            println!("mean free distance : {:7.2}", mean_free_dist);
            println!("sd   free distance : {:7.2}", sd_free_dist);
            println!("mean free chroma   : {:7.2}", mean_chroma);
            println!("sd   free chroma   : {:7.2}", sd_chroma);
            println!("mean free luminance: {:7.2}", mean_luminance);
            println!("sd   free luminance: {:7.2}", sd_luminance);
        }

        let mut fitness = 0.0;
        fitness += min_free_dist.powi(2) * 6.0;
        fitness += mean_free_dist;
        fitness += min_fixed_dist.powi(2) * 6.0;
        fitness += mean_fixed_dist;
        fitness += -sd_fixed_dist.powi(2) * 2.0;
        fitness += -sd_luminance.powi(4);
        fitness += -sd_chroma.powi(2) * 3.0;

        fitness
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
}

fn main() {
    let generations = 1000;
    let population_size = 50;
    let runs = 1;

    // benchmark parameters:
    // let generations = 500;
    // let population_size = 50;
    // let runs = 20;

    let mut run_stats = stats::OnlineStats::new();
    let mut run_minmax = stats::MinMax::new();
    for run in 0..runs {
        let mut rng = thread_rng();
        let mut p: Population<ColorScheme> = Population::new(population_size, &mut rng);

        let mut latest: Option<ColorScheme> = None;
        for i in 0..generations {
            let heat = (1.0 - i as f32 / generations as f32).powi(1);
            let stats = p.next_generation(heat, &mut rng);

            if i % (generations / 100) == 0 {
                stats.0.preview();
                println!("{:04}: best fitness: {:11.5}, avg: {:6.2}, sd: {:6.2}  heat: {:5.3}\n",
                         i,
                         stats.0.fitness_print(false),
                         stats.1,
                         stats.2,
                         heat);
            }

            latest = Some(stats.0);
        }
        let best = latest.unwrap();
        best.preview();
        run_stats.add(best.fitness());
        run_minmax.add(best.fitness());
        println!("{:8.3}", best.fitness());
        best.fitness_print(true);
    }
    println!("\nbest: {:8.3}\navg:  {:8.3}\nsd:   {:8.3}",
             run_minmax.max().unwrap(),
             run_stats.mean(),
             run_stats.stddev());
}
