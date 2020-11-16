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

    pub fn usize(&mut self) -> usize {
        self.seed = (self.a * self.seed + self.c) % self.m;                                                                                                    
        self.seed                                                                                                                                              
    }

    pub fn f64(&mut self) -> f64 {                                                                                                                          
        let value = self.usize() as f64;
        value / self.m as f64
    }
}
