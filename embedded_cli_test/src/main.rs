use embedded_cli;

const MENU: &[embedded_cli::MenuItem] = &[embedded_cli::MenuItem {
    command: "hello",
    description: "Prints hello world",
    parameters: &[],
    function: |_, output_queue| {
        for c in "Hello world!\r\n".chars() {
            output_queue.enqueue(c).ok();
        }
    },
}];

fn main() {
    let mut cli = embedded_cli::EmbeddedCli::new("test", MENU);
    for c in "hello\r\n".chars() {
        cli.add_char(c);
        while let Some(output) = cli.get_output_char() {
            print!("{}", output);
        }
    }

    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();
    cli.process();

    while let Some(output) = cli.get_output_char() {
        print!("{}", output);
    }
}
