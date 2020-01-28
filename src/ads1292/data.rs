use core::fmt;

use crate::data::{ChannelData, GpioStatus, LeadOffStatus};

/// Represents a 9-byte data block from the Ads1292
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Ads1292Data {
    pub data: [u8; 9],
}

impl Ads1292Data {
    /// Get the Lead-off status for this data block
    pub fn lead_off_status(&self) -> LeadOffStatus {
        let status = (self.data[0] << 1) | (self.data[1] >> 7);
        LeadOffStatus { status }
    }

    /// Get the GPIO status for this data block
    pub fn gpio_status(&self) -> GpioStatus {
        let status = self.data[1] >> 5;
        GpioStatus { status }
    }

    /// Get the data from channel 1
    pub fn channel_1(&self) -> ChannelData {
        ChannelData(self.data[3], self.data[4], self.data[5])
    }

    /// Get the data from channel 2
    pub fn channel_2(&self) -> ChannelData {
        ChannelData(self.data[6], self.data[7], self.data[8])
    }
}

impl From<[u8; 9]> for Ads1292Data {
    fn from(data: [u8; 9]) -> Self {
        Self { data }
    }
}

impl fmt::Display for Ads1292Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[\n\tLead off:\t{};\n\tGPIO:\t{};\n\tch1:\t{};\n\tch2:\t{}\n]",
            self.lead_off_status(),
            self.gpio_status(),
            self.channel_1(),
            self.channel_2()
        )
    }
}
