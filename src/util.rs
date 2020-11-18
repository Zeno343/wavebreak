pub struct Queue<T> {
    queue: Vec<T>,
    pub max_size: usize,
}

impl<T> Queue<T> {
    pub fn new(max_size: usize) -> Queue<T> {
        Queue {
            queue: Vec::new(),
            max_size, 
        }
    }

    pub fn push(&mut self, item: T) {
        if self.queue.len() == self.max_size {
            //remove the first item to make room
            self.pop();
            self.queue.push(item);
        } else {
            self.queue.push(item);
        }
    }

    pub fn pop(&mut self) -> T {
        self.queue.remove(0)
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.queue.iter()
    }
}
    
