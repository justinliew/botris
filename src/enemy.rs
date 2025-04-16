pub struct Enemy {
    pub countdown: f64,
}

const COOLDOWN : f64 = 30.;

impl Enemy {
    pub fn new() -> Self {
        Enemy{
            countdown: COOLDOWN
        }
    }

    pub fn update(&mut self, dt: f64) -> bool {
        self.countdown -= dt;
        if self.countdown < 0. {
            self.countdown = COOLDOWN;
            return true;
        }
        return false;
    }

    pub fn attack(&mut self, chain: u32) {
        self.countdown += chain as f64 * 2.;
    }
}