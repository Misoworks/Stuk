pub use accesskit::{
    Action, Node, NodeId, Rect as AccessRect, Role, Toggled, Tree, TreeId, TreeUpdate,
};
use stuk_layout::Rect;

pub const ROOT_NODE_ID: NodeId = NodeId(1);

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityTree {
    root: NodeId,
    focus: NodeId,
    nodes: Vec<(NodeId, Node)>,
}

impl AccessibilityTree {
    pub fn new(root: NodeId, nodes: Vec<(NodeId, Node)>) -> Self {
        Self {
            root,
            focus: root,
            nodes,
        }
    }

    pub fn empty() -> Self {
        let root = ROOT_NODE_ID;
        let mut node = Node::new(Role::Window);
        node.set_label("Stuk");
        Self::new(root, vec![(root, node)])
    }

    pub fn from_root(root: AccessibilityNode) -> Self {
        let mut nodes = Vec::new();
        let root_id = push_accessible_node(&root, &[], &mut nodes);
        Self::new(root_id, nodes)
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn focus(&self) -> NodeId {
        self.focus
    }

    pub fn nodes(&self) -> &[(NodeId, Node)] {
        &self.nodes
    }

    pub fn update(&self) -> TreeUpdate {
        self.clone().into_update()
    }

    pub fn into_update(self) -> TreeUpdate {
        let mut tree = Tree::new(self.root);
        tree.toolkit_name = Some("Stuk".to_string());
        tree.toolkit_version = Some(env!("CARGO_PKG_VERSION").to_string());
        TreeUpdate {
            nodes: self.nodes,
            tree: Some(tree),
            tree_id: TreeId::ROOT,
            focus: self.focus,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityNode {
    role: Role,
    label: Option<String>,
    value: Option<String>,
    bounds: Option<Rect>,
    actions: Vec<Action>,
    toggled: Option<Toggled>,
    children: Vec<AccessibilityNode>,
}

impl AccessibilityNode {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            value: None,
            bounds: None,
            actions: Vec::new(),
            toggled: None,
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn toggled(mut self, toggled: impl Into<Toggled>) -> Self {
        self.toggled = Some(toggled.into());
        self
    }

    pub fn child(mut self, child: AccessibilityNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<AccessibilityNode>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Clone, Debug)]
pub struct AccessibilityTreeBuilder {
    next_id: u64,
    nodes: Vec<(NodeId, Node)>,
}

impl AccessibilityTreeBuilder {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            nodes: Vec::new(),
        }
    }

    pub fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push((id, node));
        id
    }

    pub fn finish(self, root: NodeId) -> AccessibilityTree {
        AccessibilityTree::new(root, self.nodes)
    }
}

impl Default for AccessibilityTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn push_accessible_node(
    source: &AccessibilityNode,
    path: &[usize],
    nodes: &mut Vec<(NodeId, Node)>,
) -> NodeId {
    let id = node_id_for_path(path);
    let mut node = Node::new(source.role);

    if let Some(label) = &source.label {
        node.set_label(label.as_str());
    }
    if let Some(value) = &source.value {
        node.set_value(value.as_str());
    }
    if let Some(bounds) = source.bounds {
        node.set_bounds(access_rect(bounds));
    }
    for action in &source.actions {
        node.add_action(*action);
    }
    if let Some(toggled) = source.toggled {
        node.set_toggled(toggled);
    }

    let mut child_ids = Vec::with_capacity(source.children.len());
    for (index, child) in source.children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(index);
        child_ids.push(push_accessible_node(child, &child_path, nodes));
    }
    node.set_children(child_ids);
    nodes.push((id, node));
    id
}

fn node_id_for_path(path: &[usize]) -> NodeId {
    if path.is_empty() {
        return ROOT_NODE_ID;
    }

    let mut hash = 0xcbf29ce484222325_u64;
    for index in path {
        for byte in (*index as u64).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    NodeId(hash.max(2))
}

fn access_rect(rect: Rect) -> AccessRect {
    AccessRect::new(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.x + rect.width),
        f64::from(rect.y + rect.height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_accesskit_tree_update() {
        let root = AccessibilityNode::new(Role::Window)
            .label("Window")
            .bounds(Rect::new(0.0, 0.0, 200.0, 120.0))
            .child(
                AccessibilityNode::new(Role::Button)
                    .label("Save")
                    .action(Action::Click),
            );

        let tree = AccessibilityTree::from_root(root);
        let update = tree.update();

        assert_eq!(update.focus, ROOT_NODE_ID);
        assert_eq!(update.tree.as_ref().unwrap().root, ROOT_NODE_ID);
        assert_eq!(update.nodes.len(), 2);
    }
}
