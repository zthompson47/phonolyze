use ordered_float::OrderedFloat;
use palette::{FromColor, Lab, Srgb, Xyz};

#[allow(unused)]
#[derive(Default, Debug)]
struct Msh {
    m: f32,
    s: f32,
    h: f32,
}

impl Msh {
    fn adjust_hue(&self, m_unsaturated: f32) -> f32 {
        if self.m >= m_unsaturated {
            self.h
        } else {
            let h_spin =
                self.s * (m_unsaturated.powi(2) - self.m.powi(2)) / (self.m * self.s.sin());
            if self.h > -(std::f32::consts::PI / 3.0) {
                self.h + h_spin
            } else {
                self.h - h_spin
            }
        }
    }
}

impl FromColor<Lab> for Msh {
    fn from_color(v: Lab) -> Self {
        let m = (v.l * v.l + v.a * v.a + v.b * v.b).powf(0.5);
        let s = (v.l / m).acos();
        let h = (v.b / v.a).atan();

        Self { m, s, h }
    }
}

impl FromColor<Srgb> for Msh {
    fn from_color(srgb: Srgb) -> Self {
        let lrgb = srgb.into_linear();
        let xyz = Xyz::from_color(lrgb);
        let lab = Lab::from_color(xyz);

        Self::from_color(lab)
    }
}

impl FromColor<Msh> for Srgb {
    fn from_color(Msh { s, m, h }: Msh) -> Self {
        let l = s.cos() * m;
        let a = ((m.powi(2) - s.cos().powi(2) * m.powi(2)) / (1.0 + h.tan().powi(2))).powf(0.5);
        let b = a * h.tan();
        let xyz = Xyz::from_color(Lab::new(l, a, b));

        Self::from_color(xyz)
    }
}

fn _main() {
    let srgb = Srgb::new(1.0, 0.378342, 0.0);
    let lrgb = srgb.into_linear();
    let xyz = Xyz::from_color(lrgb);
    let lab = Lab::from_color(xyz);
    let msh = Msh::from_color(lab);

    println!("{srgb:?}");
    println!("{lrgb:?}");
    println!("{xyz:?}");
    println!("{lab:?}");
    println!("{msh:?}");

    let srgb = Srgb::from_color(msh);
    println!("{srgb:?}");

    let _ = image::ImageBuffer::from_fn(1024, 64, |w, _| {
        let color = interpolate_color(
            Srgb::new(0.0, 0.0, 1.0),
            Srgb::new(1.0, 0.0, 0.0),
            w as f32 / 1024.0,
        );

        let r = (color.red * 255.0).clamp(0.0, 255.0) as u8;
        let g = (color.green * 255.0).clamp(0.0, 255.0) as u8;
        let b = (color.blue * 255.0).clamp(0.0, 255.0) as u8;

        image::Rgba([r, g, b, 255])
    })
    .save("out.png");
}

fn rad_diff(a: f32, b: f32) -> f32 {
    (a - b).abs() - std::f32::consts::PI
}

fn interpolate_color(c1: Srgb, c2: Srgb, mut interp: f32) -> Srgb {
    let mut result = Msh::default();
    let (mut c1, mut c2) = (Msh::from_color(c1), Msh::from_color(c2));
    // ------------------------------| TODO RadDiff(c1.h, c2.h)
    if c1.s > 0.05 || c2.s > 0.05 || rad_diff(c1.h, c2.h) > std::f32::consts::PI / 3.0 {
        result.m = *[OrderedFloat(c1.m), OrderedFloat(c2.m), OrderedFloat(88.0)]
            .iter()
            .max()
            .unwrap()
            .as_ref();
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

    println!("------->> {:?}", result);
    Srgb::from_color(result)
}

pub fn create_gradient_texture(a: [f32; 3], b: [f32; 3]) -> Vec<u8> {
    (0..=255)
        .flat_map(|x| {
            let color = interpolate_color(
                Srgb::new(a[0], a[1], a[2]),
                Srgb::new(b[0], b[1], b[2]),
                x as f32 / 255.0,
            );

            let r = (color.red * 255.0).clamp(0.0, 255.0) as u8;
            let g = (color.green * 255.0).clamp(0.0, 255.0) as u8;
            let b = (color.blue * 255.0).clamp(0.0, 255.0) as u8;

            [r, g, b, 255]
        })
        .collect()
}
