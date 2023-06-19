use embedded_cli;

const MENU: &[embedded_cli::MenuItem] = &[
    embedded_cli::MenuItem {
        command: "hello",
        description: "Prints hello world",
        parameters: &[],
        function: |_, output_queue| {
            for c in "Hello world 123!\r\n".chars() {
                output_queue.enqueue(c).ok();
            }
        },
    },
    embedded_cli::MenuItem {
        command: "test",
        description: "Prints test",
        parameters: &[
            embedded_cli::MenuParameters {
                name: "a",
                description: "a something or other...",
            },
            embedded_cli::MenuParameters {
                name: "b",
                description: "b something or other...",
            },
        ],
        function: |_, output_queue| {
            for c in "Help!\r\n".chars() {
                output_queue.enqueue(c).ok();
            }
        },
    },
];

fn main() {
    let mut cli = embedded_cli::EmbeddedCli::new("test", MENU);
    for c in "hello\r\n".chars() {
        cli.add_char(c);
        cli.process();
        //     println!("C: {}", c);
        // println!("Input buffer: {:?}", cli.input_buffer  );
        // println!("Output buffer: {:?}", cli.output_buffer  );
    }

    while let Some(output) = cli.get_output_char() {
        print!("{}", output);
    }

    for c in "help\r\n".chars() {
        cli.add_char(c);
        cli.process();
        //     println!("C: {}", c);
        // println!("Input buffer: {:?}", cli.input_buffer  );
        // println!("Output buffer: {:?}", cli.output_buffer  );
    }

    while let Some(output) = cli.get_output_char() {
        print!("{}", output);
    }
    // println!("Output buffer: {:?}", cli.output_buffer);
}
