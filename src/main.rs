#![allow(dead_code)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f4xx_hal::{i2c::I2c, pac, prelude::*};
use veml7700::drivers::veml7700::Veml7700;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let mut delay = stm32f4xx_hal::delay::Delay::new(cp.SYST, &clocks);
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let mut led = gpioc.pc13.into_push_pull_output();
    let scl = gpiob.pb6.into_alternate_open_drain();
    let sck = gpiob.pb7.into_alternate_open_drain();

    let i2c1 = I2c::new(dp.I2C1, (scl, sck), 100.khz(), clocks);
    let mut veml7700 = Veml7700::new(i2c1).unwrap();
    veml7700.enable().unwrap();
    delay.delay_ms(4u16);

    hprintln!("Hello semihosting world").unwrap();
    let mut lux: f32 = 0.0;
    loop {
        led.set_high();
        lux = veml7700.read_lux().unwrap();
        delay.delay_ms(500u16);
        led.set_low();
        delay.delay_ms(100u16);
        led.set_high();
        hprintln!("lux: {}", lux).unwrap();
        delay.delay_ms(100u16);
        led.set_low();
        delay.delay_ms(500u16);
    }
}
