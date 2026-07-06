use std::cmp::PartialEq;
use std::fmt::Debug;

/// Represents a (mostly) "proper" binary tree; each Node has 0 or 2 children,
/// but there is a special case where the tree is rooted with a single leaf node.
/// Non-leaf nodes in the tree can be labelled with an optional node data type `N`,
/// which defaults to `()`.
/// Leaf nodes have a required leaf data type `L`.
pub enum Tree<L, N = ()> {
    Empty,
    Node {
        left: Box<Self>,
        right: Box<Self>,
        data: Option<N>,
    },
    Leaf(L),
}

impl<L, N> PartialEq for Tree<L, N>
where
    L: PartialEq,
    N: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Empty, Self::Empty) => true,
            (
                Self::Node {
                    left: l_left,
                    right: l_right,
                    data: l_data,
                },
                Self::Node {
                    left: r_left,
                    right: r_right,
                    data: r_data,
                },
            ) => (l_left == r_left) && (l_right == r_right) && (l_data == r_data),
            (Self::Leaf(l), Self::Leaf(r)) => l == r,
            _ => false,
        }
    }
}

impl<L, N> Debug for Tree<L, N>
where
    L: Debug,
    N: Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Empty => fmt.write_str("Empty"),
            Self::Node { left, right, data } => fmt
                .debug_struct("Node")
                .field("left", &left)
                .field("right", &right)
                .field("data", &data)
                .finish(),
            Self::Leaf(l) => fmt.debug_tuple("Leaf").field(&l).finish(),
        }
    }
}

/// Represents a location in the tree for the Zipper; the path contains directions
/// from the current position back towards the root of the tree.
enum Path<L, N> {
    /// The current position is the top of the tree
    Top,
    /// The current position is the left hand side of its parent node;
    /// Cursor::it holds the left node of the tree with the fields here
    /// in Path::Left representing the partially constructed state of
    /// the parent Tree::Node
    Left {
        right: Box<Tree<L, N>>,
        data: Option<N>,
        up: Box<Self>,
    },
    /// The current position is the right hand side of its parent node;
    /// Cursor::it holds the right node of the tree with the fields here
    /// in Path::Right representing the partially constructed state of
    /// the parent Tree::Node
    Right {
        left: Box<Tree<L, N>>,
        data: Option<N>,
        up: Box<Self>,
    },
}

impl<L, N> Debug for Path<L, N>
where
    L: Debug,
    N: Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Top => fmt.write_str("Top"),
            Self::Left { right, data, up } => fmt
                .debug_struct("Left")
                .field("right", &right)
                .field("data", &data)
                .field("up", &up)
                .finish(),
            Self::Right { left, data, up } => fmt
                .debug_struct("Right")
                .field("left", &left)
                .field("data", &data)
                .field("up", &up)
                .finish(),
        }
    }
}

/// The cursor is used to indicate the current position within the tree and enable
/// constant time mutation operations on that position as well as movement around
/// the tree.
/// The cursor isn't a reference to a location within the tree; it is an alternate
/// representation of the tree and thus requires ownership of the tree to create.
/// When you are done using the cursor you may wish to transform it back into
/// a tree.
pub struct Cursor<L, N> {
    it: Box<Tree<L, N>>,
    path: Box<Path<L, N>>,
}

impl<L, N> Debug for Cursor<L, N>
where
    L: Debug,
    N: Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Cursor")
            .field("it", &self.it)
            .field("path", &self.path)
            .finish()
    }
}

pub struct ParentIterator<'a, L, N> {
    path: &'a Path<L, N>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathBranch {
    IsLeft,
    IsRight,
}

impl<'a, L, N> std::iter::Iterator for ParentIterator<'a, L, N> {
    type Item = (PathBranch, &'a Option<N>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.path {
            Path::Top => None,
            Path::Left { data, up, .. } => {
                self.path = &*up;
                Some((PathBranch::IsLeft, data))
            }
            Path::Right { data, up, .. } => {
                self.path = &*up;
                Some((PathBranch::IsRight, data))
            }
        }
    }
}
