//! « Annuler »'s token (D92) in its three orders, a played program counting its stops: cancelled
//! before the program starts, between its start and its hold, and once held.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use kitokey_shell::cancel::Cancel;

/// A played program's stop, counted.
fn program() -> (Arc<AtomicUsize>, impl FnOnce() + Send + 'static) {
    let stops = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&stops);
    (stops, move || {
        counted.fetch_add(1, Ordering::SeqCst);
    })
}

#[test]
fn cancelled_before_its_program_starts_it_says_so_and_none_is_started() {
    let token = Cancel::default();
    assert!(!token.cancelled());
    token.cancel();
    assert!(token.cancelled(), "whoever would start it must see it cancelled");
}

#[test]
fn cancelled_between_its_programs_start_and_its_hold_the_program_is_stopped_as_it_is_held() {
    let token = Cancel::default();
    let (stops, stop) = program();
    // Started; « Annuler » now; then held.
    token.cancel();
    assert_eq!(stops.load(Ordering::SeqCst), 0);
    token.hold(stop);
    assert_eq!(stops.load(Ordering::SeqCst), 1, "held after a cancel, it is stopped at once");
    token.cancel();
    assert_eq!(stops.load(Ordering::SeqCst), 1, "stopped once");
}

#[test]
fn cancelled_once_held_the_program_is_stopped_once() {
    let token = Cancel::default();
    let (stops, stop) = program();
    token.hold(stop);
    assert_eq!(stops.load(Ordering::SeqCst), 0, "held, it runs");
    token.cancel();
    token.cancel();
    assert_eq!(stops.load(Ordering::SeqCst), 1);
    assert!(token.cancelled());
}

#[test]
fn a_cancel_racing_the_hold_stops_the_program_exactly_once_whichever_comes_first() {
    for _ in 0..2000 {
        let token = Cancel::default();
        let (stops, stop) = program();
        let cancelling = token.clone();
        let cancel = std::thread::spawn(move || cancelling.cancel());
        token.hold(stop);
        cancel.join().unwrap();
        assert_eq!(stops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn a_making_is_told_from_another_by_its_token() {
    let one = Cancel::default();
    assert!(one.same(&one.clone()));
    assert!(!one.same(&Cancel::default()));
}
