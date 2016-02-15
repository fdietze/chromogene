extern crate palette;
extern crate rand;
use palette::{Rgb, Lab};
use palette::Limited;

use rand::{thread_rng, Rng};

mod genetic;
use genetic::{Population, Genotype};

fn distance(a: &Lab, b: &Lab) -> f32 {
    ((a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)).sqrt()
}

pub fn term_bgcolor(color: Rgb, text: &str) -> String {
    format!("\x1b[48;2;{};{};{}m{}\x1b[0m",
            (color.red * 255f32) as usize,
            (color.green * 255f32) as usize,
            (color.blue * 255f32) as usize,
            text)
}

pub fn print_color(color: &Lab) {
    print!("{}", term_bgcolor((*color).into(), "   "));
}


#[derive(Debug, Clone)]
struct ColorScheme {
    color_count: usize,
    target_distance: f32,
    fixed_colors: Vec<Lab>,
    free_colors: Vec<Lab>,
}

impl ColorScheme {
    fn random(count: usize, target_distance: f32, fixed_colors: Vec<Lab>) -> ColorScheme {
        let free_count = count as isize - fixed_colors.len() as isize;
        assert!(free_count >= 0);

        let mut rng = thread_rng();

        let mut free_colors = vec![];
        for _ in 0..free_count {
            let rgb = Rgb::new(rng.gen_range(0f32, 1f32),
                               rng.gen_range(0f32, 1f32),
                               rng.gen_range(0f32, 1f32));
            free_colors.push(rgb.into());
        }


        ColorScheme {
            color_count: count,
            target_distance: target_distance,
            fixed_colors: fixed_colors,
            free_colors: free_colors,
        }
    }

    pub fn preview(&self) {
        for color in self.fixed_colors.iter() {
            print_color(color);
        }
        println!("");
        for color in self.free_colors.iter() {
            print_color(color);
        }
        println!("");
    }
}

impl Genotype<ColorScheme> for ColorScheme {
    fn fitness(&self) -> f32 {
        let colors = self.fixed_colors.iter().chain(self.free_colors.iter());
        let mut distances = 0f32;
        // for all combinations
        for (i, el1) in colors.clone().enumerate() {
            for el2 in colors.clone().skip(i + 1) {
                let distance = distance(el1, el2);
                let error = (1f32 + (distance - self.target_distance).abs())
                                .powi(self.color_count as i32);
                distances += error;
                // print_color(el1);
                // print_color(el2);
                // println!(" distance: {}, error: {}", distance, error);
            }
        }
        let distance_count = self.color_count * (self.color_count - 1) / 2;
        let avg_error = distances / (distance_count as f32);
        let fitness = -avg_error;
        // println!("fitness: {}, avg error: {}\n", fitness, avg_error);
        fitness
    }
    fn mutated(&self) -> ColorScheme {
        let mut rng = thread_rng();
        let mutation = 0.2f32;
        let mutated_free = self.free_colors
                               .iter()
                               .map(|color| {
                                   let lab = Lab::new((color.l +
                                                       rng.gen_range(-mutation, mutation))
                                                          .max(0f32)
                                                          .min(1f32),
                                                      (color.a +
                                                       rng.gen_range(-mutation, mutation))
                                                          .max(-1f32)
                                                          .min(1f32),
                                                      (color.b +
                                                       rng.gen_range(-mutation, mutation))
                                                          .max(-1f32)
                                                          .min(1f32));
                                   let mut rgb: Rgb = lab.into();
                                   rgb.clamp_self();
                                   let lab = rgb.into();
                                   lab
                               })
                               .collect();
        ColorScheme {
            color_count: self.color_count,
            target_distance: self.target_distance,
            fixed_colors: self.fixed_colors.clone(),
            free_colors: mutated_free,
        }
    }

    fn create_random_population(size: usize) -> Population<ColorScheme> {
        let mut schemes = vec![];
        for _ in 0..size {
            schemes.push(ColorScheme::random(8,
                                             1f32,
                                             vec![Rgb::new(0.0, 43.0 / 255.0, 54.0 / 255.0)
                                                      .into(),
                                                  Rgb::new(253.0 / 255.0,
                                                           246.0 / 255.0,
                                                           227.0 / 255.0)
                                                      .into()]));
        }
        Population { genotypes: schemes }
    }
}

fn main() {
    let mut p = ColorScheme::create_random_population(100);
    for _ in 0..10000 {
        let best = p.iterate();
        best.preview();
    }
}
