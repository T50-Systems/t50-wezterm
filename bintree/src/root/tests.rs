#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_split_and_iterate() {
        let t: Tree<i32, i32> = Tree::new()
            .cursor()
            .assign_top(1)
            .unwrap()
            .split_leaf_and_insert_right(2)
            .unwrap()
            .tree();

        let t = t
            .cursor()
            .go_to_nth_leaf(1)
            .unwrap()
            .split_leaf_and_insert_right(3)
            .unwrap()
            .tree();

        let mut leaves = vec![];

        let mut cursor = t.cursor();
        loop {
            eprintln!("cursor: {:?}", cursor);
            if cursor.is_leaf() {
                leaves.push(*cursor.leaf_mut().unwrap());
            }
            match cursor.preorder_next() {
                Ok(c) => cursor = c,
                Err(_) => break,
            }
        }

        assert_eq!(leaves, vec![1, 2, 3]);
    }

    #[test]
    fn populate() {
        let t: Tree<i32, i32> = Tree::new()
            .cursor()
            .assign_top(1)
            .unwrap()
            .split_leaf_and_insert_right(2)
            .unwrap()
            .tree();

        assert_eq!(
            t,
            Tree::Node {
                left: Box::new(Tree::Leaf(1)),
                right: Box::new(Tree::Leaf(2)),
                data: None
            }
        );

        let t = t.cursor().assign_node(Some(100)).unwrap().tree();

        assert_eq!(
            t,
            Tree::Node {
                left: Box::new(Tree::Leaf(1)),
                right: Box::new(Tree::Leaf(2)),
                data: Some(100),
            }
        );

        let t = t
            .cursor()
            .go_left()
            .unwrap()
            .split_leaf_and_insert_left(3)
            .unwrap()
            .assign_node(Some(101))
            .unwrap()
            .go_left()
            .unwrap()
            .split_leaf_and_insert_right(4)
            .unwrap()
            .assign_node(Some(102))
            .unwrap()
            .go_left()
            .unwrap()
            .split_leaf_and_insert_right(5)
            .unwrap()
            .assign_node(Some(103))
            .unwrap()
            .tree();

        assert_eq!(
            t,
            Tree::Node {
                left: Box::new(Tree::Node {
                    left: Box::new(Tree::Node {
                        left: Box::new(Tree::Node {
                            left: Box::new(Tree::Leaf(3)),
                            right: Box::new(Tree::Leaf(5)),
                            data: Some(103)
                        }),
                        right: Box::new(Tree::Leaf(4)),
                        data: Some(102)
                    }),
                    right: Box::new(Tree::Leaf(1)),
                    data: Some(101)
                }),
                right: Box::new(Tree::Leaf(2)),
                data: Some(100),
            }
        );

        let mut cursor = t.cursor();
        assert_eq!(100, cursor.node_mut().unwrap().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(101, cursor.node_mut().unwrap().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(102, cursor.node_mut().unwrap().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(103, cursor.node_mut().unwrap().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(3, cursor.leaf_mut().copied().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(5, cursor.leaf_mut().copied().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(4, cursor.leaf_mut().copied().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(1, cursor.leaf_mut().copied().unwrap());

        cursor = cursor.preorder_next().unwrap();
        assert_eq!(2, cursor.leaf_mut().copied().unwrap());

        assert!(cursor.preorder_next().is_err());
    }
}
