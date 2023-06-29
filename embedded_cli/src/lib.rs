#![no_std]

use heapless::spsc::Queue;
use heapless::String;
use heapless::Vec;

const BUFFER_SIZE: usize = 1028;

pub struct MenuParameters {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct MenuItem<'a> {
    pub command: &'static str,
    pub description: &'static str,
    pub function: fn(&Vec<&str, 8>, &mut String<BUFFER_SIZE>),
    pub parameters: &'a [MenuParameters],
}

pub struct EmbeddedCli {
    pub name: &'static str,
    pub input_buffer: String<BUFFER_SIZE>,
    pub output_buffer: Queue<char, BUFFER_SIZE>,
    menu: &'static [MenuItem<'static>],
}

impl EmbeddedCli {
    #[must_use]
    pub fn new(name: &'static str, menu: &'static [MenuItem<'static>]) -> Self {
        let mut s = Self {
            name,
            input_buffer: String::new(),
            output_buffer: Queue::new(),
            menu,
        };
        s.output_buffer.enqueue('\n').ok();
        s.output_buffer.enqueue('\r').ok();
        s.output_buffer.enqueue('>').ok();
        s.output_buffer.enqueue(' ').ok();
        s
    }

    // TODO: Update to return failure if queue is full
    pub fn add_char(&mut self, c: char) {
        // Backspace
        if (c == '\u{8f}') || (c == '\u{7f}') {
            self.output_buffer.enqueue('\x08').ok();
            self.output_buffer.enqueue(' ').ok();
            self.output_buffer.enqueue('\x08').ok();
            self.input_buffer.pop();
        } else {
            self.input_buffer.push(c).ok();
            self.output_buffer.enqueue(c).ok();
        }
    }

    // TODO: This takes up 5K of flash.  Can we make it smaller?
    pub fn process(&mut self) {
        if self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
            while self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
                self.input_buffer.pop();
            }

            if self.input_buffer.len() == 0 {
                return;
            }

            // Checking through menu list to see if what's been entered was relevent.
            // But first checking for the work help.
            let mut help_string = String::<BUFFER_SIZE>::new();
            help_string.push_str("\r\n").ok();

            // Split input_buffer into a vector, based on whitespaces.
            {
                let input_vector: Vec<&str, 8> = self.input_buffer.split(' ').collect();

                // Check through menu list to see if what's been entered was relevant.
                // But first checking for the word "help".

                {
                    if input_vector[0] == "help" {
                        let mut command_found = false;
                        // TODO: This would be better if it was a separate function, but borrow issues...
                        if input_vector.len() == 1 {
                            help_string.push_str("AVAILABLE ITEMS:\n\r").ok();
                            for item in self.menu {
                                help_string.push_str("  ").ok();
                                help_string.push_str(item.command).ok();

                                for params in item.parameters {
                                    help_string.push_str(" <").ok();
                                    help_string.push_str(params.name).ok();
                                    help_string.push_str(">").ok();
                                }
                                help_string.push_str("\r\n").ok();
                            }
                        } else {
                            for item in self.menu {
                                if item.command == (input_vector[1]) {
                                    command_found = true;

                                    help_string.push_str("SUMMARY:\n\r").ok();
                                    help_string.push_str(item.command).ok();
                                    for params in item.parameters {
                                        help_string.push_str(" <").ok();
                                        help_string.push_str(params.name).ok();
                                        help_string.push_str(">").ok();
                                    }
                                    help_string.push_str("\r\n\r\n").ok();
                                    help_string.push_str("PARAMETERS:\n\r").ok();
                                    for parameter in item.parameters {
                                        help_string.push_str("  <").ok();
                                        help_string.push_str(parameter.name).ok();
                                        help_string.push_str("> - ").ok();
                                        help_string.push_str(parameter.description).ok();
                                        help_string.push_str("\r\n").ok();
                                    }
                                    help_string.push_str("\r\n").ok();

                                    help_string.push_str("DESCRIPTION:\n\r").ok();
                                    help_string.push_str(item.description).ok();
                                    help_string.push_str("\r\n").ok();

                                    break;
                                }
                            }
                            if !command_found {
                                help_string.push_str("Unknown command: ").ok();
                                help_string.push_str(input_vector[1]).ok();
                                help_string.push_str("\r\n").ok();
                            }
                        }
                    } else {
                        let mut found = false;
                        for item in self.menu {
                            if item.command == input_vector[0] {
                                (item.function)(&input_vector, &mut help_string);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            help_string.push_str("Unknown command: ").ok();
                            help_string.push_str(input_vector[0]).ok();
                            help_string.push_str("\r\n").ok();
                        }
                    }
                }
            }

            self.input_buffer.clear();
            help_string.push_str("\r\n> ").ok();

            for c in help_string.chars() {
                self.output_buffer.enqueue(c).ok();
            }
        }
    }

    pub fn get_output_char(&mut self) -> Option<char> {
        self.output_buffer.dequeue()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     const MENU: &[MenuItem] = &[
//         MenuItem {
//             command: "hello",
//             description: "Prints hello world",
//             parameters: &[],
//             function: |_, output_queue| {
//                 for c in "Hello world! function\r\n".chars() {
//                     output_queue.enqueue(c).ok();
//                 }
//             },
//         },
//         MenuItem {
//             command: "test",
//             description: "Prints test",
//             parameters: &[
//                 MenuParameters {
//                     name: "a",
//                     description: "a something or other...",
//                 },
//                 MenuParameters {
//                     name: "b",
//                     description: "b something or other...",
//                 },
//             ],
//             function: |_, output_queue| {
//                 for c in "Test function!\r\n".chars() {
//                     output_queue.enqueue(c).ok();
//                 }
//             },
//         },
//     ];
//
//     #[test]
//     fn correct_name_stored() {
//         let cli = EmbeddedCli::new("test2", MENU);
//         assert_eq!(cli.name, "test2");
//     }
// }
//
