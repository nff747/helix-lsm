use helix_lsm::write_throttle::{ThrottleAction, WriteThrottle};

#[test]
fn test_throttle_states() {
    let throttle = WriteThrottle::new(8, 12);
    assert_eq!(throttle.evaluate(5), ThrottleAction::Proceed);
    assert_eq!(throttle.evaluate(8), ThrottleAction::DelayMillis(5));
    assert_eq!(throttle.evaluate(12), ThrottleAction::Block);
}
