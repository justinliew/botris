
#[derive(Debug,Default,Copy,Clone)]
pub struct Chain {
    value: Option<usize>,
    countdown: f64, 
}

impl Chain {
    pub fn inc(&mut self) {
        *self.value.get_or_insert(1) += 1;
        self.countdown = 5.;
    }

    pub fn clear(&mut self) {
        self.value = None;
        self.countdown = 0.;
    }

    pub fn get_value(&self) -> Option<usize> {
        self.value
    }

    pub fn update(&mut self, dt: f64) -> Option<usize> {
        if self.countdown > 0. {
            self.countdown -= dt;
            if self.countdown <= 0. {
                let v = self.value;
                self.value = None;
                return v;
            }
        }
        None
    }
}