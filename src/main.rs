#![allow(warnings)]
#![feature(iter_arith)]

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

mod fitness;
use fitness::{ColorSchemeProblemDescription, FitnessData, StatValues};
use fitness::Parameter::*;
use fitness::Stat::*;
use fitness::TargetDirection::*;
use fitness::{Target, Strength};

mod genetic;
use genetic::{Population, Genotype, ProblemDescription};

#[macro_use]
mod color;
#[allow(unused_imports)]
use color::*;

mod colorscheme;
use colorscheme::ColorScheme;






fn main() {
    let descr = ColorSchemeProblemDescription {
        free_color_count: 8,
        fixed_colors: vec![srgb!(0, 43, 54), srgb!(253, 246, 227)],
        fitness_targets: vec![
            Target::new(Maximize, Min, FreeDistance, Strength { factor: 1.0, exponent: 2 }),
            // Target::new(FreeDistance, Mean, Strength { factor: 1.0, exponent: 1}),
            Target::new(Maximize, Min, FixedDistance, Strength { factor: 1.0, exponent: 1}),
            // Target::new(FixedDistance, Mean,Strength { factor: 1.0, exponent: 1}),
            Target::new(Minimize, StdDev, FixedDistance, Strength { factor: -2.0, exponent: 1}),
            Target::new(Minimize, StdDev, Luminance, Strength { factor: -1.0, exponent: 4}),
            Target::new(Minimize, StdDev, Chroma, Strength { factor: 1.0, exponent: 2}),
        ],
    };

    let generations = 200;
    let population_size = 100;
    let runs = 1;

    // benchmark parameters:
    // let generations = 500;
    // let population_size = 50;
    // let runs = 20;

    let mut run_stats = stats::OnlineStats::new();
    let mut run_minmax = stats::MinMax::new();
    for run in 0..runs {
        let mut rng = thread_rng();
        let mut p: Population<ColorScheme, ColorSchemeProblemDescription> =
            Population::new(population_size, descr.clone(), &mut rng);

        let mut latest: Option<ColorScheme> = None;
        for i in 0..generations {
            let heat = (1.0 - i as f32 / generations as f32).powi(1);
            let stats = p.next_generation(heat, &mut rng);

            if generations < 100 || i % (generations / 100) == 0 {
                stats.0.preview(&descr);
                println!("{:04}: best fitness: {:11.5}, avg: {:6.2}, sd: {:6.2}  heat: {:5.3}\n",
                         i,
                         stats.0.get_fitness(),
                         stats.1,
                         stats.2,
                         heat);
            }

            latest = Some(stats.0);
        }
        let best = latest.unwrap();
        best.preview(&descr);
        run_stats.add(best.get_fitness());
        run_minmax.add(best.get_fitness());
        println!("{:8.3}", best.get_fitness());
        best.print_fitness(&descr);
    }
    println!("\nbest: {:8.3}\navg:  {:8.3}\nsd:   {:8.3}",
             run_minmax.max().unwrap(),
             run_stats.mean(),
             run_stats.stddev());
}
