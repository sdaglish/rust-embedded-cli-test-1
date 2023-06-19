use heapless::spsc::Queue;
use heapless::String;
use heapless::Vec;

const BUFFER_SIZE: usize = 128;

pub struct MenuParameters {
    name: &'static str,
    description: &'static str,
}

pub struct MenuItem<'a> {
    pub command: &'static str,
    pub description: &'static str,
    pub function: fn(&Vec<&str, 8>, &mut Queue<char, BUFFER_SIZE>),
    pub parameters: &'a [MenuParameters],
}

pub struct EmbeddedCli {
    pub name: &'static str,
    input_buffer: String<BUFFER_SIZE>,
    output_buffer: Queue<char, BUFFER_SIZE>,
    menu: &'static [MenuItem<'static>],
}

impl EmbeddedCli {
    pub fn new(name: &'static str, menu: &'static [MenuItem<'static>]) -> Self {
        Self {
            name,
            input_buffer: String::new(),
            output_buffer: Queue::new(),
            menu,
        }
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
            self.output_buffer.enqueue('\r').ok();
            self.output_buffer.enqueue('\n').ok();

            while self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
                self.input_buffer.pop();
            }

            // Checking through menu list to see if what's been entered was relevent.
            // But fifrst checking for the work help.

            // Split input_buffer into a vector, based on whitespaces.
            {
                let mut input_vector: Vec<&str, 8> = Vec::new();
                for s in self.input_buffer.split(' ') {
                    input_vector.push(s).ok();
                }

                // Check through menu list to see if what's been entered was relevant.
                // But first checking for the word "help".

                if input_vector[0] == "help" {
                    // for item in self.menu {
                    //     self.output_buffer.enqueue_str(item.command).ok();
                    //     self.output_buffer.enqueue_str(" - ").ok();
                    //     self.output_buffer.enqueue_str(item.description).ok();
                    //     self.output_buffer.enqueue('\r').ok();
                    //     self.output_buffer.enqueue('\n').ok();
                    // }
                } else {
                    for item in self.menu {
                        if item.command.starts_with(input_vector[0]) {
                            (item.function)(&input_vector, &mut self.output_buffer);
                            break;
                        }
                    }
                }
            }
            self.input_buffer.clear();
        }
    }

    pub fn get_output_char(&mut self) -> Option<char> {
        self.output_buffer.dequeue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn add_char_adds_to_back_of_output_queue() {
    //     let mut cli = EmbeddedCli::new("test");
    //     cli.add_char('a');
    //     cli.add_char('b');
    //     assert_eq!(cli.output_buffer.dequeue(), Some('a'));
    //     assert_eq!(cli.output_buffer.dequeue(), Some('b'));
    // }
    //
    // #[test]
    // fn check_backspace() {
    //     let mut cli = EmbeddedCli::new("test");
    //     cli.add_char('a');
    //     cli.add_char('b');
    //     assert_eq!(cli.input_buffer.len(), 2);
    //     cli.add_char('\u{8f}');
    //     assert_eq!(cli.output_buffer.dequeue(), Some('a'));
    //     assert_eq!(cli.output_buffer.dequeue(), Some('b'));
    //     assert_eq!(cli.output_buffer.dequeue(), Some('\x08'));
    //     assert_eq!(cli.output_buffer.dequeue(), Some(' '));
    //     assert_eq!(cli.output_buffer.dequeue(), Some('\x08'));
    //
    //     assert_eq!(cli.input_buffer.len(), 1);
    // }
    //
    // #[test]
    // fn correct_name_stored() {
    //     let cli = EmbeddedCli::new("test2");
    //     assert_eq!(cli.name, "test2");
    // }
}
