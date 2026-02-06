use core::ops::Range;

use crate::prop::{Property, PropertyError};
use alloc::{boxed::Box, vec, vec::Vec};
use utils::endian::{BigEndian32, EndianData};
pub struct DeviceTree {
    pub root_id: usize,
    pub container: Vec<Node>,
    pub mem_rsv_map: Vec<Range<usize>>,
}

pub struct Node {
    pub node_id: usize,
    pub parent_id: usize,
    pub full_name: Box<str>,
    pub node_name: Box<str>,
    pub unit_addr: Box<str>,
    pub children: Vec<usize>,
    pub props: Vec<Property>,
}

impl DeviceTree {
    pub fn is_root(&self, node: &Node) -> bool {
        self.get_parent(node).node_id == node.node_id
    }
    pub fn get_parent(&self, node: &Node) -> &Node {
        &self.container[node.parent_id]
    }
    pub fn get_children<'b>(&'b self, node: &Node) -> impl Iterator<Item = &'b Node> {
        node.children.iter().map(|x| &self.container[*x])
    }
    pub fn get_property<'b>(&self, node: &'b Node, name: impl AsRef<str>) -> Option<&'b Property> {
        let name = name.as_ref();
        for prop in &node.props {
            if prop.name.as_ref().eq(name) {
                return Some(prop);
            }
        }
        None
    }
    pub fn get_node(&self, path: impl AsRef<str>) -> Option<&Node> {
        let path_str = path.as_ref();
        let mut node = &self.container[self.root_id];
        for section in path_str.split('/') {
            if section.trim().is_empty() {
                continue;
            }
            let mut found = false;
            for subnode in self.get_children(node) {
                if subnode.full_name.as_ref().eq(path_str) {
                    node = subnode;
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }
        Some(node)
    }
    pub fn get_nodes(&self, path: impl AsRef<str>) -> Vec<&Node> {
        let path_str = path.as_ref();
        let root = &self.container[self.root_id];
        let mut path = path_str.split('/').collect();
        self.get_sub_nodes(root, &mut path, 0)
    }
    fn get_sub_nodes<'a, 'b>(
        &'b self,
        node: &'b Node,
        path: &Vec<&str>,
        mut cursor: usize,
    ) -> Vec<&'b Node> {
        while cursor < path.len() && path[cursor].trim().is_empty() {
            cursor += 1;
        }
        if cursor >= path.len() {
            return vec![node];
        }
        let sec = path[cursor];
        self.get_children(node)
            .flat_map(|child| {
                if sec.eq("*")
                    || child.full_name.as_ref().eq(sec)
                    || child.node_name.as_ref().eq(sec)
                {
                    return self.get_sub_nodes(child, path, cursor + 1);
                } else {
                    return vec![];
                }
            })
            .collect()
    }
    pub fn get_reg_value(&self, node: &Node) -> Result<Vec<Range<usize>>, PropertyError> {
        let mut size_cel = 1;
        let mut addr_cel = 2;
        if !self.is_root(node) {
            let parent = self.get_parent(node);
            if let Some(prop) = self.get_property(parent, "#address-cells") {
                addr_cel = prop.value_as_u32()? as usize;
            }
            if let Some(prop) = self.get_property(parent, "#size-cells") {
                size_cel = prop.value_as_u32()? as usize;
            }
        }
        let reg = self
            .get_property(node, "reg")
            .ok_or(PropertyError::PropNotFound)?;
        let reg = reg.value_as_proplist::<BigEndian32>()?;
        let width = size_cel + addr_cel;
        let count = reg.len() / width;
        let mut res = vec![];
        for i in 0..count {
            let index = width * i;
            let mut addr = 0;
            let mut sz = 0;
            for j in index..index + addr_cel {
                addr = (addr << 32) + (reg[j].value() as usize);
            }
            for j in index + addr_cel..index + width {
                sz = (sz << 32) + (reg[j].value() as usize);
            }
            res.push(Range {
                start: addr,
                end: addr + sz,
            });
        }
        Ok(res)
    }
}
