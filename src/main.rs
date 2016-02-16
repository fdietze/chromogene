extern crate palette;
extern crate rand;
use palette::{Lab, Rgb};
use palette::Limited;
use palette::pixel::Srgb;

use rand::{thread_rng, Rng};
use rand::distributions::{Normal, IndependentSample};

mod genetic;
use genetic::{Population, Genotype};

mod color;
use color::{ciede2000, euclidean_distance};

pub fn term_bgcolor(color: Srgb<f64>, text: &str) -> String {
    // format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m(RGB({red:3} {green:3} {blue:3}))",
    format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m",
    red = (color.red * 255f64) as usize,
    green = (color.green * 255f64) as usize,
    blue = (color.blue * 255f64) as usize,
    text = text,
    )
}

pub fn print_color(color: &Lab<f64>) {
    let mut rgb: Rgb<f64> = (*color).into();
    rgb.clamp_self();
    let color = Srgb::from_linear(rgb);
    print!("{}", term_bgcolor(color, "   "));
}


#[derive(Debug, Clone)]
struct ColorScheme {
    color_count: usize,
    fixed_colors: Vec<Lab<f64>>,
    free_colors: Vec<Lab<f64>>,
}

impl ColorScheme {
    fn random(count: usize, fixed_colors: Vec<Lab<f64>>) -> ColorScheme {
        let free_count = count as isize - fixed_colors.len() as isize;
        assert!(free_count >= 0);

        let mut rng = thread_rng();

        let mut free_colors = vec![];
        for _ in 0..free_count {
            let rgb = Rgb::<f64>::new(rng.gen_range(0f64, 1f64),
                                      rng.gen_range(0f64, 1f64),
                                      rng.gen_range(0f64, 1f64));
            free_colors.push(rgb.into());
        }


        ColorScheme {
            color_count: count,
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
    fn fitness_print(&self, print: bool) -> f64 {
        // let target_distance = 30f64;
        let colors = self.fixed_colors.iter().chain(self.free_colors.iter());
        // let mut sum_distance_error = 0f64;
        // for all combinations
        let mut min_dist = std::f64::MAX;
        let mut sum_dist = 0f64;
        for (i, el1) in colors.clone().enumerate() {
            for el2 in colors.clone().skip(i + 1) {
                // let distance = euclidean_distance(el1, el2);
                let distance = ciede2000(el1, el2);
                if distance < min_dist {
                    min_dist = distance;
                }
                sum_dist += distance;

                if print {
                    print_color(el1);
                    print_color(el2);
                    println!(" distance: {:5.2} Lab({:3.0} {:3.0} {:3.0}) Lab({:3.0} {:3.0} \
                              {:3.0})",
                             distance,
                             el1.l * 100.0,
                             el1.a * 128.0,
                             el1.b * 128.0,
                             el2.l * 100.0,
                             el2.a * 128.0,
                             el2.b * 128.0);
                }
            }
        }

        let distance_count = self.color_count * (self.color_count - 1) / 2;
        let avg_dist = sum_dist / distance_count as f64;

        let mut var_dist = 0f64;
        for (i, el1) in colors.clone().enumerate() {
            for el2 in colors.clone().skip(i + 1) {
                // let distance = euclidean_distance(el1, el2);
                let distance = ciede2000(el1, el2);
                var_dist += (distance - avg_dist) * (distance - avg_dist);
            }
        }
        var_dist /= distance_count as f64;


        if print {
            println!("min distance: {}", min_dist);
            println!("avg distance: {}", avg_dist);
            println!("var distance: {}", var_dist);
            println!("sum distance: {}", sum_dist);
        }
        // let avg_distance_error = sum_distance_error / (distance_count as f64);


        // let mut sum_luminance_error = 0f64;
        // let mut sum_chromacity_error = 0f64;
        // let avg_luminance = colors.clone().fold(0f64, |sum, c| sum + c.l) / self.color_count as f64;
        // let avg_chromacity = colors.clone()
        //                            .fold(0f64, |sum, c| sum + (c.a * c.a + c.b * c.b).sqrt()) /
        //                      self.color_count as f64;
        // for color in colors.clone() {
        //     sum_luminance_error += (1f64 + (avg_luminance - color.l).abs())
        //                                .powi(self.color_count as i32);
        //     sum_chromacity_error += (1f64 +
        //                              (avg_chromacity -
        //                               (color.a * color.a + color.b * color.b).sqrt())
        //                                  .abs())
        //                                 .powi(self.color_count as i32);
        // }

        // let avg_luminance_error = sum_luminance_error / (self.color_count as f64) * 2.0;
        // let avg_chromacity_error = sum_chromacity_error / (self.color_count as f64) * 200.0;


        let fitness = min_dist * min_dist - var_dist;
        // println!("avg luminance: {}, avg chromacity: {}",
        //          avg_luminance,
        //          avg_chromacity);
        // println!("avg distance error: {}\navg luminance error: {}\navg chromacity error: {}",
        //          avg_distance_error,
        //          avg_luminance_error,
        //          avg_chromacity_error,
        //          );
        fitness
    }
}

impl Genotype<ColorScheme> for ColorScheme {
    fn fitness(&self) -> f64 {
        self.fitness_print(false)
    }
    fn mutated(&self, heat: f64) -> ColorScheme {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 0.2);
        let mutated_free = self.free_colors
                               .iter()
                               .map(|color| {
                                   let lab = Lab::<f64>::new((color.l +
                                                              normal.ind_sample(&mut rng) * heat)
                                                                 .max(0f64)
                                                                 .min(1f64),
                                                             (color.a +
                                                              normal.ind_sample(&mut rng) * heat)
                                                                 .max(-1f64)
                                                                 .min(1f64),
                                                             (color.b +
                                                              normal.ind_sample(&mut rng) * heat)
                                                                 .max(-1f64)
                                                                 .min(1f64));
                                   let mut rgb: Rgb<f64> = lab.into();
                                   rgb.clamp_self();
                                   let lab = rgb.into();
                                   lab
                               })
                               .collect();
        ColorScheme {
            color_count: self.color_count,
            fixed_colors: self.fixed_colors.clone(),
            free_colors: mutated_free,
        }
    }

    fn create_random_population(size: usize) -> Population<ColorScheme> {
        let mut schemes = vec![];
        for _ in 0..size {
            schemes.push(ColorScheme::random(10,
                                             vec![
                                             Srgb::<f64>::new(0.0,
                                                                  43.0 / 255.0,
                                                                  54.0 / 255.0)
                                             .to_linear().into(),
                                             Srgb::<f64>::new(253.0 / 255.0,
                                                             246.0 / 255.0,
                                                             227.0 / 255.0)
                                             .to_linear().into(),
                                             ]));
        }
        Population { genotypes: schemes }
    }
}

fn main() {
    let generations = 100;
    let population_size = 1000;
    let elitism = 0;

    let mut p = ColorScheme::create_random_population(population_size);
    for i in 0..generations {
        let heat = 1.0 - i as f64 / generations as f64;
        let best = p.iterate(heat, elitism);
        best.preview();
        println!("{:04}: best fitness: {:11.5}, heat: {:5.3}",
                 i,
                 best.fitness_print(false),
                 heat);
    }
}
