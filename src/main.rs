extern crate palette;
extern crate rand;
use palette::{Lab, Lch, Rgb};
use palette::Limited;
use palette::pixel::Srgb;

use rand::{thread_rng, Rng};
use rand::distributions::{Normal, IndependentSample};

mod genetic;
use genetic::{Population, Genotype};

mod color;
#[allow(unused_imports)]
use color::{ciede2000, euclidean_distance};

fn srgb(r: usize, g: usize, b: usize) -> Lab<f64> {
    Srgb::<f64>::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
        .to_linear()
        .into()
}


pub fn term_bgcolor(color: Srgb<f64>, text: &str) -> String {
    // format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m(RGB({red:3} {green:3} {blue:3}))",
    format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m",
            red = (color.red * 255f64) as usize,
            green = (color.green * 255f64) as usize,
            blue = (color.blue * 255f64) as usize,
            text = text,
            )
}

pub fn term_fgcolor(color: Srgb<f64>, text: &str) -> String {
    // format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m(RGB({red:3} {green:3} {blue:3}))",
    format!("\x1b[38;2;{red};{green};{blue}m{text}\x1b[0m",
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

pub fn print_colored_text(bg: &Lab<f64>, fg: &Lab<f64>, text: &str) {
    let mut rgb_bg: Rgb<f64> = (*bg).into();
    rgb_bg.clamp_self();
    let bg = Srgb::from_linear(rgb_bg);
    let mut rgb_fg: Rgb<f64> = (*fg).into();
    rgb_fg.clamp_self();
    let fg = Srgb::from_linear(rgb_fg);
    print!("{}", term_bgcolor(bg, &term_fgcolor(fg, text)));
}

fn print_col_dist(coldist: (&Lab<f64>, &Lab<f64>, f64)) {
    let (col1, col2, dist) = coldist;
    print_color(col1);
    print_color(col2);
    println!(" distance: {:5.2} Lab({:3.0} {:3.0} {:3.0}) Lab({:3.0} {:3.0} {:3.0})",
             dist,
             col1.l * 100.0,
             col1.a * 128.0,
             col1.b * 128.0,
             col2.l * 100.0,
             col2.a * 128.0,
             col2.b * 128.0);
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
            // TODO: colorblind transformation
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
        let mut sorted = self.free_colors.clone();
        sorted.sort_by_key(|&col| {
            let lch: Lch<f64> = col.into();
            (lch.hue.to_positive_degrees() * 100.0) as usize + (lch.l * 1000.0) as usize
        });
        for color in sorted.iter() {
            print_color(color);
        }

        println!("");
        for bg in self.fixed_colors.iter() {
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

        let fixed_free_dist: Vec<(&Lab<f64>, &Lab<f64>, f64)> = self.fixed_colors
                                                                    .iter()
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

impl Genotype<ColorScheme> for ColorScheme {
    fn fitness(&self) -> f64 {
        self.fitness_print(false)
    }
    fn mutated(&self, heat: f64) -> ColorScheme {
        let mut rng = thread_rng();
        let distribution = Normal::new(0.0, 0.15);
        let mutated_free = self.free_colors
                               .iter()
                               .map(|color| {
                                   let lab = Lab::<f64>::new((color.l +
                                                              distribution.ind_sample(&mut rng) *
                                                              heat)
                                                                 .max(0f64)
                                                                 .min(1f64),
                                                             (color.a +
                                                              distribution.ind_sample(&mut rng) *
                                                              heat)
                                                                 .max(-1f64)
                                                                 .min(1f64),
                                                             (color.b +
                                                              distribution.ind_sample(&mut rng) *
                                                              heat)
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
        let fixed = vec![srgb(0, 43, 54), srgb(253, 246, 227)];
        let total = 10;

        let schemes = (0..size)
                          .map(|_| ColorScheme::random(total, fixed.clone()))
                          .collect();
        Population { genotypes: schemes }
    }
}

fn main() {
    let generations = 100;
    let population_size = 100;
    let elitism = 0;

    let mut p = ColorScheme::create_random_population(population_size);
    let mut latest: Option<ColorScheme> = None;
    for i in 0..generations {
        let heat = (1.0 - i as f64 / generations as f64).powi(2);
        let best = p.iterate(heat, elitism);
        best.preview();
        println!("{:04}: best fitness: {:11.5}, heat: {:5.3}\n",
                 i,
                 best.fitness_print(false),
                 heat);
        latest = Some(best);
    }
    // latest.unwrap().fitness_print(true);
}
