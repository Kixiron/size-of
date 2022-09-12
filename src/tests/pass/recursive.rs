use core::mem::size_of;
use size_of::{SizeOf, TotalSize};
#[cfg(not(feature = "derive"))]
use size_of_derive::SizeOf;

#[derive(SizeOf)]
enum Action {
    Action1,
    Action2,
    InjectActions {
        // FIXME: This should happen automatically where possible
        #[size_of(skip_bounds)]
        injected_actions: Vec<Self>,
    },
}

fn main() {
    let action1 = Action::Action1.size_of();
    assert_eq!(action1, TotalSize::total(size_of::<Action>()));

    let action2 = Action::Action2.size_of();
    assert_eq!(action2, TotalSize::total(size_of::<Action>()));

    let empty_injection = Action::InjectActions {
        injected_actions: Vec::new(),
    }
    .size_of();
    assert_eq!(empty_injection, TotalSize::total(size_of::<Action>()));

    let filled_injection = Action::InjectActions {
        injected_actions: vec![Action::Action1, Action::Action2],
    }
    .size_of();
    assert_eq!(
        filled_injection,
        TotalSize::new(size_of::<Action>() * 3, 0, 0, 1),
    );
}
