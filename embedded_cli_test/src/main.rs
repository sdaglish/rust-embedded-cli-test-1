#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

// TODO: This might get removed later on when going to release
use panic_rtt_target as _;
// use panic_reset as _;
use rtic::app;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use embedded_cli::{MenuItem, MenuParameters};
    use heapless::spsc::{Consumer, Producer, Queue};
    use heapless::String;
    use heapless::Vec;
    use rtic_monotonics::systick::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal::{
        pac::USART2,
        prelude::*,
        serial::{config::Config, Rx, Serial, Tx},
    };

    // TODO: Move to separate file
    const MENU: &[MenuItem] = &[
        MenuItem {
            command: "hello",
            description: "Prints hello world",
            parameters: &[],
            function: |_, output_string| {
                output_string.clear();
                output_string.push_str("Hello world! function\r\n").ok();
            },
        },
        MenuItem {
            command: "test",
            description: "Prints test",
            parameters: &[
                MenuParameters {
                    name: "a",
                    description: "a something or other...",
                },
                MenuParameters {
                    name: "b",
                    description: "b something or other...",
                },
            ],
            function: |_, output_string| {
                output_string.clear();
                output_string.push_str("Test function!\r\n").ok();
            },
        },
        MenuItem {
            command: "temperature_setpoint",
            description: "Control the temperature setpoint",
            parameters: &[
                MenuParameters {
                    name: "control",
                    description: "'set', 'get', or 'default'",
                },
                MenuParameters {
                    name: "value",
                    description: "The value to set the setpoint to (only used with 'set')",
                },
            ],
            function: cli_temperature_setpoint,
        },
    ];

    fn cli_temperature_setpoint(parameters: &Vec<&str, 8>, output_string: &mut String<1028>) {
        output_string.clear();
        if parameters.len() == 1 {
            output_string.push_str("Missing parameter\r\n").ok();
            return;
        }
        match parameters[1] {
            "set" => {
                if parameters.len() != 3 {
                    output_string.push_str("Missing parameter\r\n").ok();
                    return;
                }
                output_string
                    .push_str("Setting temperature setpoint to ")
                    .ok();
                output_string.push_str(parameters[2]).ok();
                output_string.push_str("\r\n").ok();
            }
            "get" => {
                output_string
                    .push_str("Getting temperature setpoint\r\n")
                    .ok();
            }
            "default" => {
                output_string
                    .push_str("Setting temperature setpoint to default\r\n")
                    .ok();
            }
            _ => {
                output_string.push_str("Unknown parameter\r\n").ok();
            }
        }
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        serial_debug_rx: Rx<USART2>,
        serial_debug_tx: Tx<USART2>,
        uart2_rx_consumer: Consumer<'static, u8, 32>,
        uart2_rx_producer: Producer<'static, u8, 32>,
        uart2_tx_consumer: Consumer<'static, u8, 32>,
        uart2_tx_producer: Producer<'static, u8, 32>,
        serial_debug_cli: embedded_cli::EmbeddedCli,
    }

    #[init(local = [uart2_rx_queue: Queue<u8, 32> = Queue::new(), uart2_tx_queue: Queue<u8, 32> = Queue::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        rprintln!("Hello, world!");

        let dp = cx.device;
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(8_000_000.Hz())
            .use_hse(8_000_000.Hz())
            .freeze();

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 8_000_000, systick_mono_token); // default STM32F401 clock-rate is 16MHz

        // Uart Rx is PA3, Tx is PA2
        let gpioa = dp.GPIOA.split();
        let tx_pin = gpioa.pa2.into_alternate();
        let rx_pin = gpioa.pa3.into_alternate();

        let (serial_debug_tx, mut serial_debug_rx) = Serial::new(
            dp.USART2,
            (tx_pin, rx_pin),
            Config::default().baudrate(115_200.bps()),
            &clocks,
        )
        .unwrap()
        .split();

        serial_debug_rx.listen();

        let (uart2_rx_producer, uart2_rx_consumer) = cx.local.uart2_rx_queue.split();
        let (uart2_tx_producer, uart2_tx_consumer) = cx.local.uart2_tx_queue.split();

        let serial_debug_cli = embedded_cli::EmbeddedCli::new("Serial Debug", MENU);

        cli_task::spawn().ok();

        (
            Shared {},
            Local {
                serial_debug_rx,
                serial_debug_tx,
                uart2_rx_consumer,
                uart2_rx_producer,
                uart2_tx_consumer,
                uart2_tx_producer,
                serial_debug_cli,
            },
        )
    }

    #[task(local = [uart2_rx_consumer, serial_debug_cli, serial_debug_tx])]
    async fn cli_task(cx: cli_task::Context) {
        let uart2_rx_consumer = cx.local.uart2_rx_consumer;

        loop {
            while uart2_rx_consumer.peek().is_some() {
                if let Some(byte) = uart2_rx_consumer.dequeue() {
                    cx.local.serial_debug_cli.add_char(byte as char);
                }
            }
            cx.local.serial_debug_cli.process();

            // Check if there is something from embedded_cli.get_output_char. If there is
            // then send it to serial_debug_tx
            loop {
                let byte = cx.local.serial_debug_cli.get_output_char();
                match byte {
                    Some(byte) => {
                        cx.local.serial_debug_tx.write(byte as u8).ok();
                        Systick::delay(1.millis()).await;
                    }
                    None => {
                        break;
                    }
                }
            }
            Systick::delay(100.millis()).await;
        }
    }

    #[task(binds = USART2, local = [serial_debug_rx, uart2_rx_producer])]
    fn usart_rx(cx: usart_rx::Context) {
        let rx = cx.local.serial_debug_rx;
        if let Ok(byte) = rx.read() {
            cx.local.uart2_rx_producer.enqueue(byte).ok();
        } else {
            rprintln!("No space on uart2_rx_producer");
        }
    }
}
