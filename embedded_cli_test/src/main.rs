#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

// TODO: This might get removed later on when going to release
use panic_rtt_target as _;
// use panic_reset as _;
use rtic::app;

mod menu;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use heapless::spsc::{Consumer, Producer, Queue};
    use heapless::String;
    use heapless::Vec;
    use rtic_monotonics::systick::Systick;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal::{
        i2c::{DutyCycle, Mode},
        pac::USART2,
        prelude::*,
        serial::{config::Config, Rx, Serial, Tx},
    };

    const UART_RX_SIZE: usize = 1024;

    pub fn cli_temperature_setpoint(parameters: &Vec<&str, 8>, output_string: &mut String<1028>) {
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
        uart2_rx_consumer: Consumer<'static, u8, UART_RX_SIZE>,
        uart2_rx_producer: Producer<'static, u8, UART_RX_SIZE>,
        serial_debug_cli: embedded_cli::EmbeddedCli,
        i2c: stm32f4xx_hal::i2c::I2c<stm32f4xx_hal::pac::I2C1>,
    }

    #[init(local = [uart2_rx_queue: Queue<u8, UART_RX_SIZE> = Queue::new(), uart2_tx_queue: Queue<u8, UART_RX_SIZE> = Queue::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        rprintln!("Starting app");

        let dp = cx.device;
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16_000_000.Hz()).freeze();

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
        // .unwrap()
        .expect("USART2 not setup correctly")
        .split();

        serial_debug_rx.listen();

        let (uart2_rx_producer, uart2_rx_consumer) = cx.local.uart2_rx_queue.split();
        // let (uart2_tx_producer, uart2_tx_consumer) = cx.local.uart2_tx_queue.split();

        let serial_debug_cli = embedded_cli::EmbeddedCli::new("Serial Debug", crate::menu::MENU);

        // Setting up the I2C peripheral for GPIO expander
        // SCL = PB8, SDA = PB9
        let gpiob = dp.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_open_drain();
        let sda = gpiob.pb9.into_alternate_open_drain();

        let mut i2c = dp.I2C1.i2c(
            (scl, sda),
            Mode::Fast {
                frequency: 400_000.Hz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            &clocks,
        );

        // Write to the GPIO expander - PCA9535PW - and set IO - 0.0 to output and high
        i2c.write(0x20, &[0x06, 0x00]).unwrap();
        i2c.write(0x20, &[0x02, 0x00]).unwrap();

        cli_task::spawn().ok();
        gpio_toggle::spawn().ok();

        (
            Shared {},
            Local {
                serial_debug_rx,
                serial_debug_tx,
                uart2_rx_consumer,
                uart2_rx_producer,
                serial_debug_cli,
                i2c,
            },
        )
    }

    // Obtains the CLI rx, sends it over to the CLI, and then transmits whateven is stored in the tx
    #[task(local = [uart2_rx_consumer, serial_debug_cli, serial_debug_tx])]
    async fn cli_task(cx: cli_task::Context) {
        let uart2_rx_consumer = cx.local.uart2_rx_consumer;
        let serial_debug_cli = cx.local.serial_debug_cli;

        loop {
            while uart2_rx_consumer.peek().is_some() {
                if let Some(byte) = uart2_rx_consumer.dequeue() {
                    serial_debug_cli.add_char(byte as char);
                }
            }
            serial_debug_cli.process();

            // Check if there is something from embedded_cli.get_output_char. If there is
            // then send it to serial_debug_tx
            while !serial_debug_cli.output_buffer_is_empty() {
                let byte = serial_debug_cli.get_output_char();
                match byte {
                    Some(byte) => {
                        cx.local.serial_debug_tx.write(byte as u8).ok();
                        // Systick::delay(1.millis()).await;
                    }
                    _ => {}
                };
            }

            Systick::delay(10.millis()).await;
        }
    }

    #[task(local = [i2c])]
    async fn gpio_toggle(cx: gpio_toggle::Context) {
        let i2c = cx.local.i2c;
        loop {
            i2c.write(0x20, &[0x02, 0x00]).unwrap();
            Systick::delay(5000.millis()).await;
            i2c.write(0x20, &[0x02, 0x1]).unwrap();
            Systick::delay(5000.millis()).await;
        }
    }

    #[task(binds = USART2, local = [serial_debug_rx, uart2_rx_producer])]
    fn usart_rx(cx: usart_rx::Context) {
        let rx = cx.local.serial_debug_rx;
        if let Ok(byte) = rx.read() {
            cx.local.uart2_rx_producer.enqueue(byte).ok();
        } else {
            rprintln!("Error reading from USART2");
        }
    }
}
