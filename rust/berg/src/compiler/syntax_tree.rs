use compiler::internals::*;

pub struct SyntaxTree {
    nodes: Vec<SyntaxNodeData>;
}

pub struct SyntaxNodeData {
    token: TokenSpan,
    parent: Option<SyntaxTreeIndex>,
}

pub struct SyntaxTreeIndex(usize);

struct SyntaxNode {
    tree: &SyntaxTree,
    index: usize,
}

impl SyntaxTree {
    pub fn len() -> SyntaxTreeIndex { nodes.len }

    pub fn set_parent(&mut self, child: SyntaxTreeIndex, parent: SyntaxTreeIndex) {
        assert!(self.nodes[child].parent.is_none());
        self.nodes[child].parent = Some(parent);
    }
    pub fn append(&mut self, token: TokenSpan) {
        self.nodes.append(SyntaxNodeData { token });
    }
}

impl Index<SyntaxTree::SyntaxTreeIndex> for SyntaxTree {
    type Output = SyntaxNode;

    fn index(&self, index: SyntaxTreeIndex) -> SyntaxNode {
        SyntaxNode { tree: &self, index }
    }
}

impl SyntaxNode {
    fn data(&self) -> SyntaxNodeData {
        tree.nodes[index]
    }

    pub fn parent(&self) -> Option<SyntaxNode> {
        if data().parent.is_some
            SyntaxNode { tree: self.tree, index: data().parent }
        else
            None
    }
    pub fn left_child(&self) -> Option<SyntaxNode> {
        if data().left_child.is_some
            SyntaxNode { tree: self.tree, index: data().left_child }
        else
            None
    }
    pub fn right_child(&self) -> Option<SyntaxNode> {
        if data().right_child.is_some
            SyntaxNode { tree: self.tree, index: data().right_child }
        else
            None
    }
    pub fn next(&self) -> Option<SyntaxNode> {
        if (self.index+1) < tree.len()
            SyntaxNode { tree: self.tree, index: self.index+1 }
        else
            None
    }
    pub fn prev(&self) -> Option<SyntaxNode> {
        if (self.index-1) >= 0
            SyntaxNode { tree: self.tree, index: self.index-1 }
        else
            None
    }
}

impl Into<SyntaxTreeIndex> for SyntaxNode {
    fn into(self) -> SyntaxTreeIndex {
        self.index
    }
}
