use crepe::crepe;
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Strategy {
    NoShuffle,
    Simple,
    Hash,
    Broadcast,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Fact {
    Edge(Id, Id, Strategy),
    Req(Id, usize),
}

type Id = i32;

use Strategy::*;

crepe! {
    @input
    struct FactInput(Fact);

    struct Edge(Id, Id, Strategy);
    struct Req(Id, usize);
    struct Node(Id);

    @output
    #[derive(Debug)]
    struct Para(Id, usize);

    @output
    #[derive(Debug)]
    struct Failed(Id);

    Edge(x, y, s) <- FactInput(i), let Fact::Edge(x, y, s) = i;
    Req(x, p) <- FactInput(i), let Fact::Req(x, p) = i;
    Node(x) <- Edge(x, _, _);
    Node(y) <- Edge(_, y, _);

    Req(y, p) <- Edge(x, y, NoShuffle), Req(x, p);
    Req(x, p) <- Edge(x, y, NoShuffle), Req(y, p);
    Req(y, 1) <- Edge(_, y, Simple);
    Req(x, 1) <- Edge(x, _, Broadcast);

    Failed(x) <- Req(x, p1), Req(x, p2), (p1 != p2);
    Para(x, p) <- Req(x, p), !Failed(x);
    Para(x, 4) <- Node(x), !Req(x, _);
}

fn run(facts: impl IntoIterator<Item = Fact>) {
    let facts = facts.into_iter().map(FactInput);
    let mut runtime = Crepe::new();
    runtime.extend(facts);

    let (ps, fs) = runtime.run();
    let ps = ps.into_iter().sorted_by_key(|p| p.0).collect_vec();
    println!("Parallelism: {ps:?}");
    println!("Failed: {fs:?}");
    println!();
}

fn main() {
    use Fact::*;

    run([
        Req(0, 6),
        Edge(0, 1, NoShuffle),
        Edge(1, 2, Hash),
        Edge(1, 3, Hash),
        Edge(2, 4, NoShuffle),
        Edge(3, 5, Hash),
        Edge(4, 5, Hash),
        Edge(5, 6, Simple),
        Edge(6, 7, NoShuffle),
    ]);

    run([
        Req(0, 8),
        Req(1, 16),
        Edge(0, 2, NoShuffle),
        Edge(1, 2, NoShuffle),
        Edge(2, 3, Hash),
    ]);

    run([
        Req(0, 6),
        Req(1, 1),
        Edge(0, 2, NoShuffle),
        Edge(1, 3, NoShuffle),
        Edge(3, 2, Broadcast),
        Edge(2, 4, Hash),
        Edge(4, 5, Simple),
    ])
}
