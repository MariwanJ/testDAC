use stm32f7::stm32f767::dfsdm::ch;
use stm32f7xx_hal::pac::{ self, DAC };

/// Marker trait for valid DAC pins
/// Implement it for PA4 and PA5 in Analog mode
use stm32f7xx_hal::gpio::gpioa::{ PA4, PA5 };
use stm32f7xx_hal::gpio::Analog;
use core::error;
use core::marker::PhantomData;

pub struct Ch1;
pub struct Ch2;

#[derive(Copy, Clone, Debug)]
pub enum DACState {
    Reset = 0x00, // DAC not yet initialized or disabled
    Ready = 0x01, // DAC initialized and ready for use
    Busy = 0x02, // DAC internal processing is ongoing
    Timeout = 0x03, // DAC timeout state
    Error = 0x04, // DAC error state
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LockType {
    UNLOCKED = 0x00,
    LOCKED = 0x01,
}
#[derive(Copy, Clone, Debug)]
pub enum DacAlign {
    DacAlign12bR = 0x00000000,
    DacAlign12bL = 0x00000004,
    DacAlign8bR = 0x00000008,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum STATUStype {
    HAL_OK = 0x00,
    HAL_ERROR = 0x01,
    HAL_BUSY = 0x02,
    HAL_TIMEOUT = 0x03,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DacTrigger {
    None,
    Timer2,
    Timer4,
    Timer5,
    Timer6,
    Timer7,
    Timer8,
    ExtTrig,
    Software,
    // More timers or external triggers can be added
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OUTPUT_BUF {
    Disabled = 0,
    Enabled = 1,
}

pub trait DacHW<PIN> {
    fn new(dac: DAC, rcc: &pac::RCC, pin: PIN) -> Self;
    fn enable(&mut self) -> STATUStype;
    fn disable(&mut self) -> STATUStype;
    fn start(&mut self, ch:u8) -> STATUStype;
    fn write(&mut self, value: u16) -> STATUStype;
    fn set_trigger(&mut self, trigger: DacTrigger) -> STATUStype;
    fn buffer(&mut self, buf: OUTPUT_BUF) -> STATUStype;
    fn lock(&mut self) -> STATUStype;
    fn unlock(&mut self) -> STATUStype;
}
pub struct Dac<PIN> where PIN: DACInstance {
    dac: DAC,
    pin: PIN,
    ch_ph: PhantomData<PIN::Channel>,
    ch: u8,
    state: DACState,
    locked: LockType,
    trigger: DacTrigger,
    align: DacAlign,
    buffer: OUTPUT_BUF,
}

trait DACInstance {
    type Channel;
    const CHANNEL: u8;
    const EN_BIT: u32;
    const SWTRIG_BIT: u32;
}
impl DACInstance for PA4<Analog> {
    type Channel = Ch1;
    const CHANNEL: u8 = 1;
    const EN_BIT: u32 = 0; // EN1
    const SWTRIG_BIT: u32 = 0; // SWTRIG1
}
impl DACInstance for PA5<Analog> {
    type Channel = Ch2;
    const CHANNEL: u8 = 2;
    const EN_BIT: u32 = 16; // EN2
    const SWTRIG_BIT: u32 = 1; // SWTRIG2
}

impl<PIN> DacHW<PIN> for Dac<PIN> where PIN: DACInstance {
    fn new(dac: DAC, rcc: &pac::RCC, pin: PIN) -> Self {
        //initialize clock:
        rcc.apb1enr.modify(|r, w| unsafe {
            // Set the DAC clock enable bit (bit 29)
            let new_bits = r.bits() | (1 << 29); // Set DACEN bit
            w.bits(new_bits) // Write the new bits back to the register
        });

        // Set the GPIOA clock enable bit
        rcc.ahb1enr.modify(|r, w| unsafe {
            let new_bits = r.bits() | (0x1 << 0); // Set the GPIOA clock enable bit
            w.bits(new_bits)
        });
        let tmpreg = rcc.ahb1enr.read().bits();
        if (tmpreg & (0x1 << 0)) == 0 {
            // Check if the GPIOAEN bit is still 0
            //panic!("Failed to initialize GPIOA"); // TODO: Consider how you want to handle this failure
            
        }
        // enable DAC clock here
        Self {
            dac,
            pin,
            ch_ph: PhantomData,
            ch: 0,
            state: DACState::Reset,
            locked: LockType::UNLOCKED,
            trigger: DacTrigger::None,
            align: DacAlign::DacAlign12bR,
            buffer: OUTPUT_BUF::Disabled,
        }
    }
    fn enable(&mut self) -> STATUStype {
        self.dac.cr.modify(|r, w| unsafe { w.bits(r.bits() | (1u32 << PIN::EN_BIT)) });
        STATUStype::HAL_OK
    }
    fn disable(&mut self) -> STATUStype {
        self.dac.cr.modify(|_, w| {
            if PIN::EN_BIT == 0 { w.en1().clear_bit() } else { w.en2().clear_bit() }
        });
        STATUStype::HAL_OK
    }
    fn start(&mut self, ch:u8) -> STATUStype {
        self.ch = ch;
        self.lock();
        self.state = DACState::Busy;

        self.enable();

        self.dac.swtrigr.write(|w| unsafe { w.bits(1u32 << PIN::SWTRIG_BIT) });

        self.state = DACState::Ready;
        self.unlock();
        STATUStype::HAL_OK
    }

    fn write(&mut self, data: u16) -> STATUStype {
        if self.ch == 1 {
            match self.align {
                DacAlign::DacAlign12bR => {
                    debug_assert!(data <= 0x0fff);
                    self.dac.dhr12r1.write(|w| unsafe { w.bits(data as u32) });
                }
                DacAlign::DacAlign12bL => {
                    debug_assert!(data <= 0x0fff);
                    self.dac.dhr12l1.write(|w| unsafe { w.bits((data as u32) << 4) });
                }
                DacAlign::DacAlign8bR => {
                    debug_assert!(data <= 0xff);
                    self.dac.dhr8r1.write(|w| unsafe { w.bits(data as u32) });
                }
                _ => unreachable!(),
            }
        } else if self.ch == 2 {
            match self.align {
                DacAlign::DacAlign12bR => {
                    debug_assert!(data <= 0x0fff);
                    self.dac.dhr12r2.write(|w| unsafe { w.bits(data as u32) });
                }
                DacAlign::DacAlign12bL => {
                    debug_assert!(data <= 0x0fff);
                    self.dac.dhr12l2.write(|w| unsafe { w.bits((data as u32) << 4) });
                }
                DacAlign::DacAlign8bR => {
                    debug_assert!(data <= 0xff);
                    self.dac.dhr8r2.write(|w| unsafe { w.bits(data as u32) });
                }
                _ => {
                    unreachable!();
                }
            }
        } else {
            return STATUStype::HAL_ERROR;
        }
        STATUStype::HAL_OK
    }

    fn set_trigger(&mut self, trigger: DacTrigger) -> STATUStype {
        self.trigger = trigger;
        STATUStype::HAL_OK
    }
    fn buffer(&mut self, buff: OUTPUT_BUF) -> STATUStype {
        self.buffer = buff;
        STATUStype::HAL_OK
    }

    fn lock(&mut self) -> STATUStype {
        if self.locked == LockType::LOCKED {
            STATUStype::HAL_BUSY
        } else {
            self.locked = LockType::LOCKED;
            STATUStype::HAL_OK
        }
    }
    fn unlock(&mut self) -> STATUStype {
        self.locked = LockType::UNLOCKED;
        STATUStype::HAL_OK
    }
}
