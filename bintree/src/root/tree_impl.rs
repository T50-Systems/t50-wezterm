impl<L, N> Tree<L, N> {
    /// Construct a new empty tree
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::Empty
    }

    /// Returns true if the tree is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Transform the tree into its Zipper based Cursor representation
    pub fn cursor(self) -> Cursor<L, N> {
        Cursor {
            it: Box::new(self),
            path: Box::new(Path::Top),
        }
    }

    pub fn num_leaves(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Leaf(_) => 1,
            Self::Node { left, right, .. } => left.num_leaves() + right.num_leaves(),
        }
    }
}
