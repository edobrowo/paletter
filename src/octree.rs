use crate::color::Rgb24;
use std::ops;

pub type Handle = usize;

#[derive(Debug)]
pub struct Octree {
    nodes: Vec<Node>,
}

impl Octree {
    const MAX_HEIGHT: u8 = 8;
    const ROOT: Handle = 0;
    const EMPTY: Handle = usize::MAX;

    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    pub fn add_color(&mut self, color: Rgb24) {
        let mut handle = Self::ROOT;

        for level in 0..Self::MAX_HEIGHT {
            let index = color.level_index(level as usize);

            if self[handle][index] == Self::EMPTY {
                self.nodes.push(Node::new());
                let new_handle = self.nodes.len();
                self[handle][index] = new_handle;
            }

            handle = self[handle][index];
        }

        let node = &mut self.nodes[handle];
        node.count += 1;
        node.r += color.r() as u64;
        node.g += color.g() as u64;
        node.b += color.b() as u64;
    }

    pub fn reduce(&mut self, handle: Handle) {
        let (r, g, b, ref_count) =
            self.nodes[handle]
                .children
                .iter()
                .fold((0, 0, 0, 0), |acc, &h| {
                    (
                        acc.0 + self[h].r,
                        acc.1 + self[h].g,
                        acc.2 + self[h].b,
                        acc.3 + self[h].count,
                    )
                });

        self.nodes[handle].r = r;
        self.nodes[handle].g = g;
        self.nodes[handle].b = b;
        self.nodes[handle].count = ref_count;
        self.nodes[handle].children = [Self::EMPTY; 8];
    }
}

impl ops::Index<Handle> for Octree {
    type Output = Node;

    fn index(&self, index: Handle) -> &Self::Output {
        &self.nodes[index]
    }
}

impl ops::IndexMut<Handle> for Octree {
    fn index_mut(&mut self, index: Handle) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub count: u64,
    pub r: u64,
    pub g: u64,
    pub b: u64,
    children: [Handle; 8],
}

impl Node {
    pub fn new() -> Self {
        Self {
            count: 0,
            r: 0,
            g: 0,
            b: 0,
            children: [Octree::EMPTY; 8],
        }
    }
}

impl ops::Index<Handle> for Node {
    type Output = Handle;

    fn index(&self, index: Handle) -> &Self::Output {
        &self.children[index]
    }
}

impl ops::IndexMut<Handle> for Node {
    fn index_mut(&mut self, index: Handle) -> &mut Self::Output {
        &mut self.children[index]
    }
}
