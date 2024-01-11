use druid::Data;

// Refers to 
#[derive(Clone, PartialEq, Data, Debug)]
pub struct CelestialObject {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub prev_x: f64,
    pub prev_y: f64,
    pub color: usize,
    pub fixed: bool,
}


impl CelestialObject {
    pub fn calculate_force(&self, source: &CelestialObject) -> (f64, f64) {
        let delta_x: f64 = source.x - self.x;
        let delta_y: f64 = source.y - self.y;

        let distance: f64 = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
        let force: f64 =  20.0 * (self.mass * source.mass) / (distance * distance);

        return (force * (delta_x / distance), force * (delta_y / distance));
    }

    pub fn update_fields_from_force(&mut self, acc: &(f64, f64), dt: &f64) {
        let prev_x = self.x;
        let prev_y = self.y;

        self.x = 2.0 * self.x - self.prev_x + acc.0 * dt * dt;
        self.y = 2.0 * self.y - self.prev_y + acc.1 * dt * dt;
        
        self.prev_x = prev_x;
        self.prev_y = prev_y;
    }

    pub fn new(x: f64, y:f64, mass:f64) -> Self {
        Self {
            x,
            y,
            mass,
            prev_x: x,
            prev_y: y,
            color: 0 as usize,
            fixed: false,
        }
    }

    pub fn new_v0(x: f64, y:f64, mass:f64, v_x: f64, v_y: f64) -> Self {
        Self {
            x: x + v_x,
            y: y + v_y,
            mass,
            prev_x: x,
            prev_y: y,
            color: 0 as usize,
            fixed: false,
        }
    }
}
