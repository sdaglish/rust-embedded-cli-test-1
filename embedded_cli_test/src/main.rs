#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_rtt_target as _;
use rtic::app;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use heapless::spsc::{Consumer, Producer, Queue};
    use rtic_monotonics::systick::*;
    use stm32f4xx_hal::{
        pac::USART2,
        prelude::*,
        serial::{config::Config, Rx, Serial, Tx},
    };
    use embedded_cli;
    
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }
}
