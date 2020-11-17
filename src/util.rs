pub struct Queue<T> {
    queue: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            queue: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.queue.push(item);
    }

    pub fn pop(&mut self) -> T {
        self.queue.remove(0)
    }
}
    
