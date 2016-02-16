use palette::Lab;
use std::f64::consts::PI;

#[allow(dead_code)]
pub fn euclidean_distance(a: &Lab<f64>, b: &Lab<f64>) -> f64 {
    (((a.l - b.l) * 100.0).powi(2) + ((a.a - b.a) * 128.0).powi(2) + ((a.b - b.b) * 128.0).powi(2))
        .sqrt()
}

// TODO: http://www.brucelindbloom.com/index.html?Eqn_DeltaE_CMC.html
// http://colormine.org/delta-e-calculator/cmc

pub fn ciede2000(lab1: &Lab<f64>, lab2: &Lab<f64>) -> f64 {
    // ported from: https://github.com/THEjoezack/ColorMine/blob/master/ColorMine/ColorSpaces/Comparisons/CieDe2000Comparison.cs

    let lab1l = lab1.l as f64 * 100.0;
    let lab1a = lab1.a as f64 * 128.0;
    let lab1b = lab1.b as f64 * 128.0;
    let lab2l = lab2.l as f64 * 100.0;
    let lab2a = lab2.a as f64 * 128.0;
    let lab2b = lab2.b as f64 * 128.0;

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

#[cfg(test)]
mod test {
    use super::*;
    use palette::Lab;

    fn ciede2000_case(l1: f64, a1: f64, b1: f64, l2: f64, a2: f64, b2: f64, de: f64) {
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
