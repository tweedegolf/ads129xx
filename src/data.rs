use core::fmt;

pub struct LeadOffStatus {
    /// The status. Bits [5:7] are unused
    pub status: u8,
}

impl LeadOffStatus {
    /// Clock divider selection
    pub fn clk_div(&self) -> u8 {
        self.status >> 6 & 1
    }

    /// RLD Lead-off status
    pub fn rld_stat(&self) -> bool {
        self.status & 1 << 4 > 0
    }

    /// Channel 2 negative electrode status
    pub fn in2n_off(&self) -> bool {
        self.status & 1 << 3 > 0
    }

    /// Channel 2 positive electrode status
    pub fn in2p_off(&self) -> bool {
        self.status & 1 << 2 > 0
    }

    /// Channel 1 negative electrode status
    pub fn in1n_off(&self) -> bool {
        self.status & 1 << 1 > 0
    }

    /// Channel 1 positive electrode status
    pub fn in1p_off(&self) -> bool {
        self.status & 1 > 0
    }
}

impl fmt::Display for LeadOffStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[clk_div: {}; rld_stat: {}; in2n_off: {}; in2p_off: {}; in1n_off: {}; in1p_off: {}]",
            self.clk_div(),
            self.rld_stat(),
            self.in2n_off(),
            self.in2p_off(),
            self.in1n_off(),
            self.in1p_off(),
        )
    }
}

pub struct GpioStatus {
    /// The status. Bits [4:7] are not used/
    pub status: u8,
}

impl GpioStatus {
    /// GPIO 2 control
    pub fn gpio_c_2(&self) -> bool {
        self.status & 1 << 3 > 0
    }

    /// GPIO 1 control
    pub fn gpio_c_1(&self) -> bool {
        self.status & 1 << 2 > 0
    }

    /// GPIO 2 data
    pub fn gpio_d_2(&self) -> bool {
        self.status & 1 << 1 > 0
    }

    /// GPIO 1 data
    pub fn gpio_d_1(&self) -> bool {
        self.status & 1 > 0
    }
}

impl fmt::Display for GpioStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[gpio_c_2: {}; gpio_c_1: {}; gpio_d_2: {}; gpio_d_1: {}]",
            self.gpio_c_2(),
            self.gpio_c_1(),
            self.gpio_d_2(),
            self.gpio_d_1()
        )
    }
}

#[derive(Copy, Clone, Default)]
pub struct ChannelData(pub u8, pub u8, pub u8);

impl ChannelData {
    /// Converts this channel's data into temperature in degrees Celcius (page 19)
    pub fn temp(self) -> i32 {
        let microvolts: i32 = self.into();
        (microvolts - 145_300) / 490 + 25
        // microvolts
    }
}

impl From<ChannelData> for i32 {
    fn from(channel_data: ChannelData) -> Self {
        unsafe { core::mem::transmute([0, channel_data.0, channel_data.1, channel_data.2]) }
    }
}

impl fmt::Display for ChannelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:02x?}, {:02x?}, {:02x?})", self.0, self.1, self.2)
    }
}

#[derive(Copy, Clone, Default)]
pub struct Ads1292Data {
    pub data: [u8; 9],
}

impl Ads1292Data {
    pub fn lead_off_status(&self) -> LeadOffStatus {
        let status = (self.data[0] << 1) | (self.data[1] >> 7);
        LeadOffStatus { status }
    }

    pub fn gpio_status(&self) -> GpioStatus {
        let status = self.data[1] >> 5;
        GpioStatus { status }
    }

    pub fn channel_1(&self) -> ChannelData {
        ChannelData(self.data[3], self.data[4], self.data[5])
    }

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
