trait Row {}

trait DefaultOrd {
    fn cmp_default(&self, _: &Self) -> std::cmp::Ordering;
}

struct DefaultOrdered<T>(T);

impl<T: DefaultOrd> PartialEq for DefaultOrdered<T> {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<T: DefaultOrd> PartialOrd for DefaultOrdered<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.cmp_default(&other.0).into()
    }
}

impl<T: DefaultOrd> Eq for DefaultOrdered<T> {}

impl<T: DefaultOrd> Ord for DefaultOrdered<T> {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl<R: Row> Row for DefaultOrdered<R> {}

struct OwnedRow;
struct ScalarImpl;

type DefaultOrdOwnedRow = DefaultOrdered<OwnedRow>;
type DefaultOrdScalarImpl = DefaultOrdered<ScalarImpl>;

fn main() {}
