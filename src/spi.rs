use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use embedded_hal::spi as eh_spi;

fn infallible<T>(r: core::result::Result<T, core::convert::Infallible>) -> T {
    match r {
        Ok(x) => x,
        Err(never) => match never {},
    }
}

/// SPI mode
pub const MODE: eh_spi::Mode = eh_spi::MODE_1;

/// A SPI device also triggering the nCS-pin when suited.
pub struct SpiDevice<SPI, NCS, TIM> {
    /// Underlying peripheral
    spi: SPI,
    /// nCS
    ncs: NCS,
    /// Timer for nCS delay
    timer: TIM,
}

impl<SPI, NCS, TIM, E> SpiDevice<SPI, NCS, TIM>
where
    SPI: bspi::Write<u8, Error = E> + bspi::Transfer<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    TIM: CountDown,
{
    /// Create a new SPI device
    pub fn new(spi: SPI, mut ncs: NCS, timer: TIM) -> Self {
        infallible(ncs.set_high());

        SpiDevice { spi, ncs, timer }
    }

    /// Transfer the buffer to the device, the passed buffer will contain the read data.
    #[inline]
    pub fn transfer(&mut self, buffer: &mut [u8]) -> Result<(), E> {
        infallible(self.ncs.set_low());
        self.wait(20);
        let res = self.spi.transfer(buffer);
        self.wait(20);
        infallible(self.ncs.set_high());
        self.wait(10);
        res?; // Drop out of function with SPIError only after setting NCS.
        Ok(())
    }

    /// Transfer the buffer to the device, the passed buffer will contain the read data.
    /// WARNING: This function runs spi transfers more power efficiently by avoiding the delays
    /// that are usually necessary when communicating with the ADS1292 device. Use a delay of at
    /// least 50 microseconds between uses of this and other spi transfer and write functions.
    #[inline]
    pub unsafe fn unsafe_transfer(&mut self, buffer: &mut [u8]) -> Result<(), E> {
        infallible(self.ncs.set_low());
        let res = self.spi.transfer(buffer);
        infallible(self.ncs.set_high());
        res?; // Drop out of function with SPIError only after setting NCS.
        Ok(())
    }

    /// Write a number of bytes to the device.
    #[inline]
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), E> {
        infallible(self.ncs.set_low());
        self.wait(20);
        let res = self.spi.write(buffer);
        self.wait(20);
        infallible(self.ncs.set_high());
        self.wait(10);
        res?; // Drop out of function with SPIError only after setting NCS.
        Ok(())
    }

    pub fn wait(&mut self, i: u16) {
        crate::util::wait(&mut self.timer, i);
    }

    /// Consume self and release inner resources.
    pub fn into_inner(self) -> (SPI, NCS, TIM) {
        (self.spi, self.ncs, self.timer)
    }
}
