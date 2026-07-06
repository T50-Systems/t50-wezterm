impl<L, N> Cursor<L, N> {
    /// If the current position is a non-root leaf node, remove it
    /// and unsplit its parent by replacing its parent with either
    /// the opposite branch of the tree from this leaf.
    /// On success, yields the revised cursor, which now points to
    /// the newly unsplit node, along with the leaf value and prior
    /// parent node value.
    /// On failure, yields `Err` containing the unchanged cursor.
    pub fn unsplit_leaf(self) -> Result<(Self, L, Option<N>), Self> {
        if !self.is_leaf() || self.is_top() {
            return Err(self);
        }

        match (*self.it, *self.path) {
            (Tree::Leaf(l), Path::Left { right, data, up }) => Ok((
                Self {
                    it: right,
                    path: up,
                },
                l,
                data,
            )),
            (Tree::Leaf(l), Path::Right { left, data, up }) => {
                Ok((Self { it: left, path: up }, l, data))
            }
            (Tree::Leaf(_), Path::Top) => unreachable!(),
            (Tree::Empty, _) => unreachable!(),
            (Tree::Node { .. }, _) => unreachable!(),
        }
    }

    pub fn split_node_and_insert_left(self, to_insert: L) -> Result<Self, Self> {
        match *self.it {
            Tree::Node { left, right, data } => Ok(Self {
                it: Box::new(Tree::Node {
                    data: None,
                    right: Box::new(Tree::Node { left, right, data }),
                    left: Box::new(Tree::Leaf(to_insert)),
                }),
                path: self.path,
            }),
            _ => Err(self),
        }
    }

    pub fn split_node_and_insert_right(self, to_insert: L) -> Result<Self, Self> {
        match *self.it {
            Tree::Node { left, right, data } => Ok(Self {
                it: Box::new(Tree::Node {
                    data: None,
                    left: Box::new(Tree::Node { left, right, data }),
                    right: Box::new(Tree::Leaf(to_insert)),
                }),
                path: self.path,
            }),
            _ => Err(self),
        }
    }

    /// If the current position is a leaf, split it into a Node where
    /// the left side holds the current leaf value and the right side
    /// holds the provided `right` value.
    /// The cursor position remains unchanged.
    /// Consumes the cursor and returns a new cursor representing the
    /// mutated tree.
    /// If the current position is not a leaf, yields `Err` containing
    /// the unchanged cursor.
    pub fn split_leaf_and_insert_right(self, right: L) -> Result<Self, Self> {
        match *self.it {
            Tree::Leaf(left) => Ok(Self {
                it: Box::new(Tree::Node {
                    data: None,
                    left: Box::new(Tree::Leaf(left)),
                    right: Box::new(Tree::Leaf(right)),
                }),
                path: self.path,
            }),
            _ => Err(self),
        }
    }

    /// If the current position is a leaf, split it into a Node where
    /// the right side holds the current leaf value and the left side
    /// holds the provided `left` value.
    /// The cursor position remains unchanged.
    /// Consumes the cursor and returns a new cursor representing the
    /// mutated tree.
    /// If the current position is not a leaf, yields `Err` containing
    /// the unchanged cursor.
    pub fn split_leaf_and_insert_left(self, left: L) -> Result<Self, Self> {
        match *self.it {
            Tree::Leaf(right) => Ok(Self {
                it: Box::new(Tree::Node {
                    data: None,
                    left: Box::new(Tree::Leaf(left)),
                    right: Box::new(Tree::Leaf(right)),
                }),
                path: self.path,
            }),
            _ => Err(self),
        }
    }

    /// If the current position is not a leaf, move the cursor to
    /// its left child.
    /// Consumes the cursor and returns a new cursor representing the
    /// mutated tree.
    /// If the current position is a Leaf, yields `Err` containing
    /// the unchanged cursor.
    pub fn go_left(self) -> Result<Self, Self> {
        match *self.it {
            Tree::Node { left, right, data } => Ok(Self {
                it: left,
                path: Box::new(Path::Left {
                    data,
                    right,
                    up: self.path,
                }),
            }),
            _ => Err(self),
        }
    }

    /// If the current position is not a leaf, move the cursor to
    /// its right child.
    /// Consumes the cursor and returns a new cursor representing the
    /// mutated tree.
    /// If the current position is a Leaf, yields `Err` containing
    /// the unchanged cursor.
    pub fn go_right(self) -> Result<Self, Self> {
        match *self.it {
            Tree::Node { left, right, data } => Ok(Self {
                it: right,
                path: Box::new(Path::Right {
                    data,
                    left,
                    up: self.path,
                }),
            }),
            _ => Err(self),
        }
    }

    /// If the current position is not at the root of the tree,
    /// move up to the parent of the current position.
    /// Consumes the cursor and returns a new cursor representing the
    /// new location.
    /// If the current position is the top of the tree,
    /// yields `Err` containing the unchanged cursor.
    pub fn go_up(self) -> Result<Self, Self> {
        match *self.path {
            Path::Top => Err(self),
            Path::Right { left, data, up } => Ok(Self {
                it: Box::new(Tree::Node {
                    left,
                    right: self.it,
                    data,
                }),
                path: up,
            }),
            Path::Left { right, data, up } => Ok(Self {
                it: Box::new(Tree::Node {
                    right,
                    left: self.it,
                    data,
                }),
                path: up,
            }),
        }
    }

    /// Move the current position to the next in a preorder traversal.
    /// Returns the modified cursor position.
    ///
    /// In the case where there are no more nodes in the preorder traversal,
    /// yields `Err` with the newly adjusted cursor; calling `preorder_next`
    /// after it has yielded `Err` can potentially yield `Ok` with previously
    /// visited nodes, so the caller must take care to stop iterating when
    /// `Err` is received!
    pub fn preorder_next(mut self) -> Result<Self, Self> {
        // Since we are a "proper" binary tree, we know we cannot have
        // difficult cases such as a left without a right or vice versa.

        if self.is_leaf() {
            if self.is_left() {
                return self.go_up()?.go_right();
            }

            // while (We were on the right)
            loop {
                self = self.go_up()?;

                if self.is_top() {
                    return Err(self);
                }

                if self.is_left() {
                    return self.go_up()?.go_right();
                }
            }
        } else {
            self.go_left()
        }
    }

    /// Move the current position to the next in a postorder traversal.
    /// Returns the modified cursor position.
    ///
    /// In the case where there are no more nodes in the postorder traversal,
    /// yields `Err` with the newly adjusted cursor; calling `postorder_next`
    /// after it has yielded `Err` can potentially yield `Ok` with previously
    /// visited nodes, so the caller must take care to stop iterating when
    /// `Err` is received!
    pub fn postorder_next(mut self) -> Result<Self, Self> {
        // Since we are a "proper" binary tree, we know we cannot have
        // difficult cases such as a left without a right or vice versa.

        if self.is_leaf() {
            if self.is_right() {
                return self.go_up()?.go_left();
            }

            // while (We were on the left)
            loop {
                self = self.go_up()?;

                if self.is_top() {
                    return Err(self);
                }

                if self.is_right() {
                    return self.go_up()?.go_left();
                }
            }
        } else {
            self.go_right()
        }
    }

    /// Move to the nth (preorder) leaf from the current position.
    pub fn go_to_nth_leaf(mut self, n: usize) -> Result<Self, Self> {
        let mut next = 0;
        loop {
            if self.is_leaf() {
                if next == n {
                    return Ok(self);
                }
                next += 1;
            }
            self = self.preorder_next()?;
        }
    }

    /// Consume the cursor and return the root of the Tree
    pub fn tree(mut self) -> Tree<L, N> {
        loop {
            self = match self.go_up() {
                Ok(up) => up,
                Err(top) => return *top.it,
            }
        }
    }
}
