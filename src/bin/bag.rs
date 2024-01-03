use std::{cell::RefCell, rc::Rc};

use diplomatic_bag::DiplomaticBag;

fn main() {
    // // `Rc` is neither `Send` nor `Sync`
    // let foo = DiplomaticBag::new(|_| Rc::new(RefCell::new(0)));

    // std::thread::spawn({
    //     let foo = foo.clone();
    //     move || {
    //         foo.as_ref().map(|_, rc| {
    //             *rc.borrow_mut() = 1;
    //         });
    //     }
    // });

    use diplomatic_bag::DiplomaticBag;
    let one = DiplomaticBag::new(|_handler| 1);
    let two = std::thread::spawn(|| DiplomaticBag::new(|_handler| 2))
        .join()
        .unwrap();
    let three = one.and_then(|handler, one| one + handler.unwrap(two));

    assert_eq!(3, three);
}
