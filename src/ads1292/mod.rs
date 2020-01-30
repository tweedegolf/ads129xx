/// ADS1292-specific data formats
pub mod data;
/// ADS1292-specific data stream
pub mod data_stream;

use crate::spi::SpiDevice;
use crate::{Ads129xx, Ads129xxError, Command, Register, Result};

use data::Ads1292Data;
use data_stream::Ads1292DataStream;
use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

/// Represents an ADS1292 ECG front-end module
pub struct Ads1292<SPI, NCS, TIM> {
    spi: SpiDevice<SPI, NCS, TIM>,
}

impl<SPI, NCS, TIM, E> Ads1292<SPI, NCS, TIM>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    TIM: CountDown,
{
    /// Create a new Ads1292.
    pub fn new(spi: SpiDevice<SPI, NCS, TIM>) -> Ads1292<SPI, NCS, TIM> {
        Ads1292 { spi }
    }

    /// Initialize the Ads1292. Sends SDATAC command, as by default it is in continuous data
    /// reading mode. Check that it reports a valid device ID.
    pub fn init(&mut self) -> Result<(), E> {
        // We start in DATAC, thus need to stop it.
        self.cmd(Command::SDATAC)?;
        self.spi.wait(40);

        let id = self.read_register(Register::ID)?;
        if id & 0x10 != 0x10 {
            // Bit 4 must be high in ID.
            return Err(Ads129xxError::BootFailure);
        }

        Ok(())
    }

    /// Send RDATA command and read a single data block from the ADS1292
    #[inline]
    pub fn read_data(&mut self) -> Result<Ads1292Data, E> {
        // Send Read command
        self.cmd(Command::RDATA)?;
        let mut buf = [0u8; 9];
        // Receive data
        self.spi.transfer(&mut buf)?;
        Ok(buf.into())
    }

    /// Read a single data block without sending the RDATA command first
    /// To be used in RDATAC mode.
    /// WARNING: This function retrieves ecg data more power efficiently by avoiding the delays
    /// that are usually necessary when communicating with the ADS1292 device. Use a delay of at
    /// least 50 microseconds between retrieving samples and between retrieving a sample and
    /// sending any other command, register read or register write.
    pub fn read(&mut self) -> Result<Ads1292Data, E> {
        let mut buf = [0u8; 9];

        // Receive data
        unsafe {
            self.spi.unsafe_transfer(&mut buf)?;
        }
        Ok(buf.into())
    }

    /// Convert this Ads1292 into a Ads1292DataStream
    pub fn into_data_stream(self) -> Result<Ads1292DataStream<SPI, NCS, TIM, E>, E> {
        Ads1292DataStream::init(self)
    }
}

impl<SPI, NCS, TIM, E> Ads129xx<SPI, NCS, TIM, E> for Ads1292<SPI, NCS, TIM>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = core::convert::Infallible>,
    TIM: CountDown,
{
    fn spi_device(&mut self) -> &mut SpiDevice<SPI, NCS, TIM> {
        &mut self.spi
    }

    fn into_spi_device(self) -> SpiDevice<SPI, NCS, TIM> {
        self.spi
    }
}
