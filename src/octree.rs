use crate::color::Rgb24;
use std::ops;

/// Handle associated with a particular octant.
///
/// Bit pattern:
///     0bLLLII...II
/// - L: octree level
/// - I: level index
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle(u32);

impl Handle {
    /// Creates a new handle.
    pub const fn new(level: usize, index: usize) -> Handle {
        assert!(level < 8);
        assert!(index < 2 << 29);
        let level = (level as u32) << 29;
        let index = index as u32;
        Handle { 0: level | index }
    }

    /// Extracts the level bits.
    pub fn level(&self) -> usize {
        (((0b111 << 29) & self.0) >> 29) as usize
    }

    /// Extracts the index bits.
    pub fn index(&self) -> usize {
        (!(0b111 << 29) & self.0) as usize
    }
}

/// Branch octants hold handles to 8 child octants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Branch {
    pub children: [Handle; 8],
}

/// Leaf octants hold summed RGB values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leaf {
    pub count: u64,
    pub r: u64,
    pub g: u64,
    pub b: u64,
}

/// RGB octant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Octant {
    Branch(Branch),
    Leaf(Leaf),
}

impl Octant {
    /// Creates a new branch octant.
    pub fn new_branch() -> Self {
        Self::Branch(Branch {
            children: [Octree::EMPTY; 8],
        })
    }

    /// Creates a new leaf octant.
    pub fn new_leaf(color: &Rgb24) -> Self {
        Self::Leaf(Leaf {
            count: 1,
            r: color.r() as u64,
            g: color.g() as u64,
            b: color.b() as u64,
        })
    }

    /// Creates leaf octant from summed RGB values.
    pub fn new_reduced(count: u64, r: u64, g: u64, b: u64) -> Self {
        Self::Leaf(Leaf { count, r, g, b })
    }

    /// Retrieves a child octant.
    pub fn child(&self, index: usize) -> Option<Handle> {
        match self {
            Octant::Branch(Branch { children }) => Some(children[index]),
            Octant::Leaf(_) => None,
        }
    }

    /// Sets a child octant. Does nothing if the specified octant is a leaf.
    pub fn set_child(&mut self, index: usize, handle: Handle) {
        match self {
            Octant::Branch(Branch { children }) => children[index] = handle,
            Octant::Leaf(_) => (),
        }
    }

    /// Retrieves the number of child octants.
    pub fn child_count(&self) -> usize {
        match self {
            Octant::Branch(Branch { children }) => children.len(),
            Octant::Leaf(_) => 0,
        }
    }

    /// Adds a color into the octant.
    pub fn add_color(&mut self, color: &Rgb24) {
        match self {
            Octant::Branch(_) => (),
            Octant::Leaf(leaf) => {
                leaf.count += 1;
                leaf.r += color.r() as u64;
                leaf.g += color.g() as u64;
                leaf.b += color.b() as u64;
            }
        }
    }

    /// Consumes the octant and returns the averaged RGB value.
    pub fn into_rgb24(self) -> Rgb24 {
        match self {
            Octant::Branch(_) => Rgb24::new(0, 0, 0),
            Octant::Leaf(Leaf { count, r, g, b }) => {
                Rgb24::new((r / count) as u8, (g / count) as u8, (b / count) as u8)
            }
        }
    }
}

/// RGB-indexed octree.
#[derive(Debug)]
pub struct Octree {
    octants: [Vec<Octant>; Octree::MAX_BRANCH_HEIGHT],
}

impl Octree {
    const MAX_BRANCH_HEIGHT: usize = 8;

    const HANDLE_ROOT: Handle = Handle::new(0, 0);
    const EMPTY: Handle = Handle::new(Octree::MAX_BRANCH_HEIGHT - 1, 2 << 29 - 1);

    /// Creates a new RGB octree.
    pub fn new() -> Self {
        let mut octants: [Vec<Octant>; Octree::MAX_BRANCH_HEIGHT] = Default::default();
        octants[0].push(Octant::new_branch());
        Self {
            octants: Default::default(),
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
        &self.octants[handle.level()][handle.index()]
    }
}

impl ops::IndexMut<Handle> for Octree {
    fn index_mut(&mut self, handle: Handle) -> &mut Self::Output {
        &mut self.octants[handle.level()][handle.index()]
    }
}

impl Octree {
    /// Adds a color via index traversal.
    pub fn add_color(&mut self, color: &Rgb24) {
        let mut handle = Self::HANDLE_ROOT;

        for level in 0..Self::MAX_BRANCH_HEIGHT - 1 {
            let index = color.level_index(level);

            if self[handle].child(index).is_some_and(|c| c == Self::EMPTY) {
                self.octants[handle.level()].push(Octant::new_branch());
                let child_handle = Handle::new(level, self.octants[level].len());
                self[handle].set_child(index, child_handle);
            }

            if let Some(child) = self[handle].child(index) {
                handle = child;
            }
        }

        let level = Self::MAX_BRANCH_HEIGHT - 1;
        let index = color.level_index(level);

        if self[handle].child(index).is_some_and(|c| c == Self::EMPTY) {
            self.octants[handle.level()].push(Octant::new_leaf(color));
            let child_handle = Handle::new(level, self.octants[level].len());
            self[handle].set_child(index, child_handle);
        } else {
            let child = self[handle].child(index).unwrap();
            self[child].add_color(color);
        }
    }

    /// Builds the octree from a list of colors.
    pub fn build(&mut self, colors: &[Rgb24]) {
        colors.iter().for_each(|color| self.add_color(color));
    }

    /// Reduces an octree to the specified number of leaf octants.
    ///
    /// If the reduction cannot be made exactly, the number of octants is
    /// maintained above the expected size.
    ///
    pub fn into_palette(mut self, size: usize) -> Vec<Rgb24> {
        // All leaves are initially stored in the highest level.
        let mut leaf_count = self.octants[Self::MAX_BRANCH_HEIGHT - 1].len();

        for level in (0..Self::MAX_BRANCH_HEIGHT).rev() {
            for i in 0..self.octants[level].len() {
                let child_count = self.octants[level][i].child_count();
                if leaf_count - child_count < size {
                    return self.octants[level]
                        .iter()
                        .map(|octant| octant.into_rgb24())
                        .collect();
                }

                self.octants[level][i] = match self.octants[level][i] {
                    Octant::Branch(Branch { children }) => {
                        let (count, r, g, b) = children.iter().fold((0, 0, 0, 0), |acc, &h| {
                            let (count, r, g, b) =
                                if let Octant::Leaf(Leaf { count, r, g, b }) = self[h] {
                                    (count, r, g, b)
                                } else {
                                    (0, 0, 0, 0)
                                };
                            (acc.0 + count, acc.1 + r, acc.2 + g, acc.3 + b)
                        });
                        Octant::new_reduced(count, r as u64, g as u64, b as u64)
                    }
                    Octant::Leaf(_) => unreachable!(),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handles() {
        let handle = Handle::new(0, 0);
        assert_eq!(handle.level(), 0);
        assert_eq!(handle.index(), 0);

        let handle = Handle::new(4, 16279);
        assert_eq!(handle.level(), 4);
        assert_eq!(handle.index(), 16279);

        let handle = Handle::new(7, 536870911);
        assert_eq!(handle.level(), 7);
        assert_eq!(handle.index(), 536870911);
    }
}
