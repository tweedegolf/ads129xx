use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

#[derive(Debug, Copy, Clone)]
pub enum SpiError<E, E2> {
    /// SPI bus I/O error
    BusError(E),
    /// Error setting the nCS pin
    NCSError(E2),
    /// An error occurred whilst waiting
    WaitError,
}

impl<E, E2> core::convert::From<E> for SpiError<E, E2> {
    fn from(error: E) -> Self {
        SpiError::BusError(error)
    }
}

/// A SPI device also triggering the nCS-pin when suited.
pub struct SpiDevice<SPI, NCS, TIM> {
    /// Underlying peripheral
    spi: SPI,
    /// nCS
    ncs: NCS,
    /// Timer for nCS delay
    timer: TIM,
}

impl<SPI, NCS, TIM, E, EO> SpiDevice<SPI, NCS, TIM>
where
    SPI: bspi::Write<u8, Error = E> + bspi::Transfer<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    pub fn new(spi: SPI, mut ncs: NCS, timer: TIM) -> Result<Self, SpiError<E, EO>> {
        ncs.set_high().map_err(SpiError::NCSError)?;

        Ok(SpiDevice { spi, ncs, timer })
    }

    #[inline]
    pub fn transfer(&mut self, buffer: &mut [u8]) -> Result<(), SpiError<E, EO>> {
        self.ncs.set_low().map_err(SpiError::NCSError)?;
        let res = (|| {
            self.wait(20)?;
            self.spi.transfer(buffer)?;
            self.wait(20)
        })();
        self.ncs.set_high().map_err(SpiError::NCSError)?;
        self.wait(10)?;

        res?; // Drop out of function with SPIError only after setting NCS.
        Ok(())
    }

    #[inline]
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), SpiError<E, EO>> {
        self.ncs.set_low().map_err(SpiError::NCSError)?;
        let res = (|| {
            self.wait(20)?;
            self.spi.write(buffer)?;
            self.wait(20)
        })();
        self.ncs.set_high().map_err(SpiError::NCSError)?;
        self.wait(10)?;

        res?; // Drop out of function with SPIError only setting NCS.
        Ok(())
    }

    // TODO remove?
    pub fn wait(&mut self, i: u16) -> Result<(), SpiError<E, EO>> {
        crate::util::wait(&mut self.timer, i).map_err(|_| SpiError::WaitError)
    }
}
