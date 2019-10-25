use embedded_hal::timer::CountDown;
use nb::{block, Result};
use void::Void;

/// Blockingly wait i clock overflows.
pub fn wait<TIM: CountDown>(timer: &mut TIM, i: u16) -> Result<(), Void> {
    for _ in 0..i {
        block!(timer.wait())?;
    }
    Ok(())
}
