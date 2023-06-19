use heapless::spsc::Queue;
use heapless::String;

pub struct EmbeddedCli {
    pub name: &'static str,
    input_buffer: String<128>,
    output_buffer: Queue<char, 128>,
}

impl EmbeddedCli {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            input_buffer: String::new(),
            output_buffer: Queue::new(),
        }
    }

    // TODO: Update to return failure if queue is full
    pub fn add_char(&mut self, c: char) {
        // self.input_buffer.enqueue(c).ok();
        self.input_buffer.push(c).ok();
        self.output_buffer.enqueue(c).ok();
    }

    pub fn process(&mut self) {
        // let c = self.input_buffer.dequeue();
        // let c = self.input_buffer.last().cloned();
        if self.input_buffer.ends_with('\r') || self.input_buffer.ends_with('\n') {
            self.output_buffer.enqueue('\r').ok();
            self.output_buffer.enqueue('\n').ok();

            self.input_buffer.pop();
        }
    }

    pub fn get_output_char(&mut self) -> Option<char> {
        self.output_buffer.dequeue()
    }
}
// impl<T> EmbeddedCli<T> {
//     pub fn new(input: T, output: T) -> Self {
//         Self { input, output }
//     }

//     pub fn add_char(&mut self, c: char) {
//         self.input.push(c);
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_char_adds_to_back_of_output_queue() {
        let mut cli = EmbeddedCli::new("test");
        cli.add_char('a');
        cli.add_char('b');
        assert_eq!(cli.output_buffer.dequeue(), Some('a'));
        assert_eq!(cli.output_buffer.dequeue(), Some('b'));
    }

    // #[test]
    // fn add_char_adds_to_back_of_queue() {
    //     let mut cli = EmbeddedCli::new("test");
    //     cli.add_char('a');
    //     assert_eq!(cli.input_buffer[0], Some('a'));
    // }

    // #[test]
    // fn add_char_does_not_overflow() {
    //     let mut cli = EmbeddedCli::new("test");
    //     for _ in 0..128 {
    //         cli.add_char('a');
    //     }
    //     assert_eq!(cli.input_buffer.enqueue('a'), Err('a'));
    // }

    // #[test]
    // fn add_char_does_not_overflow_when_full() {
    //     let mut cli = EmbeddedCli::new("test");
    //     for _ in 0..128 {
    //         cli.add_char('a');
    //     }
    //     cli.input_buffer.dequeue();
    //     cli.add_char('a');
    //     assert_eq!(cli.input_buffer.enqueue('a'), Err('a'));
    // }

    #[test]
    fn correct_name_stored() {
        let cli = EmbeddedCli::new("test2");
        assert_eq!(cli.name, "test2");
    }
}
