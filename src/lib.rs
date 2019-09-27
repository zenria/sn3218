//! SN3218 6 channel LED driver for use with [`embedded-hal`]
//!
//! This LED driver is used on the Raspberry GFX HAT by Pimoroni.
//!
//! It is a brutal translation of https://github.com/pimoroni/sn3218/blob/master/library/sn3218.py
//!
//! The API cannot be considered as stable and can break at any time.
//!

use embedded_hal::blocking::i2c::Write;

pub struct SN3218<T: Write> {
    i2c: T,
    gamma_table: [u8; 256],
}

const I2C_ADDRESS: u8 = 0x54;
const CMD_ENABLE_OUTPUT: u8 = 0x00;
const CMD_SET_PWM_VALUES: u8 = 0x01;
const CMD_ENABLE_LEDS: u8 = 0x13;
const CMD_UPDATE: u8 = 0x16;
const CMD_RESET: u8 = 0x17;

const BUF_CMD_ENABLE_ENABLE: [u8; 1] = [0x01];
const BUF_CMD_ENABLE_DISABLE: [u8; 1] = [0x00];
const BUF_CMD_255: [u8; 1] = [0xFF];

///
impl<T: Write> SN3218<T> {
    pub fn new(i2c: T) -> Self {
        let mut gamma_table: [u8; 256] = [0; 256];
        for i in 0..256 {
            gamma_table[i] = (255f64.powf(i as f64 / 255f64)) as u8;
        }
        Self { i2c, gamma_table }
    }

    pub fn enable(&mut self) -> Result<(), T::Error> {
        self.i2c
            .cmd_write(CMD_ENABLE_OUTPUT, &BUF_CMD_ENABLE_ENABLE)
    }

    pub fn disable(&mut self) -> Result<(), T::Error> {
        self.i2c
            .cmd_write(CMD_ENABLE_OUTPUT, &BUF_CMD_ENABLE_DISABLE)
    }

    pub fn reset(&mut self) -> Result<(), T::Error> {
        self.i2c.cmd_write(CMD_RESET, &BUF_CMD_255)
    }

    // TODO we want a higher level API
    pub fn enable_leds(&mut self, mask: u32) -> Result<(), T::Error> {
        let buf = [
            (mask & 0x3F) as u8,
            ((mask >> 6) & 0x3F) as u8,
            ((mask >> 12) & 0x3F) as u8,
        ];
        self.i2c.cmd_write(CMD_ENABLE_LEDS, &buf)?;
        self.i2c.cmd_write(CMD_UPDATE, &BUF_CMD_255)
    }

    pub fn output(&mut self, values: &[u8]) -> Result<(), T::Error> {
        if values.len() != 18 {
            // TODO
            panic!("values must be 18 length")
        }
        let mut buf = [0u8; 18];
        for i in 0..18 {
            buf[i] = self.gamma_table[values[i] as usize];
        }
        self.i2c.cmd_write(CMD_SET_PWM_VALUES, &buf)?;
        self.i2c.cmd_write(CMD_UPDATE, &BUF_CMD_255)
    }
}

// internal trait to help sending commands to our device
trait SN3218CmdWrite<T: Write> {
    fn cmd_write(&mut self, command: u8, buf: &[u8]) -> Result<(), T::Error>;
}

impl<T: Write> SN3218CmdWrite<T> for T {
    fn cmd_write(&mut self, command: u8, buffer: &[u8]) -> Result<(), T::Error> {
        // *r is really shitty
        let to_send: Vec<u8> = [command].iter().chain(buffer).map(|r| *r).collect();
        self.write(I2C_ADDRESS, &to_send)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
