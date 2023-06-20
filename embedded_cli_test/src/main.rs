#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

// use panic_rtt_target as _;
use panic_reset as _;
use rtic::app;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use embedded_cli::{MenuItem, MenuParameters};
    use heapless::spsc::{Consumer, Producer, Queue};
    use heapless::Vec;
    use rtic_monotonics::systick::*;
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
            function: |_, output_queue| {
                for c in "Hello world! function\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
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
            function: |_, output_queue| {
                for c in "Test function!\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
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

    fn cli_temperature_setpoint(parameters: &Vec<&str, 8>, output_queue: &mut Queue<char, 1028>) {
        if parameters.len() == 1 {
            for c in "Missing parameter\r\n".chars() {
                output_queue.enqueue(c).ok();
            }
            return;
        }
        match parameters[1] {
            "set" => {
                if parameters.len() != 3 {
                    for c in "Missing parameter\r\n".chars() {
                        output_queue.enqueue(c).ok();
                    }
                    return;
                }
                for c in "Setting temperature setpoint to ".chars() {
                    output_queue.enqueue(c).ok();
                }
                for c in parameters[2].chars() {
                    output_queue.enqueue(c).ok();
                }
                for c in "\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
            }
            "get" => {
                for c in "Getting temperature setpoint\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
            }
            "default" => {
                for c in "Setting temperature setpoint to default\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
            }
            _ => {
                for c in "Unknown parameter\r\n".chars() {
                    output_queue.enqueue(c).ok();
                }
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

        let serial_debug_cli = embedded_cli::EmbeddedCli::new("Serial Debug", &MENU);

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
            while true {
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
            // rprintln!("No good");
        }
    }
}
