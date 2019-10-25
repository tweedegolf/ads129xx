pub mod data;
pub mod spi;
pub mod util;

use embedded_hal::blocking::spi as bspi;
use embedded_hal::spi as eh_spi;

use spi::SpiDevice;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use data::Data;

/// SPI mode
pub const MODE: eh_spi::Mode = eh_spi::MODE_1;

/// Read / write-able registers
///
/// Table 14 page 39 of specification.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Register {
    /// ID Control Register (Factory-Programmed, Read-Only)
    ID = 0x00,
    /// Configuration Register 1
    CONFIG1 = 0x01,
    /// Configuration Register 2
    CONFIG2 = 0x02,
    /// Lead-Off Control Register
    LOFF = 0x03,
    /// Channel 1 Settings
    CH1SET = 0x04,
    /// Channel 2 Settings
    CH2SET = 0x05,
    /// Right Leg Drive Sense Selection
    RLD_SENS = 0x06,
    /// Lead-Off Sense Selection
    LOFF_SENS = 0x07,
    /// Lead-Off Status
    LOFF_STAT = 0x08,
    /// Respiration Control Register 1
    RESP1 = 0x09,
    /// Respiration Control Register 2    
    RESP2 = 0x0A,
    /// General-Purpose I/O Register
    GPIO = 0x0B,
}

impl Register {
    #[inline]
    pub fn addr(self) -> u8 {
        self as u8
    }
}

/// SPI commands
///
/// Table 13 page 35 of specification.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Command {
    /// Wake-up from standby mode
    WAKEUP = 0x02,
    /// Enter standy mode
    STANDBY = 0x04,
    /// Reset the device
    RESET = 0x06,
    /// Start or restart (synchronize) conversions
    START = 0x08,
    /// Stop conversion
    STOP = 0x0A,
    /// Channel offset calibration
    OFFSETCAL = 0x1A,
    /// Enable Read Data Continuous Mode (default @ powerup)
    ///
    /// During this mode RREG commands are ignored.
    RDATAC = 0x10,
    /// Stop Read Data Continuously Mode
    SDATAC = 0x11,
    /// Read data by command; supports multiple read back
    RDATA = 0x12,
    /// Read registers starting at an address
    RREG = 0x20,
    /// Write registers starting at an address
    WREG = 0x40,
}

impl Command {
    #[inline]
    pub fn word(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Ads1292Error<E, EO> {
    BootFailure,
    /// SPI bus error
    SpiError(crate::spi::SpiError<E, EO>),
}

impl<E, EO> Into<Ads1292Error<E, EO>> for crate::spi::SpiError<E, EO> {
    fn into(self) -> Ads1292Error<E, EO> {
        Ads1292Error::SpiError(self)
    }
}

pub struct Ads1292DataStream<'a, SPI, NCS, TIM> {
    ads1292: &'a mut Ads1292<SPI, NCS, TIM>,
}

impl<'a, SPI, NCS, TIM, E, EO> Iterator for Ads1292DataStream<'a, SPI, NCS, TIM>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    type Item = Result<Ads1292Data, Ads1292Error<E, EO>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; 9];
        Some(
            self.ads1292
                .spi
                .transfer(&mut buf)
                .map_err(|e| e.into())
                .map(|_| Ads1292Data { data: buf }),
        )
    }
}

pub struct Ads1292<SPI, NCS, TIM> {
    spi: SpiDevice<SPI, NCS, TIM>,
}

impl<SPI, NCS, TIM, E, EO> Ads1292<SPI, NCS, TIM>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    pub fn init(
        spi: SpiDevice<SPI, NCS, TIM>,
    ) -> Result<Ads1292<SPI, NCS, TIM>, Ads1292Error<E, EO>> {
        let mut result = Ads1292 { spi };

        // We start in DATAC, thus need to stop it.
        result.cmd(Command::SDATAC)?;
        result.spi.wait(40).map_err(|e| e.into())?;

        let id = result.read_register(Register::ID)?;
        if id & 0x10 != 0x10 {
            // Bit 4 must be high in ID.
            return Err(Ads1292Error::BootFailure);
        }

        Ok(result)
    }

    #[inline]
    pub fn cmd(&mut self, cmd: Command) -> Result<(), Ads1292Error<E, EO>> {
        self.spi.write(&[cmd.word()]).map_err(|e| e.into())
    }

    #[inline]
    pub fn read_register(&mut self, reg: Register) -> Result<u8, Ads1292Error<E, EO>> {
        let nreg = 0x00; // n = 1, but subtract 1
        let mut buf: [u8; 4] = [Command::RREG.word() | reg.addr(), nreg, 0x00, 0x00];
        self.spi.transfer(&mut buf).map_err(|e| e.into())?;
        Ok(buf[2])
    }

    #[inline]
    pub fn write_register(&mut self, reg: Register, data: u8) -> Result<(), Ads1292Error<E, EO>> {
        let nreg = 0x00; // n = 1, but subtract 1
        let buf: [u8; 3] = [Command::WREG.word() | reg.addr(), nreg, data];
        self.spi.write(&buf).map_err(|e| e.into())?;
        Ok(())
    }

    #[inline]
    pub fn read_data(&mut self) -> Result<Ads1292Data, Ads1292Error<E, EO>> {
        let mut buf = [0u8; 9];
        // Send Read command
        self.cmd(Command::RDATA)?;
        // Receive data
        self.spi.transfer(&mut buf).map_err(|e| e.into())?;
        Ok(buf.into())
    }

    // TODO remove in favor of some standard way to release the resource
    pub fn release(self) -> SpiDevice<SPI, NCS, TIM> {
        self.spi
    }
}

impl<'a, SPI, NCS, TIM, E, EO> IntoIterator for &'a mut Ads1292<SPI, NCS, TIM>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    type Item = Result<Data, Ads1292Error<E, EO>>;
    type IntoIter = Ads1292DataStream<'a, SPI, NCS, TIM>;

    fn into_iter(self) -> Self::IntoIter {
        Ads1292DataStream { ads1292: self }
    }
}
