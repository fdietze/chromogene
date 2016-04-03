// #![allow(warnings)]

#![feature(iter_arith)]
#![feature(slice_patterns)]

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
use std::collections::HashMap;

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

use std::sync::mpsc::channel;
use std::thread;
use std::io;

enum Change {
    Target(Target),
    FreeColorCount(usize),
    FixedColors(Vec<Lab>),
}

fn line_to_target(line: &str) -> Result<Target, String> {
    let mut line = line.trim().split(" ");
    let direction = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "minimize" => Ok(Minimize),
            "maximize" => Ok(Maximize),
            "approximate" => {
                line.next()
                    .ok_or("expected string")
                    .and_then(|s| s.parse().map_err(|_| "expected float"))
                    .map(|f| Approximate(f))
            }

            _ => Err("expected minimize, maximize or approximate"),
        }
    }));

    let stat = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "mean" => Ok(Mean),
            "stddev" => Ok(StdDev),
            "min" => Ok(Min),
            "max" => Ok(Max),
            _ => Err("expected mean, stddev, min or max"),
        }
    }));

    let parameter = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "chroma" => Ok(Chroma),
            "luminance" => Ok(Luminance),
            "freedist" => Ok(FreeDistance),
            "fixeddist" => Ok(FixedDistance),
            _ => Err("expected chroma, luminance, freedist or fixeddist"),
        }
    }));

    let factor = line.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);

    let exponent = line.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    Ok(Target::new(direction,
                   stat,
                   parameter,
                   Strength {
                       factor: factor,
                       exponent: exponent,
                   }))
}

fn main() {
    let mut descr = ColorSchemeProblemDescription {
        free_color_count: 6,
        fixed_colors: vec![srgb!(0, 43, 54), srgb!(253, 246, 227)],
        // fixed_colors: vec![srgb!(51, 51, 51)],
        // fixed_colors: vec![srgb!(255, 255, 255)],
        fitness_targets: HashMap::new(),
    };

    let (tx, rx) = channel();
    let stdin_thread = thread::spawn(move || {
        loop {
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => tx.send(input).unwrap(),
                Err(_) => break,
            }
        }
    });


    let generations = 50000000;
    let population_size = 1000;
    let runs = 1;

    // benchmark parameters:
    // let generations = 500;
    // let population_size = 50;
    // let runs = 20;

    let mut run_stats = stats::OnlineStats::new();
    let mut run_minmax = stats::MinMax::new();
    let mut last_fitness_change = 0;
    for run in 0..runs {
        let mut rng = thread_rng();
        let mut p: Population<ColorScheme, ColorSchemeProblemDescription> =
            Population::new(population_size, descr.clone(), &mut rng);

        let mut latest: Option<ColorScheme> = None;
        for i in 0..generations {
            if let Ok(line) = rx.try_recv() {
                line_to_target(&line)
                    .map(|target| {
                        descr.set(target);
                        p.problem_description = descr.clone();
                        last_fitness_change = i;
                    })
                    .unwrap_or_else(|err| println!("{}", err));
            };

            let heat = (1.0 - (i - last_fitness_change) as f32 / 200 as f32).powi(1).max(0.01);
            let stats = p.next_generation(heat, &mut rng);

            // if generations < 100 || i % (generations / 100) == 0 {
            stats.0.preview(&descr);
            stats.0.print_fitness(&descr);
            println!("{:04}: best fitness: {:11.5}, avg: {:6.2}, sd: {:6.2}  heat: {:5.3}\n",
                     i,
                     stats.0.get_fitness(),
                     stats.1,
                     stats.2,
                     heat);
            // }

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

    stdin_thread.join().unwrap();
}
