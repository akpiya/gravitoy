use std::{thread, time};
#[derive(Clone)]
struct CelestialObject {
    x: f64,
    y: f64,
    mass: f64,
    v_x: f64,
    v_y: f64
}

impl CelestialObject {
    fn calculate_force(&self, source: &CelestialObject) -> (f64, f64) {
        let delta_x: f64 = source.x - self.x;
        let delta_y: f64 = source.y - self.y;

        let distance: f64 = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
        let force: f64 = (self.mass * source.mass) / (distance * distance);

        return (force * (delta_x / distance), force * (delta_y / distance));
    }

    fn update_fields_from_force(&mut self, acc: &(f64, f64), dt: &f64) {
        self.x += self.v_x * dt;
        self.y += self.v_y * dt;
        self.v_x += acc.0 * dt;
        self.v_y += acc.0 * dt
    }

    fn update_state(&mut self, bodies: &Vec<CelestialObject>, i: &usize, dt: &f64) {
        let mut net_force = (0.0, 0.0);
        for (index, body) in bodies.iter().enumerate() {
            if *i != index {
                let result = self.calculate_force(&body);
                net_force.0 += result.0;
                net_force.1 += result.1;
            }
        }
        net_force.0 /= self.mass;
        net_force.1 /= self.mass;

        self.update_fields_from_force(&net_force, dt);
    }
}

fn main(){
    let dt: f64 = 1.0;

    let mut bodies: Vec<CelestialObject> = Vec::new();

    bodies.push(CelestialObject { x: 0.0, y: 0.0, mass: 1000000.0, v_x: 0.0, v_y: 0.0 });
    bodies.push(CelestialObject { x: 500.0, y: 0.0, mass: 300.0, v_x: 2.0, v_y: 0.0 });
    //bodies.push(CelestialObject { x: 500.0, y: 500.0, mass: 300.0, v_x: 0.5, v_y: 0.5 });
    let copy_bodies = bodies.clone();

    loop {
        for body in bodies.iter() {
            println!("({}, {})", body.x, body.y);
        }
        println!();
        for (i, body) in bodies.iter_mut().enumerate() {
            body.update_state(&copy_bodies, &i, &dt)
        }

        

        thread::sleep(time::Duration::from_millis(500));
    }

    
}
