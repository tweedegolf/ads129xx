use embedded_hal::blocking::spi as bspi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

use crate::ads1292::data::Ads1292Data;
use crate::ads1292::Ads1292;
use crate::{Command, Result, Ads129xx};

/// Ads1292 Data stream. Used to read data continuously.
pub struct Ads1292DataStream<SPI, NCS, TIM, E, EO>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    ads1292: Ads1292<SPI, NCS, TIM>,
}

impl<SPI, NCS, TIM, E, EO> Ads1292DataStream<SPI, NCS, TIM, E, EO>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    /// Initialize stream, send RDATAC command
    pub fn init(mut ads1292: Ads1292<SPI, NCS, TIM>) -> Result<Self, E, EO> {
        ads1292.cmd(Command::RDATAC)?;
        Ok(Self { ads1292 })
    }

    /// Send SDATAC command, then return wrapped ADS1292
    pub fn into_inner(mut self) -> Result<Ads1292<SPI, NCS, TIM>, E, EO> {
        self.ads1292.cmd(Command::SDATAC)?;
        Ok(self.ads1292)
    }
}

impl<SPI, NCS, TIM, E, EO> Iterator for Ads1292DataStream<SPI, NCS, TIM, E, EO>
where
    SPI: bspi::Transfer<u8, Error = E> + bspi::Write<u8, Error = E>,
    NCS: OutputPin<Error = EO>,
    TIM: CountDown,
{
    type Item = Result<Ads1292Data, E, EO>;
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
