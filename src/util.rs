use embedded_hal::timer::CountDown;
use nb::block;

/// Blockingly wait i clock overflows.
pub fn wait<TIM: CountDown>(timer: &mut TIM, i: u16) {
    for _ in 0..i {
        match block!(timer.wait()) {
            Ok(()) => (),
            Err(never) => match never {},
        }
    }
}
