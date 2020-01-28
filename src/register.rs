use bitfield::bitfield;

/// Read / write-able registers
///
/// Table 14 page 39 of specification.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
#[repr(u8)]
pub enum SampleRate {
    Sps125 = 0b000,
    Sps250 = 0b001,
    Sps500 = 0b010,
    KSps1 = 0b011,
    KSps2 = 0b100,
    KSps4 = 0b101,
    KSps8 = 0b110,
    Unknown = 0b111,
}

impl From<u8> for SampleRate {
    fn from(x: u8) -> Self {
        use SampleRate::*;
        match x {
            0b000 => Sps125,
            0b001 => Sps250,
            0b010 => Sps500,
            0b011 => KSps1,
            0b100 => KSps2,
            0b101 => KSps4,
            0b110 => KSps8,
            _ => Unknown,
        }
    }
}

impl From<SampleRate> for u8 {
    fn from(x: SampleRate) -> Self {
        x as Self
    }
}

bitfield! {
    /// Configuration for the register that configures each ADC channel sample rate.
    pub struct Conf1(u8);

    /// The single shot conversion mode, otherwise use a continuous conversion mode.
    pub single_shot, set_single_shot: 7;
    /// The oversampling rate used by all channels.
    pub u8, from into SampleRate, oversampling, set_oversampling: 2, 0;
}

bitfield! {
    /// Configuration for the register that configures the test signal, clock, reference and LOFF buffer.
    pub struct Conf2(u8);

    /// Power down the lead-off comparators.
    pub pdb_loff_comp, set_pdb_loff_comp: 6;
    /// Powers down the internal reference buffer so that the external reference can be used.
    pub pdb_refbuf, set_pdb_refbuf: 5;
    /// Enable 4.033v reference, otherwise use the 2.42v reference.
    pub vref_4v, set_vref_4v: 4;
    /// Determines if the internal oscillator signal is connected to the CLK pin when an internal oscillator is used.
    pub clk_en, set_clk_en: 3;
    /// Determines whether the test signal is turned on or off.
    pub int_test, set_int_test: 1;
    /// Determines the test signal frequency.
    pub test_freq, set_test_freq: 0;
}

#[derive(Debug)]
#[repr(u8)]
pub enum LeadOffCurrentMagnitude {
    C6nA = 0b00,
    C22nA = 0b01,
    C6uA = 0b10,
    C22uA = 0b11,
    Unknown = 0b111,
}

impl From<u8> for LeadOffCurrentMagnitude {
    fn from(x: u8) -> Self {
        use LeadOffCurrentMagnitude::*;
        match x {
            0b00 => C6nA,
            0b01 => C22nA,
            0b10 => C6uA,
            0b11 => C22uA,
            _ => Unknown,
        }
    }
}

impl From<LeadOffCurrentMagnitude> for u8 {
    fn from(x: LeadOffCurrentMagnitude) -> Self {
        x as Self
    }
}

bitfield! {
    /// Configuration for the register that configures the lead-off detection operation.
    pub struct Loff(u8);

    /// Power down the lead-off comparators.
    pub comp_th, set_comp_th: 7, 5;
    /// Powers down the internal reference buffer so that the external reference can be used.
    pub u8, from into LeadOffCurrentMagnitude, ilead_off, set_ilead_off: 3, 2;
    /// Selects ac (true) or dc (false) lead-off
    pub flead_off, set_flead_off: 0;
}

bitfield! {
    /// Configuration for the register that selects the positive and negative side from each channel for lead-off detection.
    pub struct LoffSense(u8);

    /// Controls the direction of the current used for lead-off derivation for channel 2
    pub flip2, set_flip2: 5;
    /// Controls the direction of the current used for lead-off derivation for channel 1
    pub flip1, set_flip1: 4;
    /// Controls the selection of negative input from channel 2 for lead-off detection
    pub loff2n, set_loff2n: 3;
    /// Controls the selection of positive input from channel 2 for lead-off detection
    pub loff2p, set_loff2p: 2;
    /// Controls the selection of negative input from channel 1 for lead-off detection
    pub loff1n, set_loff1n: 1;
    /// Controls the selection of positive input from channel 1 for lead-off detection
    pub loff1p, set_loff1p: 0;
}

#[derive(Debug)]
#[repr(u8)]
pub enum GainSetting {
    G6 = 0b000,
    G1 = 0b001,
    G2 = 0b010,
    G3 = 0b011,
    G4 = 0b100,
    G8 = 0b101,
    G12 = 0b110,
    Unknown = 0b111,
}

impl From<u8> for GainSetting {
    fn from(x: u8) -> Self {
        use GainSetting::*;
        match x {
            0b000 => G6,
            0b001 => G1,
            0b010 => G2,
            0b011 => G3,
            0b100 => G4,
            0b101 => G8,
            0b110 => G12,
            _ => Unknown,
        }
    }
}

impl From<GainSetting> for u8 {
    fn from(x: GainSetting) -> Self {
        x as Self
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum InputSelection {
    /// Normal electrode input (default)
    NormalElectrodeInput = 0b0000,
    /// Input shorted (for offset measurements)
    InputShorted = 0b0001,
    /// RLD_MEASURE
    RldMeasure = 0b0010,
    /// MVDD for supply measurement
    MVDD = 0b0011,
    /// Temperature sensor
    TemperatureSensor = 0b0100,
    /// Test signal
    TestSignal = 0b0101,
    /// RLD_DRP (positive input is connected to RLDIN)
    RldDrp = 0b0110,
    /// RLD_DRM (negative input is connected to RLDIN)
    RldDrm = 0b0111,
    /// RLD_DRPM (both positive and negative inputs are connected to RLDIN)
    RldDrpm = 0b1000,
    /// Route IN3P and IN3N to channel 1 inputs
    Channel3 = 0b1001,
    /// The value was something unknown, might be anything other than the above    
    Unknown = 0b1111,
}

impl From<u8> for InputSelection {
    fn from(x: u8) -> Self {
        use InputSelection::*;
        match x {
            0b0000 => NormalElectrodeInput,
            0b0001 => InputShorted,
            0b0010 => RldMeasure,
            0b0011 => MVDD,
            0b0100 => TemperatureSensor,
            0b0101 => TestSignal,
            0b0110 => RldDrp,
            0b0111 => RldDrm,
            0b1000 => RldDrpm,
            0b1001 => Channel3,
            _ => Unknown,
        }
    }
}

impl From<InputSelection> for u8 {
    fn from(x: InputSelection) -> Self {
        x as Self
    }
}

bitfield! {
    /// Configuration for the register that configures the power mode, PGA gain, and multiplexer settings channels.
    pub struct ChannelSettings(u8);

    /// Power down the channel.
    pub pd, set_pd: 7;
    /// Determines the PGA gain setting for the channel.
    pub u8, from into GainSetting, gain, set_gain: 6, 4;
    /// Determines the channel input selection.
    pub u8, from into InputSelection, mux, set_mux: 3, 0;
}

#[derive(Debug)]
#[repr(u8)]
pub enum ChopFrequency {
    FmodDiv16 = 0b00,
    FmodDiv2 = 0b10,
    FmodDiv4 = 0b11,
    Unknown = 0b01,
}

impl From<u8> for ChopFrequency {
    fn from(x: u8) -> Self {
        use ChopFrequency::*;
        match x {
            0b00 => FmodDiv16,
            0b10 => FmodDiv2,
            0b11 => FmodDiv4,
            _ => Unknown,
        }
    }
}

impl From<ChopFrequency> for u8 {
    fn from(x: ChopFrequency) -> Self {
        x as Self
    }
}

bitfield! {
    /// Configuration for the register that controls the selection of the positive and negative signals from each channel for right leg drive derivation.
    pub struct RLDSenseSelection(u8);

    /// Determines the PGA chop frequency.
    pub u8, from into ChopFrequency, chop, set_chop: 7, 6;
    /// Enable the RLD buffer power.
    pub pdb_rld, set_pbd_rld: 5;
    /// Enable the RLD lead-off sense function.
    pub rld_loff_sense, set_rld_loff_sense: 4;

    /// Controls the selection of negative inputs from channel 2 for right leg drive derivation.
    pub rld2n, set_rld2n: 3;
    /// Controls the selection of positive inputs from channel 2 for right leg drive derivation.
    pub rld2p, set_rld2p: 2;
    /// Controls the selection of negative inputs from channel 1 for right leg drive derivation.
    pub rld1n, set_rld1n: 1;
    /// Controls the selection of positive inputs from channel 1 for right leg drive derivation.
    pub rld1p, set_rld1p: 0;
}

bitfield! {
    /// Configuration for the register that controls the respiration and calibration functionality.
    pub struct RespConf2(u8);

    /// Enables offset calibration
    pub calib_on, set_calib_on: 7;
    /// Controls the respiration control frequency when RESP_CTRL = 0.
    ///
    /// **Warning**: this bit must be written with '1' for the ADS1291 and ADS1292.
    pub resp_freq_64khz, set_resp_freq_64khz: 2;
    /// Determines the RLDREF signal source.
    /// Can be fed externally (false : 0) or internally by using (AVDD – AVSS) / 2 (true : 1).
    pub rldref_int, set_rldref_int: 1;
}
