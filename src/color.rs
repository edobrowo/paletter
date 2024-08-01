use std::cmp::Ordering;
use std::fmt;

/// RGB channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RGBChannel {
    Red,
    Green,
    Blue,
}

/// HSV representation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HSV {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl HSV {
    pub fn new(h: u8, s: u8, v: u8) -> Self {
        Self { h, s, v }
    }
}

/// Color represented in RGB24.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Channel-wise minimum.
    pub fn min(left: &Self, right: &Self) -> Self {
        Self::new(
            u8::min(left.r, right.r),
            u8::min(left.g, right.g),
            u8::min(left.b, right.b),
        )
    }

    /// Channel-wise maximum.
    pub fn max(left: &Self, right: &Self) -> Self {
        Self::new(
            u8::max(left.r, right.r),
            u8::max(left.g, right.g),
            u8::max(left.b, right.b),
        )
    }

    /// Finds the RGB channel with the highest delta.
    pub fn max_channel_delta(colors: &[Self]) -> (RGBChannel, u8) {
        let high = Self::new(u8::MAX, u8::MAX, u8::MAX);
        let low = Self::new(u8::MIN, u8::MIN, u8::MIN);

        let (min, max) = colors.iter().fold((high, low), |(min, max), val| {
            (Self::min(&min, val), Self::max(&max, val))
        });

        let delta = Self::new(max.r - min.r, max.g - min.g, max.b - min.b);

        if delta.r > delta.g && delta.r > delta.b {
            (RGBChannel::Red, delta.r)
        } else if delta.g > delta.r && delta.g > delta.b {
            (RGBChannel::Green, delta.g)
        } else {
            (RGBChannel::Blue, delta.b)
        }
    }

    /// Determines the average color.
    pub fn average(colors: &[Self]) -> Self {
        let (r, g, b) = colors.iter().fold((0, 0, 0), |sum, val| {
            (
                sum.0 + val.r as u64,
                sum.1 + val.g as u64,
                sum.2 + val.b as u64,
            )
        });

        let len = colors.len();

        Self::new(
            f32::round(r as f32 / len as f32) as u8,
            f32::round(g as f32 / len as f32) as u8,
            f32::round(b as f32 / len as f32) as u8,
        )
    }

    pub fn to_hex_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Create the corresponding HSV representation.
    fn to_hsv(&self) -> HSV {
        let rp = self.r as f32 / 255.0;
        let gp = self.g as f32 / 255.0;
        let bp = self.b as f32 / 255.0;

        let cmax = f32::max(rp, f32::max(gp, bp));
        let cmin = f32::min(rp, f32::min(gp, bp));
        let delta = cmax - cmin;

        let h = 60.0
            * if cmax == rp {
                (gp - bp) / delta % 6.0
            } else if cmax == gp {
                (bp - rp) / delta + 2.0
            } else {
                (rp - gp) / delta + 4.0
            };

        let s = if cmax != 0.0 { delta / cmax } else { 0.0 };

        let v = cmax;

        HSV::new(
            f32::round(h) as u8,
            f32::round(s) as u8,
            f32::round(v) as u8,
        )
    }
}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_hsv().cmp(&other.to_hsv()))
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_hsv().cmp(&other.to_hsv())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>3} {:>3} {:>3}", self.r, self.g, self.b)
    }
}
