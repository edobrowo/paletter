use crate::color::Rgb24;
use std::ops;

/// Handle associated with a particular octant.
///
/// Follows the pattern:
///     0bLLLII...II
/// That is, the first 3 bits are the octree level, and the latter
/// 29 bits are the index within that vector.
type Handle = u32;

mod handle {
    use super::Handle;

    /// Creates a new handle.
    pub fn new(index: usize, level: usize) -> Handle {
        let level = (level as u32 & 0b111) << 29;
        level | index as u32
    }

    /// Retrieves the level bits.
    pub fn level(handle: Handle) -> usize {
        ((0b111 << 29) & handle) as usize
    }

    /// Extracts the index bits.
    pub fn index(handle: Handle) -> usize {
        (!(0b111 << 29) & handle) as usize
    }
}

/// Octants are either branches or leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctantKind {
    Branch([Handle; 8]),
    Leaf,
}

/// Octant in the RGB octree.
#[derive(Debug, Clone)]
pub struct Octant {
    pub kind: OctantKind,
    pub count: u64,
    pub r: u64,
    pub g: u64,
    pub b: u64,
}

impl Octant {
    /// Creates a new branch octant.
    pub fn new_branch() -> Self {
        Self {
            kind: OctantKind::Branch([Octree::EMPTY; 8]),
            count: 0,
            r: 0,
            g: 0,
            b: 0,
        }
    }

    /// Creates a new leaf octant.
    pub fn new_leaf(count: u64, r: u64, g: u64, b: u64) -> Self {
        Self {
            kind: OctantKind::Leaf,
            count,
            r,
            g,
            b,
        }
    }

    /// Retrieves a child octant.
    pub fn child(&self, index: usize) -> Option<Handle> {
        match self.kind {
            OctantKind::Branch(children) => Some(children[index]),
            OctantKind::Leaf => None,
        }
    }

    /// Sets a child octant. Does nothing if the specified octant is a leaf.
    pub fn set_child(&mut self, index: usize, handle: Handle) {
        match &mut self.kind {
            OctantKind::Branch(children) => children[index] = handle,
            OctantKind::Leaf => (),
        }
    }

    /// Retrieves the number of child octants.
    pub fn child_count(&self) -> usize {
        match self.kind {
            OctantKind::Branch(children) => children.len(),
            OctantKind::Leaf => 0,
        }
    }

    /// Consumes the octant to produce an averaged color.
    pub fn into_rgb24(&self) -> Rgb24 {
        Rgb24::new(
            (self.r / self.count) as u8,
            (self.g / self.count) as u8,
            (self.b / self.count) as u8,
        )
    }
}

/// RGB-indexed octree.
#[derive(Debug)]
pub struct Octree {
    octants: [Vec<Octant>; Octree::MAX_HEIGHT],
}

impl Octree {
    const MAX_HEIGHT: usize = 8;

    const HANDLE_ROOT: Handle = 0;
    const EMPTY: Handle = u32::MAX;

    /// Creates a new RGB octree.
    pub fn new() -> Self {
        Self {
            octants: [
                vec![Octant::new_branch()],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }

    /// Total number of branch and leaf octants.
    pub fn len(&self) -> usize {
        self.octants.iter().fold(0, |acc, v| acc + v.len())
    }

    /// Whether the octree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self::new()
    }
}

impl ops::Index<Handle> for Octree {
    type Output = Octant;

    fn index(&self, handle: Handle) -> &Self::Output {
        &self.octants[handle::level(handle)][handle::index(handle)]
    }
}

impl ops::IndexMut<Handle> for Octree {
    fn index_mut(&mut self, handle: Handle) -> &mut Self::Output {
        &mut self.octants[handle::level(handle)][handle::index(handle)]
    }
}

impl Octree {
    /// Adds a color via index traversal.
    pub fn add_color(&mut self, color: &Rgb24) {
        let mut handle = Self::HANDLE_ROOT;

        for level in 0..Self::MAX_HEIGHT - 1 {
            let index = color.level_index(level);

            if self[handle].child(index).is_some_and(|c| c == Self::EMPTY) {
                self.octants[handle::level(handle)].push(Octant::new_branch());
                let new_handle = handle::new(level, self.octants[level].len());
                self[handle].set_child(index, new_handle);
            }

            if let Some(child) = self[handle].child(index) {
                handle = child;
            }
        }

        let octant = &mut self[handle];
        octant.count += 1;
        octant.r += color.r() as u64;
        octant.g += color.g() as u64;
        octant.b += color.b() as u64;
    }

    /// Builds the octree from a list of colors.
    pub fn build(&mut self, colors: &[Rgb24]) {
        colors.iter().for_each(|color| self.add_color(color));
    }

    /// Reduces an octree to the desire number of leaf octants.
    ///
    /// If the reduction cannot be made exactly, the number of octants is
    /// maintained above the expected size.
    ///
    ///
    /// This method is disgusting and I must fix it
    pub fn into_palette(mut self, size: usize) -> Vec<Rgb24> {
        // All leaves are initially stored in the highest level.
        let mut leaf_count = self.octants[Self::MAX_HEIGHT - 1].len();

        for level in (0..Self::MAX_HEIGHT).rev() {
            for i in 0..self.octants[level].len() {
                let child_count = self.octants[level][i].child_count();
                if leaf_count - child_count < size {
                    return self.octants[level]
                        .iter()
                        .map(|octant| octant.into_rgb24())
                        .collect();
                }

                self.octants[level][i] = match self.octants[level][i].kind {
                    OctantKind::Branch(children) => {
                        let (r, g, b, count) = children.iter().fold((0, 0, 0, 0), |acc, &h| {
                            (
                                acc.0 + self[h].r,
                                acc.1 + self[h].g,
                                acc.2 + self[h].b,
                                acc.3 + self[h].count,
                            )
                        });
                        Octant::new_leaf(count, r, g, b)
                    }
                    OctantKind::Leaf => unreachable!(),
                };

                leaf_count -= child_count;
            }
        }

        vec![]
    }
}

/// Finds a color palette using an RGB octree.
pub fn octree(colors: &[Rgb24], palette_size: usize) -> Vec<Rgb24> {
    if palette_size >= colors.len() {
        return colors.to_vec();
    }

    let mut octree = Octree::new();
    octree.build(colors);

    octree.into_palette(palette_size)
}
