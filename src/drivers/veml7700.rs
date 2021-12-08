#![allow(dead_code)]
use embedded_hal::blocking::i2c::{Write, WriteRead};

const I2C_ADDRESS: u8 = 0x10;

// command registers
struct Registers;
impl Registers {
    const ALS_CONF: u8 = 0x00;
    const ALS_WH: u8 = 0x01;
    const ALS_WL: u8 = 0x02;
    const POWER_SAVING: u8 = 0x03;
    const ALS: u8 = 0x04;
    const WHITE: u8 = 0x05;
    const ALS_INT: u8 = 0x06;
}

struct BitFlag;
impl BitFlag {
    // 1 sets shutdown == true
    const ALS_SD: u16 = 0b01;
    const ALS_INT_EN: u16 = 0b10;
    const PSM_EN: u16 = 0b01;
    const INT_TH_LOW: u16 = (1 << 15);
    const INT_TH_HIGH: u16 = (1 << 14);
}

#[derive(Debug)]
pub enum Error<E> {
    BusError(E),
    ConfigurationError,
}

enum Gain {
    OneEighth,
    OneFourth,
    One,
    Two,
}

impl Gain {
    fn as_mask(&self) -> u16 {
        match self {
            Gain::OneEighth => 0b10,
            Gain::OneFourth => 0b11,
            Gain::One => 0b00,
            Gain::Two => 0b01,
        }
    }

    fn as_factor(&self) -> f32 {
        match self {
            Gain::OneEighth => 16.0,
            Gain::OneFourth => 8.00,
            Gain::One => 2.00,
            Gain::Two => 1.00,
        }
    }
}

enum IntTime {
    _25,
    _50,
    _100,
    _200,
    _400,
    _800,
}

impl IntTime {
    fn as_mask(&self) -> u16 {
        match self {
            IntTime::_25 => 0b1100,
            IntTime::_50 => 0b1000,
            IntTime::_100 => 0b0000,
            IntTime::_200 => 0b0001,
            IntTime::_400 => 0b0010,
            IntTime::_800 => 0b0011,
        }
    }

    fn as_factor(&self) -> f32 {
        match self {
            IntTime::_25 => 0.1152,
            IntTime::_50 => 0.0576,
            IntTime::_100 => 0.0288,
            IntTime::_200 => 0.0144,
            IntTime::_400 => 0.0072,
            IntTime::_800 => 0.0036,
        }
    }
}

pub struct Veml7700<I> {
    i2c: I,
    gain: Gain,
    time: IntTime,
    conf: u16,
}

impl<I, E> Veml7700<I>
where
    I: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I) -> Result<Self, Error<E>> {
        // default configuration with sd == 1
        let gain = Gain::OneEighth;
        let time = IntTime::_100;
        let conf: u16 = (BitFlag::ALS_SD << 0) | (gain.as_mask() << 11) | (time.as_mask() << 6);
        let mut sensor = Veml7700 {
            i2c,
            gain,
            time,
            conf,
        };
        sensor.set_config(conf)?;
        Ok(sensor)
    }

    pub fn enable(&mut self) -> Result<(), Error<E>> {
        let conf: u16 = self.conf & !(BitFlag::ALS_SD);
        self.set_config(conf)?;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), Error<E>> {
        let conf: u16 = self.conf | BitFlag::ALS_SD;
        self.set_config(conf)?;
        Ok(())
    }

    pub fn read_lux(&mut self) -> Result<f32, Error<E>> {
        let resolution = 0.0036 * self.gain.as_factor() * self.time.as_factor();
        let light_raw = self.read_raw()? as f32;
        Ok(resolution * light_raw * 1000.0)
    }

    fn set_config(&mut self, conf: u16) -> Result<(), Error<E>> {
        self.write(Registers::ALS_CONF, conf)?;
        self.conf = conf;
        Ok(())
    }

    fn read_raw(&mut self) -> Result<u16, Error<E>> {
        Ok(self.read(Registers::ALS)?)
    }

    fn read(&mut self, register: u8) -> Result<u16, Error<E>> {
        let buf = &mut [0u8; 2];
        self.i2c
            .write_read(I2C_ADDRESS, &[register], buf)
            .map_err(Error::BusError)?;
        Ok((buf[0] as u16) | (buf[1] as u16))
    }

    fn write(&mut self, register: u8, command: u16) -> Result<(), Error<E>> {
        self.i2c
            .write(
                I2C_ADDRESS,
                &[register, command as u8, (command >> 8) as u8],
            )
            .map_err(Error::BusError)?;
        Ok(())
    }
}
