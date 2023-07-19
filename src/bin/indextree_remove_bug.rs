use indextree::Arena;

fn main() {
    let mut tree: Arena<i32> = Arena::new();

    for i in 0usize.. {
        let id = tree.new_node(42);
        assert!(!id.is_removed(&tree));

        id.remove(&mut tree);
        assert!(id.is_removed(&tree));

        let new_id = tree.new_node(42);
        assert!(!new_id.is_removed(&tree));
        assert!(id.is_removed(&tree), "i: {i}, id: {id:?}, new_id: {new_id:?}");

        new_id.remove(&mut tree);
    }
}
