#![no_std]
#![no_main]

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

pub enum Error<E> {
    I2C(E),
    INVALID_PARAMETER,
}

#[derive(Debug, Copy, Clone, Default)]
struct Config {
    bits: u8,
}

#[derive(Debug)]
pub enum MODE {
    CONTINOUS,
    ONE_SHOT,
}

struct Register;
impl Register {
    const TEMPERATURE: u8 = 0xAA;
    const ACCESS_TH: u8 = 0xA1;
    const ACCESS_TL: u8 = 0xA2;
    const ACCESS_CONFIG: u8 = 0xAC;
    const START_CONVERT: u8 = 0xEE;
    const STOP_CONVERT: u8 = 0x22;
}

struct ConfigRegBits;

impl ConfigRegBits {
    const DONE: u8 = 0b1000_0000;
    const THF: u8 = 0b0100_0000;
    const TLF: u8 = 0b0010_0000;
    const NVB: u8 = 0b0001_0000;
    const RESERVED0: u8 = 0b0000_1000;
    const RESERVED1: u8 = 0b0000_0100;
    const POL: u8 = 0b0000_0010;
    const ONE_SHOT: u8 = 0b0000_0001;
}

#[derive(Debug)]
pub struct ds1621<I2C> {
    i2c: I2C,
    addr: u8,
    mode: MODE,
}

const ADDR_DEFAULT: u8 = 0x4A;

impl<I2C, E> ds1621<I2C>
where
    I2C: Read<Error = E> + Write<Error = E>,
{
    pub fn new_default(i2c: I2C) -> Self {
        ds1621 {
            i2c,
            addr: ADDR_DEFAULT,
            mode: MODE::CONTINOUS, //By default set CONTiNOUS MODE
        }
    }

    pub fn new(i2c: I2C, a_addr: u8) -> Self {
        ds1621 {
            i2c,
            addr: a_addr,
            mode: MODE::CONTINOUS, //By default set CONTiNOUS MODE
        }
    }

    pub fn set_convert_mode(&mut self, a_mode: MODE) -> Result<(), Error<E>> {
        //Lire le contenu du registre de configuration
        match self.read_config() {
            Ok(mut conf_val) => {
                //Ajuster le bit de mode de convertion
                match a_mode {
                    MODE::CONTINOUS => {
                        conf_val |= ConfigRegBits::ONE_SHOT;
                    }
                    MODE::ONE_SHOT => {
                        conf_val &= 0xFE;
                    }
                }

                //Ecrire la config ajustee
                return self.write_config(conf_val);
            }
            Err(e) => {
                return Err(Error::I2C(e));
            }
        }
    }
}

impl<I2C, E> ds1621<I2C>
where
    I2C: Read<Error = E>,
{
    pub fn read_config(&mut self) -> Result<u8, E> {
        let mut u8rd_buff: [u8; 1] = [0; 1];

        match self.i2c.read(self.addr, &mut u8rd_buff) {
            Ok(()) => {
                return Ok(u8rd_buff[0]);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

impl<I2C, E> ds1621<I2C>
where
    I2C: Write<Error = E>,
{
    pub fn write_config(&mut self, a_config: u8) -> Result<(), Error<E>> {
        match self.i2c.write(self.addr, &[a_config]) {
            Ok(()) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Error::I2C(e));
            }
        }
    }

    pub fn start_convert(&mut self) -> Result<(), E> {
        self.i2c.write(self.addr, &[Register::START_CONVERT])
    }

    pub fn stop_convert(&mut self) -> Result<(), E> {
        self.i2c.write(self.addr, &[Register::STOP_CONVERT])
    }

    pub fn write_high_temperature(&mut self, a_temp: f32) -> Result<(), Error<E>> {
        self.write_threshold_temperature(a_temp, Register::ACCESS_TH)
    }

    pub fn write_low_temperature(&mut self, a_temp: f32) -> Result<(), Error<E>> {
        self.write_threshold_temperature(a_temp, Register::ACCESS_TL)
    }

    pub fn write_threshold_temperature(&mut self, a_temp: f32, reg: u8) -> Result<(), Error<E>> {
        if reg != Register::ACCESS_TL && reg != Register::ACCESS_TH {
            return Err(Error::INVALID_PARAMETER);
        }

        let mut wr_buff: [u8; 3] = [reg as u8, a_temp as u8, 0];

        //Conserver uniquement la partie entiere
        let round = a_temp as u32;

        if (round as f32 - a_temp).ge(&0.5_f32) == true {
            wr_buff[2] = 0x80;
        }

        //Ecrire la commande
        match self.i2c.write(self.addr, &wr_buff) {
            Ok(()) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Error::I2C(e));
            }
        }
    }
}

impl<I2C, E> ds1621<I2C>
where
    I2C: WriteRead<Error = E>,
{
    pub fn read_temperature(&mut self) -> Result<f32, E> {
        let mut raw_read: [u8; 2] = [0; 2];

        match self
            .i2c
            .write_read(self.addr, &[Register::TEMPERATURE], &mut raw_read)
        {
            Ok(()) => {
                let mut temp: f32 = raw_read[0] as f32;
                if (raw_read[1] != 0) {
                    temp += 0.5;
                }

                return Ok(temp);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
