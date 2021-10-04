use super::{Vector3, Vector4};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct RgbColor(pub Vector3);

impl RgbColor {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        RgbColor(Vector3 { x: r, y: g, z: b })
    }

    pub fn r(self) -> f32 {
        self.0.x
    }

    pub fn g(self) -> f32 {
        self.0.y
    }

    pub fn b(self) -> f32 {
        self.0.z
    }

    pub fn with_a(self, a: f32) -> RgbaColor {
        RgbaColor::new(self.r(), self.g(), self.b(), a)
    }

    pub fn lerp(self, b: RgbColor, t: f32) -> Self {
        RgbColor(self.0.lerp(b.0, t))
    }
}

impl From<[f32; 3]> for RgbColor {
    fn from([r, g, b]: [f32; 3]) -> Self {
        RgbColor::new(r, g, b)
    }
}

impl From<(f32, f32, f32)> for RgbColor {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        RgbColor::new(r, g, b)
    }
}

impl Into<[f32; 3]> for RgbColor {
    fn into(self) -> [f32; 3] {
        [self.r(), self.g(), self.b()]
    }
}

impl Into<(f32, f32, f32)> for RgbColor {
    fn into(self) -> (f32, f32, f32) {
        (self.r(), self.g(), self.b())
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
#[repr(C)]
pub struct RgbaColor(pub Vector4);

impl RgbaColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        RgbaColor(Vector4 {
            x: r,
            y: g,
            z: b,
            w: a,
        })
    }

    pub fn r(self) -> f32 {
        self.0.x
    }

    pub fn g(self) -> f32 {
        self.0.y
    }

    pub fn b(self) -> f32 {
        self.0.z
    }

    pub fn a(self) -> f32 {
        self.0.w
    }

    pub fn rgb(self) -> RgbColor {
        RgbColor::new(self.r(), self.g(), self.b())
    }

    pub fn lerp(self, b: RgbaColor, t: f32) -> Self {
        RgbaColor(self.0.lerp(b.0, t))
    }

    pub fn premult(self) -> RgbColor {
        RgbColor::new(
            self.r() * self.a(),
            self.g() * self.a(),
            self.b() * self.a(),
        )
    }
}

impl From<[f32; 4]> for RgbaColor {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        RgbaColor::new(r, g, b, a)
    }
}

impl From<(f32, f32, f32, f32)> for RgbaColor {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        RgbaColor::new(r, g, b, a)
    }
}

impl Into<[f32; 4]> for RgbaColor {
    fn into(self) -> [f32; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }
}

impl Into<(f32, f32, f32, f32)> for RgbaColor {
    fn into(self) -> (f32, f32, f32, f32) {
        (self.r(), self.g(), self.b(), self.a())
    }
}
