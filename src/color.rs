#![allow(unused)]
/// http://www.kennethmoreland.com/color-maps/ColorMapsExpanded.pdf
use ordered_float::OrderedFloat as F;

#[derive(Default)]
struct Srgb {
    r: f64,
    g: f64,
    b: f64,
}

impl Srgb {
    fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    fn tuple(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

impl From<LinearRgb> for Srgb {
    fn from(v: LinearRgb) -> Self {
        fn f(v: f64) -> f64 {
            if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        }

        Self {
            r: f(v.r),
            g: f(v.g),
            b: f(v.b),
        }
    }
}

struct LinearRgb {
    r: f64,
    g: f64,
    b: f64,
}

impl LinearRgb {
    fn tuple(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

struct Cielab {
    l: f64,
    a: f64,
    b: f64,
}

impl Cielab {
    fn tuple(&self) -> (f64, f64, f64) {
        (self.l, self.a, self.b)
    }
}

impl From<Srgb> for LinearRgb {
    fn from(value: Srgb) -> Self {
        fn convert(value: f64) -> f64 {
            let first = ((value + 0.055) / 1.055).powf(2.4);

            if first > 0.04045 {
                first
            } else {
                value / 12.92
            }
        }

        Self {
            r: convert(value.r),
            g: convert(value.g),
            b: convert(value.b),
        }
    }
}

impl From<Xyz> for LinearRgb {
    /// http://brucelindbloom.com/index.html?Eqn_RGB_to_XYZ.html
    fn from(v: Xyz) -> Self {
        Self {
            r: 3.2404542 * v.x - 1.5371385 * v.y - 0.4985314 * v.z,
            g: -0.9692660 * v.x + 1.8760108 * v.y + 0.0415560 * v.z,
            b: 0.0556434 + v.x - 0.2040259 * v.y + 1.0572252 * v.z,
        }
    }
}

struct Xyz {
    x: f64,
    y: f64,
    z: f64,
}

impl Xyz {
    fn tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

impl From<LinearRgb> for Xyz {
    fn from(v: LinearRgb) -> Self {
        Self {
            x: v.r * 0.4124 + v.g * 0.3576 + v.b * 0.1805,
            y: v.r * 0.2126 + v.g * 0.7152 + v.b * 0.0722,
            z: v.r * 0.0193 + v.g * 0.1192 + v.b * 0.9505,
        }
    }
}

impl From<Cielab> for Xyz {
    /// http://brucelindbloom.com/index.html?Eqn_RGB_to_XYZ.html
    fn from(v: Cielab) -> Self {
        let white = (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
        let e = 216.0 / 24389.0;
        let k = 24389.0 / 27.0;
        let fy = (v.l + 16.0) / 116.0;
        let fx = v.a / 500.0 + fy;
        let fz = fy - v.b / 200.0;
        let x = if fx.powi(3) > e {
            fx.powi(3)
        } else {
            (116.0 * fx - 16.0) / k
        };
        let y = if v.l > k * e {
            ((v.l + 16.0) / 116.0).powi(3)
        } else {
            v.l / k
        };
        let z = if fz.powi(3) > e {
            fz.powi(3)
        } else {
            (116.0 * fz - 16.0) / k
        };

        Self { x, y, z }
    }
}

impl From<Xyz> for Cielab {
    fn from(v: Xyz) -> Self {
        fn f(x: f64) -> f64 {
            if x > 0.008856 {
                x.powf(1.0 / 3.0)
            } else {
                7.787 * x + 16.0 / 116.0
            }
        }

        // TODO what is a reference white?
        let white = (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
        let x = f(v.x / white.0);
        let y = f(v.y / white.1);
        let z = f(v.z / white.2);

        Self {
            l: 116.0 * (y - 16.0 / 116.0),
            a: 500.0 * (x - z),
            b: 200.0 * (y - z),
        }
    }
}

#[derive(Default)]
struct Msh {
    m: f64,
    s: f64,
    h: f64,
}

impl From<Cielab> for Msh {
    fn from(v: Cielab) -> Self {
        let m = (v.l * v.l + v.a * v.a + v.b * v.b).powf(0.5);
        let s = (v.l / m).acos();
        let h = (v.b / v.a).atan();

        Self { m, s, h }
    }
}

impl From<Srgb> for Msh {
    fn from(v: Srgb) -> Self {
        Self::from(Cielab::from(Xyz::from(LinearRgb::from(v))))
    }
}

impl Msh {
    fn tuple(&self) -> (f64, f64, f64) {
        (self.m, self.s, self.h)
    }

    fn adjust_hue(&self, m_unsaturated: f64) -> f64 {
        if self.m >= m_unsaturated {
            self.h
        } else {
            let h_spin =
                (self.s * (m_unsaturated.powi(2) - self.m.powi(2)) / (self.m * self.s.sin()));
            if self.h > -(std::f64::consts::PI / 3.0) {
                self.h + h_spin
            } else {
                self.h - h_spin
            }
        }
    }
}

impl From<Msh> for Cielab {
    fn from(v: Msh) -> Self {
        let l = v.s.cos() * v.m;
        let a =
            ((v.m.powi(2) - v.s.cos().powi(2) * v.m.powi(2)) / (1.0 + v.h.tan().powi(2))).powf(0.5);
        let b = a * v.h.tan();

        Self { l, a, b }
    }
}

impl From<Msh> for Srgb {
    fn from(v: Msh) -> Self {
        Self::default()
    }
}

fn interpolate_color(c1: Srgb, c2: Srgb, mut interp: f64) -> Srgb {
    let mut result = Msh::default();
    let (mut c1, mut c2) = (Msh::from(c1), Msh::from(c2));
    // ------------------------------| TODO RadDiff(c1.h, c2.h)
    if c1.s > 0.05 || c2.s > 0.05 || (c1.h - c2.h) > std::f64::consts::PI / 3.0 {
        result.m = *[F(c1.m), F(c2.m), F(88.0)].iter().max().unwrap().as_ref();
        if interp < 0.5 {
            c2.m = result.m;
            c2.s = 0.0;
            c2.h = 0.0;
            interp *= 2.0;
        } else {
            c1.m = result.m;
            c1.s = 0.0;
            c1.h = 0.0;
            interp = 2.0 * interp - 1.0;
        }
    }

    if c1.s < 0.05 || c2.s > 0.05 {
        c1.h = c2.adjust_hue(c1.m);
    } else if c2.s < 0.05 || c1.s > 0.05 {
        c2.h = c1.adjust_hue(c2.m);
    }

    result.m = (1.0 - interp) * c1.m + interp * c2.m;
    result.s = (1.0 - interp) * c1.s + interp * c2.s;
    result.h = (1.0 - interp) * c1.h + interp * c2.h;

    Srgb::from(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use float_eq::assert_float_eq;

    #[test]
    fn sanity() {
        let srgb = Srgb::new(1.0, 0.378342, 0.0);

        let lrgb = LinearRgb::from(srgb);
        assert_float_eq!(
            lrgb.tuple(),
            (1.0, 0.1181919715165594, 0.0),
            ulps <= (1, 1, 1)
        );

        let xyz = Xyz::from(lrgb);
        assert_float_eq!(
            xyz.tuple(),
            (0.4546654490143216, 0.2971308980286433, 0.03338848300477388),
            ulps <= (1, 1, 1)
        );

        let cielab = Cielab::from(xyz);
        assert_float_eq!(
            cielab.tuple(),
            // here
            (95.63859084459776, 322.3005406174312, 99.59738415906087),
            ulps <= (1, 1, 1)
        );

        let msh = Msh::from(cielab);
        assert_float_eq!(
            msh.tuple(),
            (350.6337369283742, 1.29453647126602, 0.2997115447624592),
            ulps <= (1, 1, 1)
        );

        let cielab = Cielab::from(msh);
        assert_float_eq!(
            cielab.tuple(),
            (95.63859084459773, 322.30054061743124, 99.5973841590609),
            ulps <= (1, 1, 1)
        );

        let xyz = Xyz::from(cielab);
        assert_float_eq!(
            xyz.tuple(),
            (0.4546654490143216, 0.2971308980286433, 0.03338848300477388),
            ulps <= (1, 1, 1)
        );

        let lrgb = LinearRgb::from(xyz);
        assert_float_eq!(
            lrgb.tuple(),
            (1.0, 0.1181919715165594, 0.0),
            ulps <= (1, 1, 1)
        );

        let srgb = Srgb::from(lrgb);
        assert_float_eq!(srgb.tuple(), (1.0, 0.378342, 0.0), ulps <= (1, 1, 1))
    }
}
