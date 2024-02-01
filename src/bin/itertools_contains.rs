fn main() {
    let mut v = 1..4;
    let v_mut_ref = &mut v;
    {
        assert!(v_mut_ref.contains(&2));
        assert!(v_mut_ref.contains(&1));
    }
    {
        use itertools::Itertools;
        assert!(v_mut_ref.contains(&2));
        assert!(v_mut_ref.contains(&1));
    }
}
