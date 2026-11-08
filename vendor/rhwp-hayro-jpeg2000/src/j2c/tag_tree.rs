//! The tag tree, described in Section B.10.2.
//!
//! Tag trees are quad trees where each leaf stores an integer value.
//! Each intermediate node stores the smallest value of all of its children.
//! For example, if a node stores the value 3, it means that all children
//! have a value of 3 or higher. The root node therefore stores the smallest
//! values across all children.

use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

use crate::reader::BitReader;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(crate) struct TagNode {
    /// The width of the area covered by the node.
    ///
    /// For leaf nodes, this value is always 1. In some cases, the width might
    /// be 0, in which case the leaf node doesn't actually "exist" and is just
    /// a dummy node.
    width: u32,
    /// The height of the area covered by the node.
    ///
    /// For leaf nodes, this value is always 1. In some cases, the height might
    /// be 0, in which case the leaf node doesn't actually "exist" and is just
    /// a dummy node.
    height: u32,
    /// The actual value stored in the node. Only valid once `initialized`
    /// is set to `true`.
    value: u32,
    /// Whether the node has been fully initialized. The tag tree is not
    /// stored in its complete form in the JP2 file, but is instead built
    /// up incrementally, each packet contributing the information of the
    /// tag tree. The node is therefore only initialized with its actual
    /// value once we cross it the first time.
    initialized: bool,
    /// The level inside the tree. Zero indicates that the given node is
    /// a leaf node, otherwise the level is > 0. The root node has the highest
    /// level.
    level: u16,
    /// The indices of the children of the node, some of which might be dummy
    /// nodes (indicated by the fact that the index is `usize::MAX`).
    children: [usize; 4],
}

impl TagNode {
    fn new(width: u32, height: u32, level: u16) -> Self {
        Self {
            width,
            height,
            level,
            value: 0,
            initialized: false,
            children: [usize::MAX, usize::MAX, usize::MAX, usize::MAX],
        }
    }

    /// The width of the top-left child.
    fn top_left_width(&self) -> u32 {
        u32::min(1 << (self.level - 1), self.width)
    }

    /// The height of the top-left child.
    fn top_left_height(&self) -> u32 {
        u32::min(1 << (self.level - 1), self.height)
    }
}

impl TagNode {
    fn build(
        width: u32,
        height: u32,
        level: u16,
        nodes: &mut Vec<Self>,
    ) -> crate::error::Result<Self> {
        rhwp_codec_resource::work(1)?;
        let mut tag = Self::new(width, height, level);

        if level == 0 || width == 0 || height == 0 {
            // We reached the leaf node.
            debug_assert!(level != 0 || (width <= 1 && height <= 1));

            return Ok(tag);
        }

        // Determine the width and height of the top-left child node. Based
        // on this, we can infer the dimensions of all other child nodes.
        let top_left_width = tag.top_left_width();
        let top_left_height = tag.top_left_height();

        let mut push =
            |node: Self, child_idx: usize, nodes: &mut Vec<Self>| -> crate::error::Result<()> {
                // If this is not the case, the child doesn't actually exist.
                if node.width > 0 && node.height > 0 {
                    let node_idx = nodes.len();
                    nodes.rhwp_push(node)?;
                    tag.children[child_idx] = node_idx;
                }
                Ok(())
            };

        // We always push four children, but some nodes might in reality have
        // fewer than that. In this case, the resulting node will simply have
        // a width or height of 0 and we can recognize that it technically
        // doesn't exist.
        let n1 = Self::build(top_left_width, top_left_height, level - 1, nodes)?;
        push(n1, 0, nodes)?;

        let n2 = Self::build(width - top_left_width, top_left_height, level - 1, nodes)?;
        push(n2, 1, nodes)?;

        let n3 = Self::build(top_left_width, height - top_left_height, level - 1, nodes)?;
        push(n3, 2, nodes)?;

        let n4 = Self::build(
            width - top_left_width,
            height - top_left_height,
            level - 1,
            nodes,
        )?;
        push(n4, 3, nodes)?;

        Ok(tag)
    }
}

fn read_tag_node(
    node_idx: usize,
    x: u32,
    y: u32,
    reader: &mut BitReader<'_>,
    parent_val: u32,
    max_val: u32,
    nodes: &mut [TagNode],
) -> Option<u32> {
    rhwp_codec_resource::work(1).ok()?;
    let node = &mut nodes[node_idx];

    if !node.initialized {
        let mut val = u32::max(parent_val, node.value);

        loop {
            rhwp_codec_resource::work(1).ok()?;
            if val >= max_val {
                break;
            }

            // "Each node has an associated current value, which is
            // initialized to zero (the minimum). A 0 bit in the tag tree
            // means that the minimum (or the value in the case of the
            // highest level) is larger than the current value and a 1 bit
            // means that the minimum (or the value in the case of the
            // highest level) is equal to the current value."
            match reader.read_bits_with_stuffing(1)? {
                0 => val += 1,
                1 => {
                    node.initialized = true;
                    break;
                }
                _ => unreachable!(),
            }
        }

        node.value = val;
    }

    // Abort early if we already reached the leaf node or the minimum
    // value of all children is too large.
    if node.value >= max_val || node.level == 0 {
        return Some(node.value);
    }

    let top_left_width = node.top_left_width();
    let top_left_height = node.top_left_height();

    let left = x < top_left_width;
    let top = y < top_left_height;

    match (left, top) {
        (true, true) => read_tag_node(node.children[0], x, y, reader, node.value, max_val, nodes),
        (false, true) => read_tag_node(
            node.children[1],
            x - top_left_width,
            y,
            reader,
            node.value,
            max_val,
            nodes,
        ),
        (true, false) => read_tag_node(
            node.children[2],
            x,
            y - top_left_height,
            reader,
            node.value,
            max_val,
            nodes,
        ),
        (false, false) => read_tag_node(
            node.children[3],
            x - top_left_width,
            y - top_left_height,
            reader,
            node.value,
            max_val,
            nodes,
        ),
    }
}

#[derive(Copy, Clone)]
pub(crate) struct TagTree {
    root: usize,
    width: u32,
    height: u32,
}

impl TagTree {
    pub(crate) fn new(
        width: u32,
        height: u32,
        nodes: &mut Vec<TagNode>,
    ) -> crate::error::Result<Self> {
        // Calculate how many levels the tree has in total.
        let level = u32::max(
            width
                .checked_next_power_of_two()
                .ok_or_else(rhwp_codec_resource::reject)?
                .ilog2(),
            height
                .checked_next_power_of_two()
                .ok_or_else(rhwp_codec_resource::reject)?
                .ilog2(),
        );

        let node = TagNode::build(width, height, level as u16, nodes)?;
        let idx = nodes.len();
        nodes.rhwp_push(node)?;

        Ok(Self {
            root: idx,
            width,
            height,
        })
    }

    pub(crate) fn read(
        &mut self,
        x: u32,
        y: u32,
        reader: &mut BitReader<'_>,
        max_val: u32,
        nodes: &mut [TagNode],
    ) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        debug_assert!(x < self.width && y < self.height);

        read_tag_node(self.root, x, y, reader, 0, max_val, nodes)
    }
}
