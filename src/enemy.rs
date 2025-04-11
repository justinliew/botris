pub struct Enemy {
    pub countdown: f64,
}

impl Enemy {
    pub fn new() -> Self {
        Enemy{
            countdown: 30.
        }
    }

    pub fn update(&mut self, dt: f64) -> bool {
        self.countdown -= dt;
        if self.countdown < 0. {
            self.countdown = 30.;
            return true;
        }
        return false;
    }

    pub fn attack(&mut self, chain: u32) {
        self.countdown += chain as f64 * 2.;
    }
}