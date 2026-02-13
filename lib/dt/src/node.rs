use crate::prop::{Property, PropertyError};
use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, vec, vec::Vec};
use core::ops::Range;
use utils::endian::{BigEndian32, EndianData};
pub struct DeviceTree {
    pub root_id: usize,
    pub container: Vec<Node>,
    pub mem_rsv_map: Vec<Range<usize>>,
    pub phandle_map: BTreeMap<usize, Box<str>>,
}

pub struct Node {
    pub node_id: usize,
    pub parent_id: usize,
    pub full_name: Box<str>,
    pub node_name: Box<str>,
    pub unit_addr: Box<str>,
    pub children: Vec<usize>,
    pub props: Vec<Property>,
    pub node_type: NodeType,
}

#[derive(PartialEq, Eq, Debug)]
pub enum NodeType {
    Device,
    Description,
}

impl DeviceTree {
    pub fn is_root(&self, node: &Node) -> bool {
        self.get_parent(node).node_id == node.node_id
    }
    fn full_path(&self, node: &Node) -> String {
        if self.is_root(node) {
            return String::from("");
        } else {
            return self.full_path(self.get_parent(node)) + "/" + node.full_name.as_ref();
        }
    }
    pub fn get_full_path(&self, node: &Node) -> Box<str> {
        self.full_path(node).into_boxed_str()
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
                if subnode.full_name.as_ref().eq(section) {
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
    pub fn get_node_mut<'b>(&'b mut self, path: impl AsRef<str>) -> Option<&'b mut Node> {
        let path_str = path.as_ref();
        let mut node = &mut self.container[self.root_id] as *mut Node;
        for section in path_str.split('/') {
            if section.trim().is_empty() {
                continue;
            }
            let mut found = false;
            for subnode_id in &unsafe { &*node }.children {
                let subnode = &self.container[*subnode_id];
                if subnode.full_name.as_ref().eq(section) {
                    node = &mut self.container[*subnode_id];
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }
        Some(unsafe { node.as_mut() }.unwrap())
    }
    pub fn get_nodes(&self, path: impl AsRef<str>) -> Vec<&Node> {
        let path_str = path.as_ref();
        let root = &self.container[self.root_id];
        let mut path = path_str.split('/').collect();
        self.get_sub_nodes(root, &mut path, 0)
    }
    pub fn get_nodes_mut<F: Fn(&mut Node) -> ()>(&mut self, path: impl AsRef<str>, f: F) {
        let paths: Vec<Box<str>> = self
            .get_nodes(path)
            .iter()
            .map(|x| self.get_full_path(x))
            .collect();
        for path in paths {
            if let Some(node) = self.get_node_mut(path) {
                f(node);
            }
        }
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
    fn get_intr_value(
        &self,
        intc: &Node,
        data: &[BigEndian32],
    ) -> Result<(usize, usize), PropertyError> {
        let mut intr_cells = 1;
        if let Some(prop) = self.get_property(intc, "#interrupt-cells") {
            intr_cells = prop.value_as_u32()? as usize;
        }
        let mut val: usize = 0;
        for idx in 0..intr_cells {
            val = (val << 32) + data[idx].value() as usize;
        }
        Ok((intr_cells, val))
    }
    pub fn get_intr_info<'a>(
        &'a self,
        node: &Node,
    ) -> Result<Vec<(usize, usize)>, PropertyError> {
        let mut res = vec![];
        if let Some(prop) = self.get_property(node, "interrupts-extended") {
            let data = prop.value_as_proplist::<BigEndian32>()?;
            let mut index = 0;
            while index < data.len() {
                let phandle = data[index].value() as usize;
                let path = self
                    .phandle_map
                    .get(&phandle)
                    .ok_or(PropertyError::DanglingHandle)?;
                let node = self.get_node(path).ok_or(PropertyError::DanglingHandle)?;
                let (value, step) = self.get_intr_value(node, &data[(index + 1)..data.len()])?;
                index += step + 1;
                res.push((phandle, value));
            }
        } else if let Some(parent) = self.get_property(node, "interrupt-parent")
            && let Some(intrs) = self.get_property(node, "interrupts")
        {
            let phandle = parent.value_as_u32()? as usize;
            let path = self
                .phandle_map
                .get(&phandle)
                .ok_or(PropertyError::DanglingHandle)?;
            let node = self.get_node(path).ok_or(PropertyError::DanglingHandle)?;
            let data = intrs.value_as_proplist::<BigEndian32>()?;
            let mut index = 0;
            while index < data.len() {
                let (value, step) = self.get_intr_value(node, &data[(index)..data.len()])?;
                index += step;
                res.push((phandle, value));
            }
        }
        Ok(res)
    }
}
