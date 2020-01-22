use core::fmt;

#[derive(Default, Copy, Clone, Debug)]
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

#[derive(Default, Copy, Clone, Debug)]
pub struct GpioStatus {
    /// The status. Bits [4:7] are not used
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

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct ChannelData(pub u8, pub u8, pub u8);

impl ChannelData {
    /// Converts this channel's data into temperature in degrees Celcius (page 19)
    pub fn temp(self) -> i32 {
        let units: i32 = self.into();
        (units - 145_300) / 490 + 25
    }

    pub fn millivolts(self) -> f32 {
        let units: i32 = self.into();
        (units as f32 * 2400.) / 0x800_000 as f32
    }

    pub fn from_millivolts(mv: f32) -> Self {
        ((mv * (0x800_000 as f32) / 2400.) as i32).into()
    }
}

impl From<ChannelData> for i32 {
    fn from(channel_data: ChannelData) -> Self {
        i32::from_be_bytes([channel_data.0, channel_data.1, channel_data.2, 0]) >> 8
    }
}

impl From<i32> for ChannelData {
    fn from(repr: i32) -> Self {
        let [b0, b1, b2, _b3] = (repr << 8).to_be_bytes();
        Self(b0, b1, b2)
    }
}

impl fmt::Display for ChannelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:02x?}, {:02x?}, {:02x?})", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_millivolts() {
        let c1: ChannelData = ChannelData::from_millivolts(10.9883);
        let c2: ChannelData = ChannelData::from_millivolts(-9.32);
        let c3: ChannelData = ChannelData::from_millivolts(-0.1235);
        assert_eq!(c1, ChannelData::from_millivolts(c1.millivolts()));
        assert_eq!(c2, ChannelData::from_millivolts(c2.millivolts()));
        assert_eq!(c3, ChannelData::from_millivolts(c3.millivolts()));
    }

    #[test]
    fn bijective_i32() {
        const C1: i32 = 123;
        const C2: i32 = 3243;
        const C3: i32 = -3243;
        assert_eq!(C1, ChannelData::from(C1).into());
        assert_eq!(C2, ChannelData::from(C2).into());
        assert_eq!(C3, ChannelData::from(C3).into());
    }
}
