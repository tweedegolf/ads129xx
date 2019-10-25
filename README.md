# ADS129xx
Driver crate for the Texas Instruments ADS1292 24-bit 2-channel low-power analog front end for ECG applications.

This initial version supports the ADS1292 (for the most part), but it's a goal to support the ADS1291 and ADS1292R as well.

Contributions welcome!

## Functionality

- Send commands:
```rust
ads1292.cmd(Command::START)?;
```
- Write to registers
```rust
ads1292.write_register(Register::CONFIG1, 0b001)?;
```
- Read from registers
```rust
let lead_off_status = ads1292.read_register(Register::LOFF_STAT)?;
```
- Read data
```rust
let data = ads1292.read_data()?;
```
- Read data continuously
```rust
// Opens stream, sends RDATAC command to ads
let data_stream = ads1292.data_stream()?;

let mut buf = [[Ads1292Data::default(); 2]; 2000];

for i in buf.iter_mut() {
    while !data_ready() {}
    *i = iter.next().unwrap().unwrap();
}
// Don't forget to close; this will send the SDATAC command to the ads
data_stream.into_inner();
```

## TODO's

- [ ] Support ADS1292R
- [ ] Support ADS1291
- [ ] Non-blockingly wait after sending SPI commands
- [ ] Documentation