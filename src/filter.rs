use palette::Rgb;
use std::cmp;

pub trait ColorFilter {
    fn transform(&self, Rgb) -> Rgb;
}

pub struct RedGreenFilter {
    k1: i32,
    k2: i32,
    k3: i32,
}

pub struct LinTriple {
    r: i32,
    g: i32,
    b: i32,
}

fn rgb2lin(rgb: Rgb) -> LinTriple {
    LinTriple {
        r: (rgb.red * 32767.0) as i32,
        g: (rgb.green * 32767.0) as i32,
        b: (rgb.blue * 32767.0) as i32,
    }
}

impl ColorFilter for RedGreenFilter {
    fn transform(&self, rgb: Rgb) -> Rgb {
        // get linear rgb values in the range 0..2^15-1
        let lin = rgb2lin(rgb);

        // simulated red and green are identical
        // scale the matrix values to 0..2^15 for integer computations 
        // of the simulated protan values.
        // divide after the computation by 2^15 to rescale.
        // also divide by 2^15 and multiply by 2^8 to scale the linear rgb to 0..255
        // total division is by 2^15 * 2^15 / 2^8 = 2^22
        // shift the bits by 22 places instead of dividing
        let r_blind = (self.k1 * lin.r + self.k2 * lin.g) >> 22;
        let b_blind = (self.k3 * lin.r - self.k3 * lin.g + 32768 * lin.b) >> 22;

        let red = cmp::max(0, cmp::min(255, r_blind));
        let blue = cmp::max(0, cmp::min(255, b_blind));

        // convert reduced linear rgb to gamma corrected rgb
        let red: i32 = if red >= 0 { red } else { 256 + red }; // from unsigned to signed
        let blue: i32 = if blue >= 0 { blue } else { 256 + blue }; // from unsigned to signed

        let out = Rgb::new_u8(red as u8, red as u8, blue as u8);

        out
    }
}

pub fn deutan() -> RedGreenFilter {
    RedGreenFilter{
        k1: 9591,
        k2: 23173,
        k3: -730,
    }
}

pub fn protan() -> RedGreenFilter {
    RedGreenFilter{
        k1: 3683,
        k2: 29084,
        k3: 131,
    }
}