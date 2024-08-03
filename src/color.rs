use std::cmp::Ordering;
use std::fmt;
use std::ops;

/// RGB channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RGBChannel {
    Red,
    Green,
    Blue,
}

impl RGBChannel {
    pub fn to_usize(self) -> usize {
        match self {
            RGBChannel::Red => 0,
            RGBChannel::Green => 1,
            RGBChannel::Blue => 2,
        }
    }
}

/// RGB24 representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgb24 {
    channels: [u8; 3],
}

impl Rgb24 {
    /// Creates a new RGB24 color.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            channels: [r, g, b],
        }
    }

    /// Accesses the red channel.
    pub fn r(&self) -> u8 {
        self.channels[0]
    }

    /// Accesses the green channel.
    pub fn g(&self) -> u8 {
        self.channels[1]
    }

    /// Accesses the blue channel.
    pub fn b(&self) -> u8 {
        self.channels[2]
    }

    /// Finds the channel-wise minimum.
    pub fn min(left: &Self, right: &Self) -> Self {
        Self::new(
            u8::min(left.r(), right.r()),
            u8::min(left.g(), right.g()),
            u8::min(left.b(), right.b()),
        )
    }

    /// Finds the channel-wise maximum.
    pub fn max(left: &Self, right: &Self) -> Self {
        Self::new(
            u8::max(left.r(), right.r()),
            u8::max(left.g(), right.g()),
            u8::max(left.b(), right.b()),
        )
    }

    /// Finds the channel with the greatest delta.
    pub fn max_channel_delta(colors: &[Self]) -> (RGBChannel, u8) {
        let high = Self::new(u8::MAX, u8::MAX, u8::MAX);
        let low = Self::new(u8::MIN, u8::MIN, u8::MIN);

        let (min, max) = colors.iter().fold((high, low), |(min, max), val| {
            (Self::min(&min, val), Self::max(&max, val))
        });

        let delta = Self::new(max.r() - min.r(), max.g() - min.g(), max.b() - min.b());

        if delta.r() > delta.g() && delta.r() > delta.b() {
            (RGBChannel::Red, delta.r())
        } else if delta.g() > delta.r() && delta.g() > delta.b() {
            (RGBChannel::Green, delta.g())
        } else {
            (RGBChannel::Blue, delta.b())
        }
    }

    /// Finds the channel-wise average.
    pub fn average(colors: &[Self]) -> Self {
        let (r, g, b) = colors.iter().fold((0, 0, 0), |sum, val| {
            (
                sum.0 + val.r() as u64,
                sum.1 + val.g() as u64,
                sum.2 + val.b() as u64,
            )
        });

        let len = colors.len();

        Self::new(
            f32::round(r as f32 / len as f32) as u8,
            f32::round(g as f32 / len as f32) as u8,
            f32::round(b as f32 / len as f32) as u8,
        )
    }

    // Builds a hex representation string.
    pub fn to_hex_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r(), self.g(), self.b())
    }

    /// Creates the corresponding HSV representation.
    /// Hue has range [0, 180] so that it fits in a single byte.
    fn make_hsv(&self) -> Hsv {
        let rp = self.r() as f32 / 255.0;
        let gp = self.g() as f32 / 255.0;
        let bp = self.b() as f32 / 255.0;

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

    /// Sorts a slice of colors using channel-based radix sort.
    pub fn radix_sort(colors: &mut [Self], channel: RGBChannel) {
        let channel = channel.to_usize();

        let mut buckets: Vec<Vec<Rgb24>> = vec![vec![]; 256];
        colors
            .iter()
            .for_each(|c| buckets[c[channel] as usize].push(c.clone()));

        for (i, color) in buckets.into_iter().flatten().enumerate() {
            colors[i] = color;
        }
    }

    pub fn level_index(&self, level: usize) -> usize {
        let inv = 7 - level;
        let mask = 0b10000000 >> level;
        ((self.r() & mask) >> inv << 2 | (self.g() & mask) >> inv << 1 | (self.b() & mask) >> inv)
            as usize
    }
}

impl ops::Index<usize> for Rgb24 {
    type Output = u8;

    fn index(&self, channel: usize) -> &Self::Output {
        &self.channels[channel]
    }
}

impl PartialOrd for Rgb24 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.make_hsv().cmp(&other.make_hsv()))
    }
}

impl Ord for Rgb24 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.make_hsv().cmp(&other.make_hsv())
    }
}

impl fmt::Display for Rgb24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>3} {:>3} {:>3}", self.r(), self.g(), self.b())
    }
}

/// HSV representation.
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn max_channel_delta() {
        let colors = vec![
            Rgb24::new(89, 226, 133),
            Rgb24::new(124, 168, 127),
            Rgb24::new(193, 63, 57),
            Rgb24::new(161, 246, 173),
            Rgb24::new(87, 168, 222),
            Rgb24::new(226, 51, 166),
            Rgb24::new(46, 185, 177),
        ];

        assert_eq!(Rgb24::max_channel_delta(&colors), (RGBChannel::Green, 195));

        let colors = vec![
            Rgb24::new(89, 226, 133),
            Rgb24::new(188, 71, 241),
            Rgb24::new(241, 252, 78),
            Rgb24::new(87, 168, 222),
            Rgb24::new(226, 51, 166),
            Rgb24::new(46, 185, 177),
            Rgb24::new(6, 158, 1),
            Rgb24::new(214, 187, 235),
            Rgb24::new(66, 153, 255),
            Rgb24::new(162, 104, 164),
            Rgb24::new(124, 168, 127),
            Rgb24::new(193, 63, 57),
            Rgb24::new(161, 246, 173),
            Rgb24::new(86, 110, 209),
            Rgb24::new(7, 118, 33),
            Rgb24::new(212, 183, 182),
            Rgb24::new(152, 187, 115),
            Rgb24::new(29, 214, 37),
            Rgb24::new(14, 125, 147),
            Rgb24::new(224, 141, 239),
        ];

        assert_eq!(Rgb24::max_channel_delta(&colors), (RGBChannel::Blue, 254));
    }

    #[test]
    fn average() {
        let colors = vec![
            Rgb24::new(216, 126, 83),
            Rgb24::new(87, 73, 32),
            Rgb24::new(48, 84, 50),
            Rgb24::new(80, 92, 233),
            Rgb24::new(42, 166, 15),
            Rgb24::new(57, 177, 182),
            Rgb24::new(238, 15, 176),
        ];

        assert_eq!(Rgb24::average(&colors), Rgb24::new(110, 105, 110));

        let colors = vec![
            Rgb24::new(80, 92, 233),
            Rgb24::new(42, 166, 15),
            Rgb24::new(57, 177, 182),
            Rgb24::new(160, 251, 201),
            Rgb24::new(93, 47, 225),
            Rgb24::new(78, 231, 132),
            Rgb24::new(55, 212, 194),
            Rgb24::new(238, 15, 176),
            Rgb24::new(118, 136, 206),
            Rgb24::new(52, 114, 141),
            Rgb24::new(216, 126, 83),
            Rgb24::new(87, 73, 32),
            Rgb24::new(48, 84, 50),
            Rgb24::new(140, 21, 246),
            Rgb24::new(125, 53, 60),
            Rgb24::new(44, 37, 23),
            Rgb24::new(246, 198, 85),
            Rgb24::new(171, 119, 34),
            Rgb24::new(96, 140, 65),
            Rgb24::new(130, 251, 129),
        ];

        assert_eq!(Rgb24::average(&colors), Rgb24::new(114, 127, 126));
    }

    #[test]
    fn make_hsv() {
        let color = Rgb24::new(2, 117, 186);
        let hsv = Hsv::new(101, 99, 73);
        assert_eq!(hsv, color.make_hsv());

        let color = Rgb24::new(106, 152, 243);
        let hsv = Hsv::new(110, 56, 95);
        assert_eq!(hsv, color.make_hsv());

        let color = Rgb24::new(145, 34, 121);
        let hsv = Hsv::new(156, 77, 57);
        assert_eq!(hsv, color.make_hsv());

        let color = Rgb24::new(204, 114, 97);
        let hsv = Hsv::new(5, 52, 80);
        assert_eq!(hsv, color.make_hsv());

        let color = Rgb24::new(110, 181, 114);
        let hsv = Hsv::new(62, 39, 71);
        assert_eq!(hsv, color.make_hsv());
    }

    #[test]
    fn level_handle() {
        let color = Rgb24::new(73, 153, 101);
        // 0b01001001
        // 0b10011001
        // 0b01100101

        assert_eq!(color.level_index(0), 2);
        assert_eq!(color.level_index(1), 5);
        assert_eq!(color.level_index(2), 1);
        assert_eq!(color.level_index(3), 2);
        assert_eq!(color.level_index(4), 6);
        assert_eq!(color.level_index(5), 1);
        assert_eq!(color.level_index(6), 0);
        assert_eq!(color.level_index(7), 7);
    }
}
