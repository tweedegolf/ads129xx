#![no_std]

use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use crate::spi::SpiDevice;

pub mod ads1292;
pub mod data;
pub mod spi;
mod util;

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
pub enum Ads129xxError<E, EO> {
    BootFailure,
    /// SPI bus error
    SpiError(spi::SpiError<E, EO>),
}

impl<E, EO> Into<Ads129xxError<E, EO>> for crate::spi::SpiError<E, EO> {
    fn into(self) -> Ads129xxError<E, EO> {
        Ads129xxError::SpiError(self)
    }
}

pub type Result<T, E, EO> = core::result::Result<T, Ads129xxError<E, EO>>;

/// Represents any ADS129xx device
pub trait Ads129xx<SPI, NCS, TIM, E, EO>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    /// Get a mutable reference to the wrapped SpiDevice
    fn spi_device(&mut self) -> &mut SpiDevice<SPI, NCS, TIM>;

    /// Consume self and return the wrapped SpiDevice
    fn into_spi_device(self) -> SpiDevice<SPI, NCS, TIM>;

    /// Send a command to the ADS129xx
    #[inline]
    fn cmd(&mut self, cmd: Command) -> Result<(), E, EO> {
        self.spi_device().write(&[cmd.word()]).map_err(|e| e.into())
    }

    #[inline]
    fn wait(&mut self, i: u16) -> Result<(), E, EO> {
        self.spi_device().wait(i).map_err(|e| e.into())
    }

    /// Read a register of the ADS1292
    #[inline]
    fn read_register(&mut self, reg: Register) -> Result<u8, E, EO> {
        let nreg = 0x00; // n = 1, but subtract 1
        let mut buf: [u8; 4] = [Command::RREG.word() | reg.addr(), nreg, 0x00, 0x00];
        self.spi_device().transfer(&mut buf).map_err(|e| e.into())?;
        Ok(buf[2])
    }

    /// Write in register of the ADS1292
    #[inline]
    fn write_register(&mut self, reg: Register, data: u8) -> Result<(), E, EO> {
        let nreg = 0x00; // n = 1, but subtract 1
        let buf: [u8; 3] = [Command::WREG.word() | reg.addr(), nreg, data];
        self.spi_device().write(&buf).map_err(|e| e.into())?;
        Ok(())
    }
}
