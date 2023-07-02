use embedded_cli::MenuItem;
use embedded_cli::MenuParameters;

use crate::app::cli_temperature_setpoint;

pub const MENU: &[MenuItem] = &[
    MenuItem {
        command: "hello",
        description: "Prints hello world",
        parameters: &[],
        function: |_, output_string| {
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
