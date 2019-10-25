pub mod data;
pub mod data_stream;

use crate::spi::SpiDevice;
use crate::{Ads129xxError, Command, Register, Result};

use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use data::Ads1292Data;
use data_stream::Ads1292DataStream;



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
    ) -> Result<Ads1292<SPI, NCS, TIM>, E, EO> {
        let mut result = Ads1292 { spi };

        // We start in DATAC, thus need to stop it.
        result.cmd(Command::SDATAC)?;
        result.spi.wait(40).map_err(|e| e.into())?;

        let id = result.read_register(Register::ID)?;
        if id & 0x10 != 0x10 {
            // Bit 4 must be high in ID.
            return Err(Ads129xxError::BootFailure);
        }

        Ok(result)
    }

    #[inline]
    pub fn cmd(&mut self, cmd: Command) -> Result<(), E, EO> {
        self.spi.write(&[cmd.word()]).map_err(|e| e.into())
    }

    #[inline]
    pub fn read_register(&mut self, reg: Register) -> Result<u8, E, EO> {
        let nreg = 0x00; // n = 1, but subtract 1
        let mut buf: [u8; 4] = [Command::RREG.word() | reg.addr(), nreg, 0x00, 0x00];
        self.spi.transfer(&mut buf).map_err(|e| e.into())?;
        Ok(buf[2])
    }

    #[inline]
    pub fn write_register(&mut self, reg: Register, data: u8) -> Result<(), E, EO> {
        let nreg = 0x00; // n = 1, but subtract 1
        let buf: [u8; 3] = [Command::WREG.word() | reg.addr(), nreg, data];
        self.spi.write(&buf).map_err(|e| e.into())?;
        Ok(())
    }

    #[inline]
    pub fn read_data(&mut self) -> Result<Ads1292Data, E, EO> {
        let mut buf = [0u8; 9];
        // Send Read command
        self.cmd(Command::RDATA)?;
        // Receive data
        self.spi.transfer(&mut buf).map_err(|e| e.into())?;
        Ok(buf.into())
    }

    pub fn wait(&mut self, i: u16) -> Result<(), E, EO> {
        self.spi.wait(i).map_err(|e| e.into())
    }

    pub fn into_inner(self) -> SpiDevice<SPI, NCS, TIM> {
        self.spi
    }

    pub fn into_data_stream(self) -> Result<Ads1292DataStream<SPI, NCS, TIM, E, EO>, E, EO> {
        Ads1292DataStream::init(self)
    }
}