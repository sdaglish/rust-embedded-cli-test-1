#![no_std]

use heapless::spsc::Queue;
use heapless::String;
use heapless::Vec;

const BUFFER_SIZE: usize = 1028;

// TODO: Add a way to add a string to the output buffer in a macro / function

pub struct MenuParameters {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct MenuItem<'a> {
    pub command: &'static str,
    pub description: &'static str,
    pub function: fn(&Vec<&str, 8>, &mut Queue<char, BUFFER_SIZE>),
    pub parameters: &'a [MenuParameters],
}

pub struct EmbeddedCli {
    pub name: &'static str,
    pub input_buffer: String<BUFFER_SIZE>,
    pub output_buffer: Queue<char, BUFFER_SIZE>,
    menu: &'static [MenuItem<'static>],
}

impl EmbeddedCli {
    pub fn new(name: &'static str, menu: &'static [MenuItem<'static>]) -> Self {
        let mut s = Self {
            name,
            input_buffer: String::new(),
            output_buffer: Queue::new(),
            menu,
        };
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

    pub fn process(&mut self) {
        if self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
            while self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
                self.input_buffer.pop();
            }

            if self.input_buffer.len() == 0 {
                return;
            }

            self.output_buffer.enqueue('\r').ok();
            self.output_buffer.enqueue('\n').ok();

            // Checking through menu list to see if what's been entered was relevent.
            // But fifrst checking for the work help.

            // Split input_buffer into a vector, based on whitespaces.
            {
                let input_vector: Vec<&str, 8> = self.input_buffer.split(' ').collect();
                // for s in self.input_buffer.split(' ') {
                //     input_vector.push(s).ok();
                // }

                // Check through menu list to see if what's been entered was relevant.
                // But first checking for the word "help".

                {
                    if input_vector[0] == "help" {
                        // TODO: This would be better if it was a separate function, but borrow issues...
                        if input_vector.len() == 1 {
                            for c in "AVAILABLE ITEMS:\n\r".chars() {
                                self.output_buffer.enqueue(c).ok();
                            }
                            for item in self.menu {
                                for c in "  ".chars() {
                                    self.output_buffer.enqueue(c).ok();
                                }
                                for c in item.command.chars() {
                                    self.output_buffer.enqueue(c).ok();
                                }

                                for params in item.parameters {
                                    for c in " <".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for c in params.name.chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for c in ">".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                }
                                self.output_buffer.enqueue('\r').ok();
                                self.output_buffer.enqueue('\n').ok();
                            }
                        } else {
                            for item in self.menu {
                                if item.command.starts_with(input_vector[1]) {
                                    for c in "SUMMARY:\n\r".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for c in "  ".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for c in item.command.chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for params in item.parameters {
                                        for c in " <".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in params.name.chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in ">".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                    }
                                    // self.output_buffer.enqueue_str(" - ").ok();
                                    // self.output_buffer.enqueue_str(item.description).ok();
                                    self.output_buffer.enqueue('\r').ok();
                                    self.output_buffer.enqueue('\n').ok();
                                    self.output_buffer.enqueue('\r').ok();
                                    self.output_buffer.enqueue('\n').ok();

                                    for c in "PARAMETERS:\n\r".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for parameter in item.parameters {
                                        for c in "  ".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in "<".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in parameter.name.chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in ">".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in " - ".chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        for c in parameter.description.chars() {
                                            self.output_buffer.enqueue(c).ok();
                                        }
                                        self.output_buffer.enqueue('\r').ok();
                                        self.output_buffer.enqueue('\n').ok();
                                    }
                                    self.output_buffer.enqueue('\r').ok();
                                    self.output_buffer.enqueue('\n').ok();

                                    for c in "DESCRIPTION:\n\r".chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    for c in item.description.chars() {
                                        self.output_buffer.enqueue(c).ok();
                                    }
                                    self.output_buffer.enqueue('\r').ok();
                                    self.output_buffer.enqueue('\n').ok();

                                    break;
                                }
                            }
                        }
                    } else {
                        let mut found = false;
                        for item in self.menu {
                            if item.command.starts_with(input_vector[0]) {
                                (item.function)(&input_vector, &mut self.output_buffer);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            for c in "Unknown command: ".chars() {
                                self.output_buffer.enqueue(c).ok();
                            }
                            for c in input_vector[0].chars() {
                                self.output_buffer.enqueue(c).ok();
                            }
                            self.output_buffer.enqueue('\r').ok();
                            self.output_buffer.enqueue('\n').ok();
                        }
                    }
                }
            }
            self.input_buffer.clear();
            self.output_buffer.enqueue('\r').ok();
            self.output_buffer.enqueue('\n').ok();
            self.output_buffer.enqueue('>').ok();
            self.output_buffer.enqueue(' ').ok();
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
