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
            println!("min fixed distance: {}", min_fixed_dist);
            println!("avg fixed distance: {}", avg_fixed_dist);
            println!("var fixed distance: {}", var_fixed_dist);
            for &coldist in free_dist.iter() {
                print_col_dist(coldist);
            }
            println!("min free distance : {}", min_free_dist);
            println!("avg free distance : {}", avg_free_dist);
            println!("var free distance : {}", var_free_dist);
            println!("avg free chroma   : {}", avg_chroma);
            println!("var free chroma   : {}", var_chroma);
            println!("avg free luminance: {}", avg_luminance);
            println!("var free luminance: {}", var_luminance);
        }

        // fitness
        -var_fixed_dist.powi(2) + min_free_dist.powi(3) - var_free_dist - var_chroma.powi(2) -
        var_luminance.powi(2)
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
    let generations = 200;
    let population_size = 1000;
    let elitism = 0;

    let mut p = ColorScheme::create_random_population(population_size);
    let mut latest: Option<ColorScheme> = None;
    for i in 0..generations {
        let heat = 1.0 - i as f64 / generations as f64;
        let best = p.iterate(heat, elitism);
        best.preview();
        println!("{:04}: best fitness: {:11.5}, heat: {:5.3}\n",
                 i,
                 best.fitness_print(false),
                 heat);
        latest = Some(best);
    }
    latest.unwrap().fitness_print(true);
}
