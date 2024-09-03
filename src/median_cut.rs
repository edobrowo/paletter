use crate::color::{RGBChannel, Rgb24};

/// Bucket represented as an offset in a sequential container.
/// Also saves the maximum channel delta and a tag for that channel.
#[derive(Clone, Debug)]
struct Bucket {
    pub offset: usize,
    pub channel: RGBChannel,
    pub delta: u8,
}

impl Bucket {
    /// Create a new bucket.
    pub fn new(offset: usize, channel: RGBChannel, delta: u8) -> Self {
        Self {
            offset,
            channel,
            delta,
        }
    }
}

/// Finds the median cut of a vector of RGB24 colors.
///
/// Given a list `colors` and `palette_size`, median cut
/// finds a set of colors (called the palette) of size `palette_size`
/// that approximate the distribution of colors in an image.
///
/// Median cut proceeds by organizing colors into buckets according
/// to a maximum channel delta heuristic. All colors in the list are
/// initially placed into one bucket. The bucket is then sorted by
/// the channel with the greatest range.
///
/// The bucket is then split at the median color. The maximum channel delta
/// is then computed again for each new bucket. The bucket with the highest
/// delta is then sorted by that channel, and the process repeats over
/// all buckets until the number of buckets equals `palette_size`.
///
/// The resulting palette is the averages within each bucket.
///
pub fn median_cut(colors: Vec<Rgb24>, palette_size: usize) -> Vec<Rgb24> {
    let mut colors = colors;
    let mut buckets: Vec<Bucket> = Vec::with_capacity(palette_size + 1);

    let (chan, delta) = Rgb24::max_channel_delta(&colors);
    buckets.push(Bucket::new(0, chan, delta));

    // Sentinel bucket used for splitting at the end of the container.
    buckets.push(Bucket::new(colors.len(), chan, 0));

    while buckets.len() <= palette_size {
        let (i, max_bucket) = buckets
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.delta.cmp(&y.delta))
            .unwrap();

        let start = buckets[i].offset;
        let end = buckets[i + 1].offset;
        let mid = (start + end) / 2;

        let bucket_colors = &mut colors[start..end];

        Rgb24::radix_sort(bucket_colors, max_bucket.channel);

        let (chan0, delta0) = Rgb24::max_channel_delta(&colors[start..mid]);
        let (chan1, delta1) = Rgb24::max_channel_delta(&colors[mid..end]);

        buckets[i] = Bucket::new(start, chan0, delta0);
        buckets.insert(i + 1, Bucket::new(mid, chan1, delta1));
    }

    buckets
        .iter()
        .zip(buckets.iter().skip(1))
        .map(|(a, b)| Rgb24::average(&colors[a.offset..b.offset]))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn median_cut() {
        let colors = [
            Rgb24::new(254, 182, 47),
            Rgb24::new(147, 190, 63),
            Rgb24::new(144, 129, 150),
            Rgb24::new(247, 200, 162),
            Rgb24::new(209, 78, 31),
            Rgb24::new(205, 70, 224),
            Rgb24::new(169, 152, 157),
            Rgb24::new(5, 13, 222),
            Rgb24::new(78, 208, 20),
            Rgb24::new(98, 205, 81),
            Rgb24::new(196, 126, 248),
            Rgb24::new(240, 61, 100),
            Rgb24::new(85, 254, 97),
            Rgb24::new(191, 236, 235),
            Rgb24::new(47, 56, 6),
            Rgb24::new(81, 67, 179),
            Rgb24::new(172, 69, 24),
            Rgb24::new(181, 63, 74),
            Rgb24::new(95, 229, 108),
            Rgb24::new(154, 248, 89),
        ];

        let palette = vec![
            Rgb24::new(47, 56, 6),
            Rgb24::new(147, 190, 63),
            Rgb24::new(5, 13, 222),
            Rgb24::new(113, 98, 165),
            Rgb24::new(102, 229, 79),
            Rgb24::new(211, 91, 55),
            Rgb24::new(201, 98, 236),
            Rgb24::new(202, 196, 185),
        ];
        assert_eq!(palette, super::median_cut(colors.to_vec(), 8));

        let palette = vec![
            Rgb24::new(47, 56, 6),
            Rgb24::new(147, 190, 63),
            Rgb24::new(5, 13, 222),
            Rgb24::new(81, 67, 179),
            Rgb24::new(144, 129, 150),
            Rgb24::new(88, 207, 51),
            Rgb24::new(85, 254, 97),
            Rgb24::new(125, 239, 99),
            Rgb24::new(211, 62, 87),
            Rgb24::new(172, 69, 24),
            Rgb24::new(209, 78, 31),
            Rgb24::new(254, 182, 47),
            Rgb24::new(201, 98, 236),
            Rgb24::new(169, 152, 157),
            Rgb24::new(247, 200, 162),
            Rgb24::new(191, 236, 235),
        ];
        assert_eq!(palette, super::median_cut(colors.to_vec(), 16));
    }
}
