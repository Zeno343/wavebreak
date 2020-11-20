use std::collections::BinaryHeap;
use std::cmp::{
    PartialEq,
    Eq,
    Ord,
    Ordering,
};

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

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.queue.iter()
    }
}
    
#[derive(Eq)]
pub struct PriorityItem<T, C> {
    pub item: T,
    cost: C,
}

impl<T: Eq, C: Ord> Ord for PriorityItem<T, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

impl<T, C: PartialOrd> PartialOrd for PriorityItem<T, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost).map(|part| part.reverse())
    }
}

impl<T, C: PartialEq> PartialEq for PriorityItem<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl From<((usize, usize), usize)> for PriorityItem<(usize, usize), usize> {
    fn from(from: ((usize, usize), usize)) -> PriorityItem<(usize, usize), usize> {
        PriorityItem {
            item: from.0,
            cost: from.1,
        }
    }
}

pub struct PriorityQueue<T, C> {
    heap: BinaryHeap<PriorityItem<T, C>>,
    max_size: usize,
}

impl<T: Eq, C: Ord + PartialEq> PriorityQueue<T, C> {
    pub fn new(max_size: usize) -> PriorityQueue<T, C> {
        PriorityQueue {
            heap: BinaryHeap::new(),
            max_size, 
        }
    }

    pub fn push(&mut self, item: PriorityItem<T, C>) {
        if self.heap.len() == self.max_size {
            //remove the first item to make room
            self.pop();
            self.heap.push(item);
        } else {
            self.heap.push(item);
        }
    }

    pub fn pop(&mut self) -> Option<PriorityItem<T, C>> {
        self.heap.pop()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn iter(&self) -> std::collections::binary_heap::Iter<PriorityItem<T, C>> {
        self.heap.iter()
    }
}
