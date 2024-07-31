use crate::color::{RGBChannel, Color};

/// Bucket represented as an index offset in a sequential container.
/// Also saves the maximum channel delta and a tag for that channel.
#[derive(Clone, Debug)]
struct Bucket {
    pub index: usize,
    pub channel: RGBChannel,
    pub delta: u8,
}

impl Bucket {
    pub fn new(index: usize, channel: RGBChannel, delta: u8) -> Self {
        Self { index, channel, delta }
    }
}

/// Median cut palette quantize implementation.
/// 
/// Given a list `colors` and `palette_size`, median cut
/// finds a set of colors (called the palette) of size `palette_size`
/// that approximate the distribution of colors in an image.
/// 
/// Median cut proceeds by organizing colors into buckets according
/// to a maximum channel delta heuristic. All colors in the list are
/// initially placed into one bucket. The maximum difference between
/// values in each channel is then computed. The bucket is then
/// sorted by the channel with the higest delta.
/// 
/// The bucket is then split at the median color. The maximum channel delta
/// is then computed again for each new bucket. The bucket with the highest
/// delta is then sorted by that channel, and the process repeats over
/// all buckets until the number of buckets equals `palette_size`.
/// 
/// The resulting palette is given by the averages of each bucket.
/// 
pub fn median_cut(colors: Vec<Color>, palette_size: usize) -> Vec<Color> {
    if palette_size >= colors.len() {
        return colors;
    }

    let mut colors = colors;
    let mut buckets: Vec<Bucket> = Vec::new();

    let (chan, delta) = Color::max_channel_delta(&colors);
    buckets.push(Bucket::new(0, chan, delta));

    // Sentinel bucket used for splitting at the end of the container.
    buckets.push(Bucket::new(colors.len(), chan, 0));

    while buckets.len() <= palette_size {
        let (i, max_bucket) = buckets
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.delta.cmp(&y.delta))
            .unwrap();

        let start = buckets[i].index;
        let end = buckets[i + 1].index;
        let mid = start + (end - start) / 2;

        let bucket_colors = &mut colors[start..end];

        match max_bucket.channel {
            RGBChannel::Red => bucket_colors.sort_by(|x, y| x.r.cmp(&y.r)),
            RGBChannel::Green => bucket_colors.sort_by(|x, y| x.g.cmp(&y.g)),
            RGBChannel::Blue => bucket_colors.sort_by(|x, y| x.b.cmp(&y.b)),
        };

        let (chan0, delta0) = Color::max_channel_delta(&colors[start..mid]);
        let (chan1, delta1) = Color::max_channel_delta(&colors[mid..end]);

        buckets[i] = Bucket::new(start, chan0, delta0);
        buckets.insert(i + 1, Bucket::new(mid, chan1, delta1));
    }

    buckets
        .iter()
        .zip(buckets.iter().skip(1))
        .map(|(a, b)| Color::average(&colors[a.index..b.index]))
        .collect()
}
