pub struct SimpleRng {
    m: usize,
    a: usize,
    c: usize,
    seed: usize,
}

impl SimpleRng {
    pub fn new(seed: usize) -> SimpleRng {
        let m = 2684435399;
        let a = 31792125;
        let c = 9005;

        SimpleRng { m, a, c, seed }
    }

    fn usize(&mut self) -> usize {
        self.seed = (self.a * self.seed + self.c) % self.m;
        self.seed
    }

    pub fn roll(&mut self, lower: usize, upper: usize) -> usize {
        let value = self.usize();
        value % (upper - lower) + lower
    }
}
