#![deny(unsafe_code)]
#![no_main]
#![no_std]
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate panic_halt;
extern crate nb;
extern crate stm32g0xx_hal as hal;

use hal::prelude::*;
use hal::stm32;
use hal::time::Hertz;
use hal::serial::FullConfig;
use rt::entry;
use nb::block;
use sh::hprintln;
use jdy40_rs::{
    Config,
    Jdy40AT,
};


#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();
    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    //rcc.clocks.sys_clk(32.mhz());

    //let buf: [u8; 2] = [0,0];
    let mut delay = dp.TIM14.delay(&mut rcc);
    let mut usart = dp
        .USART1
        .usart(gpioa.pa9, gpioa.pa10, FullConfig::default().baudrate(9600.bps()), &mut rcc)
        .unwrap();

    let mut cs_pin = gpiob.pb2.into_push_pull_output();
    let mut set_pin = gpiob.pb8.into_push_pull_output();
    let mut led = gpioc.pc6.into_push_pull_output();
    let mut jdy = Jdy40AT::new(usart, delay,cs_pin,set_pin).unwrap();

    loop {
        led.toggle().unwrap();
        let mut buf:[u8;1]=[0];;
        jdy.read_buffer(& mut buf);
        hprintln!("receive {}", buf.first());
        //writeln!(usart, "hello {}\r\n", byte).unwrap();
    }
}