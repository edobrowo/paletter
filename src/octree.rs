use crate::color::Rgb24;

/// Handle associated with a particular octant.
type Handle = usize;

/// Index to a child of an octant.
type Index = usize;

/// Indicates number of valid octant children.
type Size = Index;

/// Branch octants hold handles to 8 child octants.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Branch {
    pub children: [Handle; 8],
}

/// Leaf octants hold summed RGB values.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Leaf {
    pub count: u64,
    pub r: u64,
    pub g: u64,
    pub b: u64,
}

/// RGB octant.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Octant {
    Branch(Branch),
    Leaf(Leaf),
}

impl From<&Rgb24> for Octant {
    fn from(color: &Rgb24) -> Self {
        Self::Leaf(Leaf {
            count: 1,
            r: color.r() as u64,
            g: color.g() as u64,
            b: color.b() as u64,
        })
    }
}

impl Octant {
    /// Minimum child index.
    pub const MIN_CHILD: Index = 0;

    /// Maximum child index.
    pub const MAX_CHILD: Index = 7;

    /// Maximum valid child count.
    pub const MAX_SIZE: Size = 8;

    /// Creates a new branch octant.
    pub fn new_branch() -> Self {
        Self::Branch(Branch {
            children: [Octree::EMPTY; 8],
        })
    }

    /// Creates leaf octant from summed RGB values.
    pub fn new_leaf(count: u64, r: u64, g: u64, b: u64) -> Self {
        Self::Leaf(Leaf { count, r, g, b })
    }

    /// Retrieves a child octant.
    pub fn child(&self, index: Index) -> Option<Handle> {
        match self {
            Octant::Branch(Branch { children }) => Some(children[index]),
            Octant::Leaf(_) => None,
        }
    }

    /// Checks whether a child octant at a particular index exists.
    pub fn child_exists(&self, index: Index) -> bool {
        match self {
            Octant::Branch(Branch { children }) => children[index] != Octree::EMPTY,
            Octant::Leaf(_) => false,
        }
    }

    /// Sets a child octant. Does nothing if the specified octant is a leaf.
    pub fn set_child(&mut self, index: Index, handle: Handle) {
        match self {
            Octant::Branch(Branch { children }) => children[index] = handle,
            Octant::Leaf(_) => (),
        }
    }

    /// Retrieves the number of child octants.
    pub fn child_count(&self) -> Size {
        match self {
            Octant::Branch(Branch { children }) => {
                children.iter().filter(|&&c| c != Octree::EMPTY).count()
            }
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
    pub fn make_rgb24(&self) -> Option<Rgb24> {
        match self {
            Octant::Branch(_) => None,
            Octant::Leaf(Leaf { count, r, g, b }) => {
                if *count > 0 {
                    Some(Rgb24::new(
                        (r / count) as u8,
                        (g / count) as u8,
                        (b / count) as u8,
                    ))
                } else {
                    None
                }
            }
        }
    }
}

/// RGB-indexed octree.
#[derive(Debug)]
pub struct Octree {
    octants: Vec<Octant>,
    levels: [Vec<Handle>; 8],
}

impl Octree {
    /// Minimum octant level in an RGB octree.
    const MIN_LEVEL: usize = 0;

    /// Maximum octant level in an RGB octree.
    const MAX_LEVEL: usize = 8;

    /// Reserved handle. Used to reference the root octant.
    const ROOT: Handle = 0;

    /// Reserved handle. Used to reference empty children in a branch octant.
    const EMPTY: Handle = usize::MAX;

    /// Creates a new RGB octree.
    pub fn new() -> Self {
        Self {
            octants: vec![Octant::new_branch()],
            levels: Default::default(),
        }
    }

    /// Total number of branch and leaf octants.
    pub fn len(&self) -> usize {
        self.octants.len()
    }

    /// Whether the octree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Builds the octree from a list of colors.
    pub fn build(&mut self, colors: &[Rgb24]) {
        colors.iter().for_each(|color| self.add_color(color));
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self::new()
    }
}

impl Octree {
    /// Add a new branch to a octant.
    pub fn add_branch(&mut self, handle: Handle, index: Index, level: Index) {
        let branch_handle = self.make_handle();
        self.octants.push(Octant::new_branch());
        self.octants[handle].set_child(index, branch_handle);
        self.levels[level].push(branch_handle)
    }

    /// Add a new leaf to an octant.
    pub fn add_leaf(&mut self, handle: Handle, index: Index, level: Index, color: &Rgb24) {
        let leaf_handle = self.make_handle();
        self.octants.push(Octant::from(color));
        self.octants[handle].set_child(index, leaf_handle);
        self.levels[level].push(leaf_handle)
    }

    /// Create a fresh handle.
    fn make_handle(&self) -> Handle {
        self.len()
    }
}

impl Octree {
    /// Adds a color via index traversal.
    pub fn add_color(&mut self, color: &Rgb24) {
        let mut handle = Self::ROOT;

        for level in Self::MIN_LEVEL..Self::MAX_LEVEL - 1 {
            let index = color.level_index(level);

            if !self.octants[handle].child_exists(index) {
                self.add_branch(handle, index, level);
            }

            handle = self.octants[handle].child(index).unwrap();
        }

        let index = color.level_index(Self::MAX_LEVEL - 1);
        if !self.octants[handle].child_exists(index) {
            self.add_leaf(handle, index, Self::MAX_LEVEL - 1, color);
        } else {
            let child_handle = self.octants[handle].child(index).unwrap();
            self.octants[child_handle].add_color(color);
        }
    }

    /// Reduces an octree to the specified number of leaf octants.
    ///
    /// If the reduction cannot be made exactly, the number of octants is
    /// maintained above the expected size.
    ///
    pub fn into_palette(&mut self, size: usize) -> Vec<Rgb24> {
        // All leaves are initially stored at the highest level.
        let mut leaf_count = self.levels[Self::MAX_LEVEL - 1].len();

        for &handle in self.levels.iter().rev().skip(1).flatten() {
            let count = self.octants[handle].child_count();

            if leaf_count - count < size {
                break;
            }

            match &self.octants[handle] {
                Octant::Branch(Branch { children }) => {
                    // Sum the child colors.
                    let (count, r, g, b) = children.iter().filter(|&&h| h != Octree::EMPTY).fold(
                        (0, 0, 0, 0),
                        |acc, &h| {
                            if let Octant::Leaf(Leaf { count, r, g, b }) = self.octants[h] {
                                (acc.0 + count, acc.1 + r, acc.2 + g, acc.3 + b)
                            } else {
                                acc
                            }
                        },
                    );

                    // Clear the child octants.
                    for &h in children.clone().iter().filter(|&&h| h != Octree::EMPTY) {
                        self.octants[h] = Octant::new_leaf(0, 0, 0, 0);
                    }

                    // Replace the branch with a leaf.
                    self.octants[handle] = Octant::new_leaf(count, r, g, b);
                }
                Octant::Leaf(_) => unreachable!(),
            }

            leaf_count -= count;
        }

        self.octants
            .iter()
            .filter_map(|octant| octant.make_rgb24())
            .collect()
    }
}

/// Finds a color palette using an RGB octree.
pub fn octree(colors: &[Rgb24], palette_size: usize) -> Vec<Rgb24> {
    let mut octree = Octree::new();
    octree.build(colors);
    octree.into_palette(palette_size)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn octree_solve() {
        let data = vec![
            Rgb24::new(0, 0, 0),
            Rgb24::new(50, 0, 0),
            Rgb24::new(0, 50, 0),
            Rgb24::new(0, 0, 50),
            Rgb24::new(150, 0, 0),
            Rgb24::new(0, 150, 0),
            Rgb24::new(0, 0, 150),
        ];

        octree(&data, 1);
    }
}
