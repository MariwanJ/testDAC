#![no_std]
use core::convert;

use cortex_m_semihosting::hprintln;
use fugit::{Duration, Instant};

use stm32f7xx_hal::{pac, rcc::Clocks, rtc::{Rtc}};


type TickInstant = Instant<u64, 1, 1000000>;
type TickDuration = Duration<u64, 1, 1000000>;

pub struct MyTimer<'a> {
    end_time: TickInstant,
    ticker: &'a mut Ticker,
    dur: TickDuration,
}

impl<'a> MyTimer<'a> {
    pub fn new(duration: TickDuration, ticker: &'a mut Ticker) -> Self {
        Self {
            end_time: ticker.now() + duration,
            ticker,
            dur: duration,
        }
    }
    pub fn set_duration(&mut self,new_dur : TickDuration){
        self.dur=new_dur;
    }
    pub fn set_duration_u8(&mut self,new_dur : u64){
        self.dur=TickDuration::micros (new_dur);
    }
    pub fn new_default(ticker: &'a mut Ticker) -> Self {
        Self {
            end_time: ticker.now() + TickDuration::micros(500_000),
            ticker,
            dur:  TickDuration::micros(500_000),
        }
    }

    pub fn is_ready(& mut self) -> bool {
        if self.ticker.now() > self.end_time {
            self.end_time=self.ticker.now()+self.dur;
            return true
        }           
        false
    }
    pub fn blocking_is_ready(& mut self)->bool{
        while self.ticker.now() <= self.end_time {
            //hprintln!("1 {} {}", self.ticker.now(), self.end_time);
        }
        //hprintln!("2 {} {}", self.ticker.now(), self.end_time);
        self.end_time=self.ticker.now()+self.dur;
        true
    }
}

/// Keeps track of time for the system using RTC0, which ticks away at a rate
/// of 32,768/sec using a low-power oscillator that runs even when the core is
/// powered down.
///
/// RTC0's counter is only 24-bits wide, which means there will be an overflow
/// every ~8min, which we do not account for: this will be fixed in chapter 4.
pub struct Ticker {
    rtc: Rtc,
}

impl Ticker {
    /// Create on startup to get RTC0 going.
    pub fn new(mut nrtc:Rtc) -> Self {
      Ticker { rtc: nrtc } // Return a Ticker instance
    }

    pub fn now(& mut self) -> TickInstant {
        let val= self.rtc.get_datetime();
        let (h,m,s,mic)=val.time().as_hms_micro();
        let converted = (h as u64 * 3_600_000_000u64)
              + (m as u64 * 60_000_000u64)
              + (s as u64 * 1_000_000u64)
              + (mic as u64);
        TickInstant::from_ticks(converted)
    }
}
