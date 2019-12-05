#![no_std]

//!
//! Driver crate for the Texas Instruments ADS1292 24-bit 2-channel low-power analog front end for ECG applications.
//!
//! This initial version supports the ADS1292 (for the most part), but it's a goal to support the ADS1291 and ADS1292R as well.
//!
//! Usage:
//!
//! ```
//!// spi: spi interface
//!// ncs: not-Chip-Select pin
//!// timer: timer, 500kHz timeout.
//!
//!let spi_device = SpiDevice::new(spi, ncs, timer)?;
//!let mut ads = Ads1292::init(spi_device)?;
//!
//!// start conversions
//!ads.cmd(ads129xx::Command::START).unwrap();
//!ads.wait(200)?; // Wait a while in between sending commands
//!ads.cmd(ads129xx::Command::RDATAC).unwrap();
//!ads.wait(200)?;
//!
//!let mut stream = ads.into_data_stream()?;
//!
//!let mut buf = [[ChannelData::default(); 2]; 10];
//!
//!// Opens stream, sends RDATAC command to ads
//!let data_stream = ads1292.into_data_stream()?;
//!ads.wait(200)?;
//!
//!// A buffer to read data into
//!let mut buf = [[Ads1292Data::default(); 2]; 2000];
//!
//!for i in buf.iter_mut() {
//!    // some way of finding out NDRDY has been low since last read (preferably by an interrupt-set flag)
//    while !data_ready() {}
//!    // data_stream always returns data (for now), so we can unwrap here
//!    *i = data_stream.next().unwrap()?;
//!}
//!// Don't forget to close; this will send the SDATAC command to the ads
//!data_stream.into_inner();
//!```
//!

use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use crate::spi::SpiDevice;

/// Ads1292-specific code
pub mod ads1292;
/// Data representation
pub mod data;
mod register;
/// SPI interface
pub mod spi;
mod util;

pub use register::*;

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

macro_rules! simple_register {
    ($read_name:ident, $write_name:ident, $register:ident, $valuetype:ident) => {
        #[inline]
        fn $read_name(&mut self) -> Result<($valuetype), E, EO> {
            Ok($valuetype(self.read_register(Register::$register)?))
        }
        #[inline]
        fn $write_name(&mut self, value: &$valuetype) -> Result<(), E, EO> {
            self.write_register(Register::$register, value.0)
        }
    }
}

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

    simple_register!(read_conf1, write_conf1, CONFIG1, Conf1);
    simple_register!(read_conf2, write_conf2, CONFIG2, Conf2);
    simple_register!(read_loff, write_loff, LOFF, Loff);
    simple_register!(read_loff_sens, write_loff_sens, LOFF_SENS, LoffSense);
    simple_register!(read_chan1, write_chan1, CH1SET, ChannelSettings);
    simple_register!(read_chan2, write_chan2, CH2SET, ChannelSettings);
    simple_register!(read_rld_sens, write_rld_sens, RLD_SENS, RLDSenseSelection);
    simple_register!(read_resp_conf2, write_resp_conf2, RESP2, RespConf2);
}
