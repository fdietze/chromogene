extern crate palette;
extern crate rand;
use palette::{Rgb, Lab};
use palette::Limited;

use rand::{thread_rng, Rng};
use rand::distributions::{Normal, IndependentSample};
use std::f64::consts::PI;

mod genetic;
use genetic::{Population, Genotype};

#[allow(dead_code)]
fn euclidean_distance(a: &Lab<f64>, b: &Lab<f64>) -> f64 {
    ((a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)).sqrt()
}

fn ciede2000(lab1: &Lab<f64>, lab2: &Lab<f64>) -> f64 {
    // ported from: https://github.com/THEjoezack/ColorMine/blob/master/ColorMine/ColorSpaces/Comparisons/CieDe2000Comparison.cs

    let lab1l = lab1.l as f64;
    let lab1a = lab1.a as f64;
    let lab1b = lab1.b as f64;
    let lab2l = lab2.l as f64;
    let lab2a = lab2.a as f64;
    let lab2b = lab2.b as f64;

    // Set weighting factors to 1
    let k_l = 1.0f64;
    let k_c = 1.0f64;
    let k_h = 1.0f64;

    // Calculate Cprime1, Cprime2, Cabbar
    let c_star_1_ab = (lab1a * lab1a + lab1b * lab1b).sqrt();
    let c_star_2_ab = (lab2a * lab2a + lab2b * lab2b).sqrt();
    let c_star_average_ab = (c_star_1_ab + c_star_2_ab) / 2f64;

    let mut c_star_average_ab_pot7 = c_star_average_ab * c_star_average_ab * c_star_average_ab;
    c_star_average_ab_pot7 *= c_star_average_ab_pot7 * c_star_average_ab;

    let g = 0.5f64 *
            (1f64 - (c_star_average_ab_pot7 / (c_star_average_ab_pot7 + 6103515625f64)).sqrt()); //25^7
    let a1_prime = (1f64 + g) * lab1a;
    let a2_prime = (1f64 + g) * lab2a;

    let c_prime_1 = (a1_prime * a1_prime + lab1b * lab1b).sqrt();
    let c_prime_2 = (a2_prime * a2_prime + lab2b * lab2b).sqrt();
    // Angles in Degree.
    let h_prime_1 = (((lab1b).atan2(a1_prime) * 180f64 / PI) + 360f64) % 360f64;
    let h_prime_2 = (((lab2b).atan2(a2_prime) * 180f64 / PI) + 360f64) % 360f64;

    let delta_l_prime = lab2l - lab1l;
    let delta_c_prime = c_prime_2 - c_prime_1;

    let h_bar = (h_prime_1 - h_prime_2).abs();
    let delta_h_prime;
    if c_prime_1 * c_prime_2 == 0f64 {
        delta_h_prime = 0f64;
    } else {
        if h_bar <= 180f64 {
            delta_h_prime = h_prime_2 - h_prime_1;
        } else if h_bar > 180f64 && h_prime_2 <= h_prime_1 {
            delta_h_prime = h_prime_2 - h_prime_1 + 360.0f64;
        } else {
            delta_h_prime = h_prime_2 - h_prime_1 - 360.0f64;
        }
    }
    let delta_h_prime = 2f64 * (c_prime_1 * c_prime_2).sqrt() * (delta_h_prime * PI / 360f64).sin();

    // Calculate CIEDE2000
    let l_prime_average = (lab1l + lab2l) / 2f64;
    let c_prime_average = (c_prime_1 + c_prime_2) / 2f64;

    // Calculate h_prime_average

    let h_prime_average;
    if c_prime_1 * c_prime_2 == 0f64 {
        h_prime_average = 0f64;
    } else {
        if h_bar <= 180f64 {
            h_prime_average = (h_prime_1 + h_prime_2) / 2f64;
        } else if h_bar > 180f64 && (h_prime_1 + h_prime_2) < 360f64 {
            h_prime_average = (h_prime_1 + h_prime_2 + 360f64) / 2f64;
        } else {
            h_prime_average = (h_prime_1 + h_prime_2 - 360f64) / 2f64;
        }
    }
    let mut l_prime_average_minus_50_square = l_prime_average - 50f64;
    l_prime_average_minus_50_square *= l_prime_average_minus_50_square;

    let s_l = 1f64 +
              ((0.015f64 * l_prime_average_minus_50_square) /
               (20f64 + l_prime_average_minus_50_square).sqrt());
    let s_c = 1f64 + 0.045f64 * c_prime_average;
    let t = 1f64 - 0.17f64 * ((h_prime_average - 30f64).to_radians()).cos() +
            0.24f64 * ((h_prime_average * 2f64).to_radians()).cos() +
            0.32f64 * ((h_prime_average * 3f64 + 6f64).to_radians()).cos() -
            0.2f64 * ((h_prime_average * 4f64 - 63f64).to_radians()).cos();
    let s_h = 1f64 + 0.015f64 * t * c_prime_average;
    let mut h_prime_average_minus_275_div_25_square = (h_prime_average - 275f64) / 25f64;
    h_prime_average_minus_275_div_25_square *= h_prime_average_minus_275_div_25_square;
    let delta_theta = 30f64 * (-h_prime_average_minus_275_div_25_square).exp();

    let mut c_prime_average_pot_7 = c_prime_average * c_prime_average * c_prime_average;
    c_prime_average_pot_7 *= c_prime_average_pot_7 * c_prime_average;
    let r_c = 2f64 * (c_prime_average_pot_7 / (c_prime_average_pot_7 + 6103515625f64)).sqrt();

    let r_t = -((2f64 * delta_theta).to_radians()).sin() * r_c;

    let delta_l_prime_div_k_l_s_l = delta_l_prime / (s_l * k_l);
    let delta_c_prime_div_k_c_s_c = delta_c_prime / (s_c * k_c);
    let delta_h_prime_div_k_h_s_h = delta_h_prime / (s_h * k_h);

    let ciede2000 = (delta_l_prime_div_k_l_s_l * delta_l_prime_div_k_l_s_l +
                     delta_c_prime_div_k_c_s_c * delta_c_prime_div_k_c_s_c +
                     delta_h_prime_div_k_h_s_h * delta_h_prime_div_k_h_s_h +
                     r_t * delta_c_prime_div_k_c_s_c * delta_h_prime_div_k_h_s_h)
                        .sqrt();

    return ciede2000 as f64;
}

pub fn term_bgcolor(color: Rgb<f64>, text: &str) -> String {
    format!("\x1b[48;2;{};{};{}m{}\x1b[0m",
            (color.red * 255f64) as usize,
            (color.green * 255f64) as usize,
            (color.blue * 255f64) as usize,
            text)
}

pub fn print_color(color: &Lab<f64>) {
    print!("{}", term_bgcolor((*color).into(), "   "));
}


#[derive(Debug, Clone)]
struct ColorScheme {
    color_count: usize,
    target_distance: f64,
    fixed_colors: Vec<Lab<f64>>,
    free_colors: Vec<Lab<f64>>,
}

impl ColorScheme {
    fn random(count: usize, target_distance: f64, fixed_colors: Vec<Lab<f64>>) -> ColorScheme {
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
    fn fitness(&self) -> f64 {
        let colors = self.fixed_colors.iter().chain(self.free_colors.iter());
        let mut sum_distance_error = 0f64;
        // for all combinations
        for (i, el1) in colors.clone().enumerate() {
            for el2 in colors.clone().skip(i + 1) {
                // let distance = distance(el1, el2);
                let distance = ciede2000(el1, el2);
                let error = (1f64 + (distance - self.target_distance).abs())
                                .powi(self.color_count as i32);
                sum_distance_error += error;
                // print_color(el1);
                // print_color(el2);
                // println!(" distance: {}, error: {}", distance, error);
            }
        }
        let distance_count = self.color_count * (self.color_count - 1) / 2;
        let avg_distance_error = sum_distance_error / (distance_count as f64);


        let mut sum_luminance_error = 0f64;
        let mut sum_chromacity_error = 0f64;
        let avg_luminance = colors.clone().fold(0f64, |sum, c| sum + c.l) / self.color_count as f64;
        let avg_chromacity = colors.clone()
                                   .fold(0f64, |sum, c| sum + (c.a * c.a + c.b * c.b).sqrt()) /
                             self.color_count as f64;
        for color in colors.clone() {
            sum_luminance_error += (1f64 + (avg_luminance - color.l).abs())
                                       .powi(self.color_count as i32);
            sum_chromacity_error += (1f64 +
                                     (avg_chromacity -
                                      (color.a * color.a + color.b * color.b).sqrt())
                                         .abs())
                                        .powi(self.color_count as i32);
        }

        let avg_luminance_error = sum_luminance_error / (self.color_count as f64) * 1.5;
        let avg_chromacity_error = sum_chromacity_error / (self.color_count as f64) * 1.5;


        let fitness = -avg_distance_error - avg_luminance_error - avg_chromacity_error;
        // println!("avg luminance: {}, avg chromacity: {}",
        //          avg_luminance,
        //          avg_chromacity);
        // println!("avg distance error: {}\navg luminance error: {}\navg chromacity error: {}",
        //          avg_distance_error,
        //          avg_luminance_error,
        //          avg_chromacity_error,
        //          );
        // println!("fitness: {}\n", fitness);
        fitness
    }
    fn mutated(&self) -> ColorScheme {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 0.01);
        let mutated_free = self.free_colors
                               .iter()
                               .map(|color| {
                                   let lab = Lab::<f64>::new((color.l +
                                                              normal.ind_sample(&mut rng))
                                                                 .max(0f64)
                                                                 .min(1f64),
                                                             (color.a +
                                                              normal.ind_sample(&mut rng))
                                                                 .max(-1f64)
                                                                 .min(1f64),
                                                             (color.b +
                                                              normal.ind_sample(&mut rng))
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
            target_distance: self.target_distance,
            fixed_colors: self.fixed_colors.clone(),
            free_colors: mutated_free,
        }
    }

    fn create_random_population(size: usize) -> Population<ColorScheme> {
        let mut schemes = vec![];
        for _ in 0..size {
            schemes.push(ColorScheme::random(8,
                                             1.1f64,
                                             vec![Rgb::<f64>::new(0.0,
                                                                  43.0 / 255.0,
                                                                  54.0 / 255.0)
                                             .into(),
                                             Rgb::<f64>::new(253.0 / 255.0,
                                                             246.0 / 255.0,
                                                             227.0 / 255.0)
                                             .into(),
                                             ]));
        }
        Population { genotypes: schemes }
    }
}

fn main() {
    let mut p = ColorScheme::create_random_population(1000);
    for i in 0..100 {
        let best = p.iterate();
        best.preview();
        println!("{}: best fitness: {}", i, best.fitness());
    }
}
