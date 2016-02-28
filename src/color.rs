use palette::{Lab, Rgb};
use palette::pixel::Srgb;
use palette::Limited;

pub fn adjust_color_space(col: &Lab) -> Lab {
    clamp_to_rgb(col)
}

fn clamp_to_rgb(col: &Lab) -> Lab {
    let mut rgb: Rgb = (*col).into();
    rgb.clamp_self();
    rgb.into()
}

pub fn distance(col1: &Lab, col2: &Lab) -> f32 {
    let col1 = adjust_color_space(&col1);
    let col2 = adjust_color_space(&col2);
    // euclidean_distance(col1, col2)
    ciede2000(&col1, &col2)
}

#[allow(dead_code)]
pub fn euclidean_distance(a: &Lab, b: &Lab) -> f32 {
    (((a.l - b.l) * 100.0).powi(2) + ((a.a - b.a) * 128.0).powi(2) + ((a.b - b.b) * 128.0).powi(2))
        .sqrt()
}

macro_rules! srgb {
    ( $r:expr,$g:expr, $b:expr ) => {
        {
    Srgb::new($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
        .to_linear()
        .into()
        }
    };
}

pub fn term_bgcolor(color: Srgb, text: &str) -> String {
    // format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m(RGB({red:3} {green:3} {blue:3}))",
    format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m",
            red = (color.red * 255.0) as usize,
            green = (color.green * 255.0) as usize,
            blue = (color.blue * 255.0) as usize,
            text = text,
            )
}

pub fn term_fgcolor(color: Srgb, text: &str) -> String {
    // format!("\x1b[48;2;{red};{green};{blue}m{text}\x1b[0m(RGB({red:3} {green:3} {blue:3}))",
    format!("\x1b[38;2;{red};{green};{blue}m{text}\x1b[0m",
            red = (color.red * 255.0) as usize,
            green = (color.green * 255.0) as usize,
            blue = (color.blue * 255.0) as usize,
            text = text,
            )
}

pub fn print_color(color: &Lab) {
    let mut rgb: Rgb = (*color).into();
    rgb.clamp_self();
    let color = Srgb::from_linear(rgb);
    print!("{}", term_bgcolor(color, "   "));
}

pub fn print_colored_text(bg: &Lab, fg: &Lab, text: &str) {
    let mut rgb_bg: Rgb = (*bg).into();
    rgb_bg.clamp_self();
    let bg = Srgb::from_linear(rgb_bg);
    let mut rgb_fg: Rgb = (*fg).into();
    rgb_fg.clamp_self();
    let fg = Srgb::from_linear(rgb_fg);
    print!("{}", term_bgcolor(bg, &term_fgcolor(fg, text)));
}

pub fn print_col_dist(coldist: (&Lab, &Lab, f32)) {
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

// TODO: http://www.brucelindbloom.com/index.html?Eqn_DeltaE_CMC.html
// http://colormine.org/delta-e-calculator/cmc

pub fn ciede2000(lab1: &Lab, lab2: &Lab) -> f32 {
    use std::f64::consts::PI;
    // ported from: https://github.com/THEjoezack/ColorMine/blob/master/ColorMine/ColorSpaces/Comparisons/CieDe2000Comparison.cs

    let lab1l = (lab1.l * 100.0) as f64;
    let lab1a = (lab1.a * 128.0) as f64;
    let lab1b = (lab1.b * 128.0) as f64;
    let lab2l = (lab2.l * 100.0) as f64;
    let lab2a = (lab2.a * 128.0) as f64;
    let lab2b = (lab2.b * 128.0) as f64;

    // Set weighting factors to 1
    let k_l = 1.0;
    let k_c = 1.0;
    let k_h = 1.0;

    // Calculate Cprime1, Cprime2, Cabbar
    let c_star_1_ab = (lab1a * lab1a + lab1b * lab1b).sqrt();
    let c_star_2_ab = (lab2a * lab2a + lab2b * lab2b).sqrt();
    let c_star_average_ab = (c_star_1_ab + c_star_2_ab) / 2.0;

    let mut c_star_average_ab_pot7 = c_star_average_ab * c_star_average_ab * c_star_average_ab;
    c_star_average_ab_pot7 *= c_star_average_ab_pot7 * c_star_average_ab;

    let g = 0.5 * (1.0 - (c_star_average_ab_pot7 / (c_star_average_ab_pot7 + 6103515625.0)).sqrt()); //25^7
    let a1_prime = (1.0 + g) * lab1a;
    let a2_prime = (1.0 + g) * lab2a;

    let c_prime_1 = (a1_prime * a1_prime + lab1b * lab1b).sqrt();
    let c_prime_2 = (a2_prime * a2_prime + lab2b * lab2b).sqrt();
    // Angles in Degree.
    let h_prime_1 = (((lab1b).atan2(a1_prime) * 180.0 / PI) + 360.0) % 360.0;
    let h_prime_2 = (((lab2b).atan2(a2_prime) * 180.0 / PI) + 360.0) % 360.0;

    let delta_l_prime = lab2l - lab1l;
    let delta_c_prime = c_prime_2 - c_prime_1;

    let h_bar = (h_prime_1 - h_prime_2).abs();
    let delta_h_prime;
    if c_prime_1 * c_prime_2 == 0.0 {
        delta_h_prime = 0.0;
    } else {
        if h_bar <= 180.0 {
            delta_h_prime = h_prime_2 - h_prime_1;
        } else if h_bar > 180.0 && h_prime_2 <= h_prime_1 {
            delta_h_prime = h_prime_2 - h_prime_1 + 360.0;
        } else {
            delta_h_prime = h_prime_2 - h_prime_1 - 360.0;
        }
    }
    let delta_h_prime = 2.0 * (c_prime_1 * c_prime_2).sqrt() * (delta_h_prime * PI / 360.0).sin();

    // Calculate CIEDE2000
    let l_prime_average = (lab1l + lab2l) / 2.0;
    let c_prime_average = (c_prime_1 + c_prime_2) / 2.0;

    // Calculate h_prime_average

    let h_prime_average;
    if c_prime_1 * c_prime_2 == 0.0 {
        h_prime_average = 0.0;
    } else {
        if h_bar <= 180.0 {
            h_prime_average = (h_prime_1 + h_prime_2) / 2.0;
        } else if h_bar > 180.0 && (h_prime_1 + h_prime_2) < 360.0 {
            h_prime_average = (h_prime_1 + h_prime_2 + 360.0) / 2.0;
        } else {
            h_prime_average = (h_prime_1 + h_prime_2 - 360.0) / 2.0;
        }
    }
    let mut l_prime_average_minus_50_square = l_prime_average - 50.0;
    l_prime_average_minus_50_square *= l_prime_average_minus_50_square;

    let s_l = 1.0 +
              ((0.015 * l_prime_average_minus_50_square) /
               (20.0 + l_prime_average_minus_50_square).sqrt());
    let s_c = 1.0 + 0.045 * c_prime_average;
    let t = 1.0 - 0.17 * ((h_prime_average - 30.0).to_radians()).cos() +
            0.24 * ((h_prime_average * 2.0).to_radians()).cos() +
            0.32 * ((h_prime_average * 3.0 + 6.0).to_radians()).cos() -
            0.2 * ((h_prime_average * 4.0 - 63.0).to_radians()).cos();
    let s_h = 1.0 + 0.015 * t * c_prime_average;
    let mut h_prime_average_minus_275_div_25_square = (h_prime_average - 275.0) / 25.0;
    h_prime_average_minus_275_div_25_square *= h_prime_average_minus_275_div_25_square;
    let delta_theta = 30.0 * (-h_prime_average_minus_275_div_25_square).exp();

    let mut c_prime_average_pot_7 = c_prime_average * c_prime_average * c_prime_average;
    c_prime_average_pot_7 *= c_prime_average_pot_7 * c_prime_average;
    let r_c = 2.0 * (c_prime_average_pot_7 / (c_prime_average_pot_7 + 6103515625.0)).sqrt();

    let r_t = -((2.0 * delta_theta).to_radians()).sin() * r_c;

    let delta_l_prime_div_k_l_s_l = delta_l_prime / (s_l * k_l);
    let delta_c_prime_div_k_c_s_c = delta_c_prime / (s_c * k_c);
    let delta_h_prime_div_k_h_s_h = delta_h_prime / (s_h * k_h);

    let ciede2000 = (delta_l_prime_div_k_l_s_l * delta_l_prime_div_k_l_s_l +
                     delta_c_prime_div_k_c_s_c * delta_c_prime_div_k_c_s_c +
                     delta_h_prime_div_k_h_s_h * delta_h_prime_div_k_h_s_h +
                     r_t * delta_c_prime_div_k_c_s_c * delta_h_prime_div_k_h_s_h)
                        .sqrt();

    return ciede2000 as f32;
}

#[cfg(test)]
mod test {
    use super::*;
    use palette::Lab;

    fn ciede2000_case(l1: f32, a1: f32, b1: f32, l2: f32, a2: f32, b2: f32, de: f32) {
        let lab1 = Lab::new(l1 / 100.0, a1 / 128.0, b1 / 128.0);
        let lab2 = Lab::new(l2 / 100.0, a2 / 128.0, b2 / 128.0);
        assert_eq!((ciede2000(&lab1, &lab2) * 10000.0).round() / 10000.0, de);
    }

    // from http://www.ece.rochester.edu/~gsharma/ciede2000/dataNprograms/ciede2000testdata.txt
    #[test]
    fn ciede2000_test1() {
        ciede2000_case(50.0000, 2.6772, -79.7751, 50.0000, 0.0000, -82.7485, 2.0425);
    }
    #[test]
    fn ciede2000_test2() {
        ciede2000_case(50.0000, 2.6772, -79.7751, 50.0000, 0.0000, -82.7485, 2.0425);
    }
    #[test]
    fn ciede2000_test3() {
        ciede2000_case(50.0000, 3.1571, -77.2803, 50.0000, 0.0000, -82.7485, 2.8615);
    }
    #[test]
    fn ciede2000_test4() {
        ciede2000_case(50.0000, 2.8361, -74.0200, 50.0000, 0.0000, -82.7485, 3.4412);
    }
    #[test]
    fn ciede2000_test5() {
        ciede2000_case(50.0000,
                       -1.3802,
                       -84.2814,
                       50.0000,
                       0.0000,
                       -82.7485,
                       1.0000);
    }
    #[test]
    fn ciede2000_test6() {
        ciede2000_case(50.0000,
                       -1.1848,
                       -84.8006,
                       50.0000,
                       0.0000,
                       -82.7485,
                       1.0000);
    }
    #[test]
    fn ciede2000_test7() {
        ciede2000_case(50.0000,
                       -0.9009,
                       -85.5211,
                       50.0000,
                       0.0000,
                       -82.7485,
                       1.0000);
    }
    #[test]
    fn ciede2000_test8() {
        ciede2000_case(50.0000, 0.0000, 0.0000, 50.0000, -1.0000, 2.0000, 2.3669);
    }
    #[test]
    fn ciede2000_test9() {
        ciede2000_case(50.0000, -1.0000, 2.0000, 50.0000, 0.0000, 0.0000, 2.3669);
    }
    #[test]
    fn ciede2000_test10() {
        ciede2000_case(50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0009, 7.1792);
    }
    #[test]
    fn ciede2000_test114() {
        ciede2000_case(50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0010, 7.1792);
    }
    #[test]
    fn ciede2000_test15() {
        ciede2000_case(50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0011, 7.2195);
    }
    #[test]
    fn ciede2000_test16() {
        ciede2000_case(50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0012, 7.2195);
    }
    #[test]
    fn ciede2000_test17() {
        ciede2000_case(50.0000, -0.0010, 2.4900, 50.0000, 0.0009, -2.4900, 4.8045);
    }
    #[test]
    fn ciede2000_test18() {
        ciede2000_case(50.0000, -0.0010, 2.4900, 50.0000, 0.0010, -2.4900, 4.8045);
    }
    #[test]
    fn ciede2000_test19() {
        ciede2000_case(50.0000, -0.0010, 2.4900, 50.0000, 0.0011, -2.4900, 4.7461);
    }
    #[test]
    fn ciede2000_test20() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 50.0000, 0.0000, -2.5000, 4.3065);
    }
    #[test]
    fn ciede2000_test222() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 73.0000, 25.0000, -18.0000, 27.1492);
    }
    #[test]
    fn ciede2000_test23() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 61.0000, -5.0000, 29.0000, 22.8977);
    }
    #[test]
    fn ciede2000_test24() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 56.0000, -27.0000, -3.0000, 31.9030);
    }
    #[test]
    fn ciede2000_test25() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 58.0000, 24.0000, 15.0000, 19.4535);
    }
    #[test]
    fn ciede2000_test26() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 50.0000, 3.1736, 0.5854, 1.0000);
    }
    #[test]
    fn ciede2000_test27() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 50.0000, 3.2972, 0.0000, 1.0000);
    }
    #[test]
    fn ciede2000_test28() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 50.0000, 1.8634, 0.5757, 1.0000);
    }
    #[test]
    fn ciede2000_test29() {
        ciede2000_case(50.0000, 2.5000, 0.0000, 50.0000, 3.2592, 0.3350, 1.0000);
    }
    #[test]
    fn ciede2000_test30() {
        ciede2000_case(60.2574,
                       -34.0099,
                       36.2677,
                       60.4626,
                       -34.1751,
                       39.4387,
                       1.2644);
    }
    #[test]
    fn ciede2000_test332() {
        ciede2000_case(63.0109,
                       -31.0961,
                       -5.8663,
                       62.8187,
                       -29.7946,
                       -4.0864,
                       1.2630);
    }
    #[test]
    fn ciede2000_test33() {
        ciede2000_case(61.2901, 3.7196, -5.3901, 61.4292, 2.2480, -4.9620, 1.8731);
    }
    #[test]
    fn ciede2000_test34() {
        ciede2000_case(35.0831, -44.1164, 3.7933, 35.0232, -40.0716, 1.5901, 1.8645);
    }
    #[test]
    fn ciede2000_test35() {
        ciede2000_case(22.7233,
                       20.0904,
                       -46.6940,
                       23.0331,
                       14.9730,
                       -42.5619,
                       2.0373);
    }
    #[test]
    fn ciede2000_test36() {
        ciede2000_case(36.4612, 47.8580, 18.3852, 36.2715, 50.5065, 21.2231, 1.4146);
    }
    #[test]
    fn ciede2000_test37() {
        ciede2000_case(90.8027, -2.0831, 1.4410, 91.1528, -1.6435, 0.0447, 1.4441);
    }
    #[test]
    fn ciede2000_test38() {
        ciede2000_case(90.9257, -0.5406, -0.9208, 88.6381, -0.8985, -0.7239, 1.5381);
    }
    #[test]
    fn ciede2000_test39() {
        ciede2000_case(6.7747, -0.2908, -2.4247, 5.8714, -0.0985, -2.2286, 0.6377);
    }
    #[test]
    fn ciede2000_test40() {
        ciede2000_case(2.0776, 0.0795, -1.1350, 0.9033, -0.0636, -0.5514, 0.9082);
    }
}
