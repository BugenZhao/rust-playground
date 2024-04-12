use rusty_fork::rusty_fork_test;

fn main() {}

rusty_fork_test! {
    #[test]
    #[should_panic(expected = "233")]
    fn test() {
        assert_eq!(2, 3);
    }
}
