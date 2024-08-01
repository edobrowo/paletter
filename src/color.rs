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
struct Hsv {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl Hsv {
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

    /// Create the corresponding HSV representation. Hue has range [0, 180].
    fn make_hsv(&self) -> Hsv {
        let rp = self.r as f32 / 255.0;
        let gp = self.g as f32 / 255.0;
        let bp = self.b as f32 / 255.0;

        let cmax = f32::max(rp, f32::max(gp, bp));
        let cmin = f32::min(rp, f32::min(gp, bp));
        let delta = cmax - cmin;

        let h = 30.0
            * if cmax == 0.0 {
                0.0
            } else if cmax == rp {
                ((gp - bp) / delta).rem_euclid(6.0)
            } else if cmax == gp {
                (bp - rp) / delta + 2.0
            } else {
                (rp - gp) / delta + 4.0
            };

        let s = 100.0 * if cmax != 0.0 { delta / cmax } else { 0.0 };

        let v = 100.0 * cmax;

        Hsv::new(
            f32::round(h) as u8,
            f32::round(s) as u8,
            f32::round(v) as u8,
        )
    }
}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.make_hsv().cmp(&other.make_hsv()))
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> Ordering {
        self.make_hsv().cmp(&other.make_hsv())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>3} {:>3} {:>3}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn max_channel_delta() {
        let colors = vec![
            Color::new(89, 226, 133),
            Color::new(124, 168, 127),
            Color::new(193, 63, 57),
            Color::new(161, 246, 173),
            Color::new(87, 168, 222),
            Color::new(226, 51, 166),
            Color::new(46, 185, 177),
        ];

        assert_eq!(Color::max_channel_delta(&colors), (RGBChannel::Green, 195));

        let colors = vec![
            Color::new(89, 226, 133),
            Color::new(188, 71, 241),
            Color::new(241, 252, 78),
            Color::new(87, 168, 222),
            Color::new(226, 51, 166),
            Color::new(46, 185, 177),
            Color::new(6, 158, 1),
            Color::new(214, 187, 235),
            Color::new(66, 153, 255),
            Color::new(162, 104, 164),
            Color::new(124, 168, 127),
            Color::new(193, 63, 57),
            Color::new(161, 246, 173),
            Color::new(86, 110, 209),
            Color::new(7, 118, 33),
            Color::new(212, 183, 182),
            Color::new(152, 187, 115),
            Color::new(29, 214, 37),
            Color::new(14, 125, 147),
            Color::new(224, 141, 239),
        ];

        assert_eq!(Color::max_channel_delta(&colors), (RGBChannel::Blue, 254));
    }

    #[test]
    fn average() {
        let colors = vec![
            Color::new(216, 126, 83),
            Color::new(87, 73, 32),
            Color::new(48, 84, 50),
            Color::new(80, 92, 233),
            Color::new(42, 166, 15),
            Color::new(57, 177, 182),
            Color::new(238, 15, 176),
        ];

        assert_eq!(Color::average(&colors), Color::new(110, 105, 110));

        let colors = vec![
            Color::new(80, 92, 233),
            Color::new(42, 166, 15),
            Color::new(57, 177, 182),
            Color::new(160, 251, 201),
            Color::new(93, 47, 225),
            Color::new(78, 231, 132),
            Color::new(55, 212, 194),
            Color::new(238, 15, 176),
            Color::new(118, 136, 206),
            Color::new(52, 114, 141),
            Color::new(216, 126, 83),
            Color::new(87, 73, 32),
            Color::new(48, 84, 50),
            Color::new(140, 21, 246),
            Color::new(125, 53, 60),
            Color::new(44, 37, 23),
            Color::new(246, 198, 85),
            Color::new(171, 119, 34),
            Color::new(96, 140, 65),
            Color::new(130, 251, 129),
        ];

        assert_eq!(Color::average(&colors), Color::new(114, 127, 126));
    }

    #[test]
    fn make_hsv() {
        let color = Color::new(2, 117, 186);
        let hsv = Hsv::new(101, 99, 73);
        assert_eq!(hsv, color.make_hsv());

        let color = Color::new(106, 152, 243);
        let hsv = Hsv::new(110, 56, 95);
        assert_eq!(hsv, color.make_hsv());

        let color = Color::new(145, 34, 121);
        let hsv = Hsv::new(156, 77, 57);
        assert_eq!(hsv, color.make_hsv());

        let color = Color::new(204, 114, 97);
        let hsv = Hsv::new(5, 52, 80);
        assert_eq!(hsv, color.make_hsv());

        let color = Color::new(110, 181, 114);
        let hsv = Hsv::new(62, 39, 71);
        assert_eq!(hsv, color.make_hsv());
    }
}
