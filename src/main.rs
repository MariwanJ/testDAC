#![no_main]
#![no_std]

use core::cell::RefCell;
use fugit::{HertzU32};
use timers::Ticker;

use panic_halt as _;
mod timers;
use crate::timers::MyTimer;
mod mydac;
use crate::mydac::{Dac, DacTrigger, OUTPUT_BUF};
use crate::mydac::DacHW;

use fugit::ExtU64;

use stm32f7xx_hal::{
    gpio::{ GpioExt},
    pac,
    rcc::{  HSEClock, HSEClockMode, RccExt, APB1, PLLP },
    rtc::{  Rtc, RtcClock },
    serial::{  Config, Serial },
};
use cortex_m_rt::entry;
use cortex_m_semihosting::{ hprintln };

#[allow(dead_code)]
#[allow(unused)]
static mut GLOBAL_PTR: *mut MyTimer = core::ptr::null_mut();

#[entry]
fn main() -> ! {
    let mut temp:i32=0;
    while  temp<1000_000  {
        temp +=1;
    }

    let mut dp = pac::Peripherals::take().unwrap();

    let gpioa = dp.GPIOA.split(); // DAC
    let gpiob = dp.GPIOB.split(); //LED digital out
    let gpioc = dp.GPIOC.split(); // Button Digital in
    let gpiod = dp.GPIOD.split(); // UART2

    let mut dac_pin = gpioa.pa4.into_analog();
    let mut dacObj = Dac::new(dp.DAC,&dp.RCC, dac_pin);
    
    // Initialize the system clock
    let mut rcc = dp.RCC.constrain();
    // Configure the clock based on STM32IDECubeMX

    let clocks = rcc.cfgr
        // .lse(lse_var)   lse in HAL is not implemented
        .hse(HSEClock::new(HertzU32::MHz(8), HSEClockMode::Oscillator)) //8 MHz HSE crystal
        .sysclk(HertzU32::MHz(216)) // Set system clock to 216 MHz
        .use_pll()
        .pclk1(HertzU32::MHz(54))
        .pllq(9) // Set PLLQ
        .plln(216)
        .pllp(PLLP::Div2)
        .pllm(4)
        .freeze();

    let mut led1 = gpiob.pb0.into_push_pull_output();
    let mut led2 = gpiob.pb7.into_push_pull_output();
    let mut led3 = gpiob.pb14.into_push_pull_output();

    // Create a new RTC instance
    // Create a new RTC instance   - See  AN4759 Table 7.
    // HSE = 8Mhz_crystal/8 = 1Mhz
    let mut my_rtc = Rtc::new(
        dp.RTC,
        7999,
        127,
        RtcClock::Hse { divider: 8 },
        clocks,
        &mut rcc.apb1,  //rcc.apb1,
        &mut dp.PWR
    ).expect("RTC initialization failed");
    
    let mut ticker = Ticker::new(my_rtc);
    let mut my_timer: RefCell<MyTimer> = RefCell::new(MyTimer::new(100.millis(), & mut ticker));
    let mut serial_config = Config::default();
    serial_config.baud_rate = HertzU32::Hz(115200);
    // Configure UART2 on PD5 (TX) and PD6 (RX)

    let tx = gpiod.pd5.into_alternate(); // Set PA9 to alternate function for TX
    let rx = gpiod.pd6.into_alternate(); // Set PA10 to alternate function for RX
    let serial = Serial::new(dp.USART2, (tx, rx), &clocks, serial_config);
    let (mut txU, mut rxU) = serial.split();

    let mut x:i16=0;
    let mut dir:i16=1;
    dacObj.set_trigger(DacTrigger::None);
    dacObj.enable();
    dacObj.buffer(OUTPUT_BUF::Disabled);
    dacObj.start(1);
    loop {
        dacObj.write(x as u16);
		if x >= 4095 {
		    x = 4095;
		    dir = -1;
		}
		else if x <= 0 {
		    x = 0;
		    dir = 1;
		}
        x += dir;
    }
}

/* Needed to be able to compile for this processor */
use critical_section::{ self, acquire };

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _critical_section_1_0_acquire() {
    acquire();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _critical_section_1_0_release() {
    // No action needed; just return.
}

use crate::pac::interrupt;
#[interrupt]
fn QUADSPI() {
    // dummy QUADSPI interrupt handler or you get link error, must be a BUG!!!
}
