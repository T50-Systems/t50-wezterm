impl<L, N> Cursor<L, N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            it: Box::new(Tree::Empty),
            path: Box::new(Path::Top),
        }
    }

    /// References the subtree at the current cursor position
    pub fn subtree(&self) -> &Tree<L, N> {
        &*self.it
    }

    /// Returns true if the current position is a leaf node
    pub fn is_leaf(&self) -> bool {
        matches!(&*self.it, Tree::Leaf(_))
    }

    /// Returns true if the current position is the left child of its parent
    pub fn is_left(&self) -> bool {
        matches!(&*self.path, Path::Left { .. })
    }

    /// Returns true if the current position is the right child of its parent
    pub fn is_right(&self) -> bool {
        matches!(&*self.path, Path::Right { .. })
    }

    pub fn is_top(&self) -> bool {
        matches!(&*self.path, Path::Top)
    }

    /// If the current position is the root of the empty tree,
    /// assign an initial leaf value.
    /// Consumes the cursor and returns a new cursor representing
    /// the mutated tree.
    /// If the current position isn't the top of the empty tree,
    /// yields `Err` containing the unchanged cursor.
    pub fn assign_top(self, leaf: L) -> Result<Self, Self> {
        match (&*self.it, &*self.path) {
            (Tree::Empty, Path::Top) => Ok(Self {
                it: Box::new(Tree::Leaf(leaf)),
                path: self.path,
            }),
            _ => Err(self),
        }
    }

    /// If the current position is a leaf node, return a mutable
    /// reference to the leaf data, else `None`.
    pub fn leaf_mut(&mut self) -> Option<&mut L> {
        match &mut *self.it {
            Tree::Leaf(l) => Some(l),
            _ => None,
        }
    }

    /// If the current position is not a leaf node, return a mutable
    /// reference to the node data container, else yields `Err`.
    #[allow(clippy::result_unit_err)]
    pub fn node_mut(&mut self) -> Result<&mut Option<N>, ()> {
        match &mut *self.it {
            Tree::Node { data, .. } => Ok(data),
            _ => Err(()),
        }
    }

    /// Return an iterator that will visit the chain of nodes leading
    /// to the root from the current position and yield their node
    /// data at each step of iteration.
    pub fn path_to_root(&self) -> ParentIterator<'_, L, N> {
        ParentIterator { path: &*self.path }
    }

    /// If the current position is not a leaf node, assign the
    /// node data to the supplied value.
    /// Consumes the cursor and returns a new cursor representing the
    /// mutated tree.
    /// If the current position is a leaf node then yields `Err`
    /// containing the unchanged cursor.
    pub fn assign_node(mut self, value: Option<N>) -> Result<Self, Self> {
        match &mut *self.it {
            Tree::Node { data, .. } => {
                *data = value;
                Ok(self)
            }
            _ => Err(self),
        }
    }
}
