#![deny(unsafe_code)]
#![no_std]
#[macro_use]
extern crate cortex_m_semihosting as sh;
use embedded_hal::blocking::{
    delay::DelayMs,
};
use nb::block;
use embedded_hal::serial::{Read, Write};
use sh::hprintln;
use embedded_hal::digital::v2::OutputPin;
use cortex_m::asm::delay;

pub const OK_RESPONSE: [u8; 4] = *b"OK\r\n";

/// Error.
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Read error
    Read,
    /// Write error
    Write,
    /// Invalid baud rate
    InvalidBaudRate,
    /// Invalid channel error
    InvalidChannel,
}

// speed connection
#[derive(Debug, Clone, Copy)]
pub enum JdySpeed {
    Bods1200,
    Bods2400,
    Bods4800,
    Bods9600,
    Bods14400,
    Bods19200,
}
impl JdySpeed {
    pub fn get_value(&self) -> &[u8]{
        match(&self){
            JdySpeed::Bods1200 => b"AT+BAUD1\r\n",
            JdySpeed::Bods2400 => b"AT+BAUD2\r\n",
            JdySpeed::Bods4800 => b"AT+BAUD3\r\n",
            JdySpeed::Bods9600 => b"AT+BAUD4\r\n",
            JdySpeed::Bods14400 => b"AT+BAUD5\r\n",
            JdySpeed::Bods19200 => b"AT+BAUD6\r\n",
        }
    }
}

/// Power lvl
#[derive(Debug, Clone, Copy)]
pub enum JdyPower {
    /// -25db
    PowerSuperLow,
    /// -15db
    PowerLowest,
    /// -5db
    PowerLow,
    Power0db,
    Power3db,
    Power6db,
    Power9db,
    Power10db,
    Power12db,
}
impl JdyPower{
    pub fn get_value(&self) -> &[u8]{
        match(&self){
            JdyPower::PowerSuperLow => b"AT+POWE0\r\n",
            JdyPower::PowerLowest => b"AT+POWE1\r\n",
            JdyPower::PowerLow => b"AT+POWE2\r\n",
            JdyPower::Power0db => b"AT+POWE3\r\n",
            JdyPower::Power3db => b"AT+POWE4\r\n",
            JdyPower::Power6db => b"AT+POWE5\r\n",
            JdyPower::Power9db => b"AT+POWE6\r\n",
            JdyPower::Power10db => b"AT+POWE7\r\n",
            JdyPower::Power12db => b"AT+POWE9\r\n",
        }
    }
}

/// CLSS type
#[derive(Debug, Clone, Copy)]
pub enum JdyMode {
    /// serial port transceiver
    ModeA0,
    /// Remote controller or IO key indicator light(Transmitting terminal)
    ModeCO,
    /// Remote controller or IO key without indicator light (Transmitting terminal)
    ModeC1,
    /// IO is low level at normal level, high level after receiving signal and low level after delay 30mS
    ModeC2,
    /// IO is high level at normal level, low level after receiving signal and high level after delay 30mS
    ModeC3,
    /// IO is low level at normal level, receives pressed signal of high level and receives lift signal low level
    ModeC4,
    /// The IO level is reversed when IO receives the pressed signal.
    ModeC5,

}

impl JdyMode {
    pub fn get_value(&self) -> &[u8]{
        match(&self){
            JdyMode::ModeA0 => b"AT+CLSSA0\r\n",
            JdyMode::ModeCO => b"AT+CLSSC0\r\n",
            JdyMode::ModeC1 => b"AT+CLSSC1\r\n",
            JdyMode::ModeC2 => b"AT+CLSSC2\r\n",
            JdyMode::ModeC3 => b"AT+CLSSC3\r\n",
            JdyMode::ModeC4 => b"AT+CLSSC4\r\n",
            JdyMode::ModeC5 => b"AT+CLSSC5\r\n",
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Config {
    speed: JdySpeed,
    power: JdyPower,
    mode: JdyMode,
    network: [u8;4],
    device: [u8;4],
    channel: [u8;3],
}

impl Default for Config {
    fn default() -> Self {
        Config {
            speed: JdySpeed::Bods9600,
            power: JdyPower::Power6db,
            mode: JdyMode::ModeA0,
            network: [6,5,4,3],
            device: [0,0,1,0],
            channel: [0,0,7],
        }
    }
}

pub struct Jdy40AT<UART, D, CS, SET > {
    serial: UART,
    config: Config,
    cs: CS,
    set: SET,
    delay: D,
}

impl<UART, CS, SET, D, E> Jdy40AT<UART, D, CS, SET> where
    UART: Read<u8> + Write<u8>,
    CS: OutputPin<Error = E>,
    SET: OutputPin<Error = E>,
    D: DelayMs<u16>,
{

    pub fn new(serial: UART, delay: D, cs: CS, set: SET) -> Result<Self, E> {
        let mut dev = Self {
            serial,
            config: Config::default(),
            cs,
            set,
            delay,
        };

        Ok(dev)
    }

    /// Init with default config
    pub fn init(& mut self) -> Result<(), E>{
        self.set_config(self.config).unwrap_or_default();
        Ok(())
    }

    /// Check responds to "AT" query.
    pub fn is_ok(&mut self) -> bool {
        let mut n = 0;
        let mut buffer = [0u8; 4];
        while n < 4 {
            if let Ok(ch) = block!(self.serial.read()) {
                buffer[n] = ch;
                n += 1;
            }
        }
        buffer == OK_RESPONSE
    }

    /// Write entire buffer to serial port
    pub fn write_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        for ch in buffer {
            let _ = block!(self.serial.write(*ch));
        }
        Ok(())
    }

    /// Read entire buffer from serial port
    pub fn read_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let mut n = 0;
        while n < buffer.len() {
            if let Ok(ch) = block!(self.serial.read()) {
                buffer[n] = ch;
                n += 1;
            }
        }
        Ok(())
    }

    pub fn send_command(& mut self , val: &[u8]) -> Result<(), Error> {
        for ch in val {
            let _ = block!(self.serial.write(*ch));
        }

        if !self.is_ok() {
            return Err(Error::Write);
        }
        Ok(())
    }

    pub fn set_config(& mut self, config: Config) -> Result<(), E> {
        self.cs.set_low().ok();
        self.set.set_low().ok();

        self.delay.delay_ms(2);
        self.send_command(& config.power.get_value()).unwrap();
        self.send_command(& config.speed.get_value()).unwrap();
        self.send_command(& config.mode.get_value()).unwrap();

        self.send_command(&config.network).unwrap();
        self.send_command(&config.device).unwrap();
        self.send_command(&config.channel).unwrap();

        self.cs.set_high().ok();
        self.set.set_high().ok();
        self.delay.delay_ms(2);
        Ok(())
    }

}