
use crate::object::CelestialObject;
use druid::Data;
use std::sync::{Arc};
use std::cell::RefCell;
use std::rc::Rc;
use druid::Point;
use std::sync::mpsc;
use std::thread;
use std::cmp::min;
use std::sync::RwLock;


const NUM_THREADS: u32 = 3;
const TRAJECTORY_POINTS: u32 = 5;


pub fn mass_to_radius(mass:f64) -> f64{
    // mass = [1.0, infty]
    // radius = [3.0, 100.0]
    mass.sqrt() + 2.0
}


#[derive(Clone, Data)]
pub struct Simulation {
    pub dt: f64,
    pub bodies: Rc<RefCell<Vec<CelestialObject>>>,
    pub proposed_body: CelestialObject,
    pub left_mouse_pressed: bool,
    pub middle_mouse_pressed: bool,
    pub init_cursor_pos: Point,
    pub cursor_pos: Point,
    pub scale: f64, 
    pub camera_pos: Point, 
    pub init_camera_pos: Point,
    pub trajectory: Rc<RefCell<Vec<Point>>>,
    pub play: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Simulation {
    // In its current state, the trajectory is calculated as if all the other bodies are stationary.
    // I think it would be cool to also consider the motion of the other bodies in the trajectory.
    pub fn calculate_trajectory(&mut self) {
        let mut body = self.proposed_body.clone();
        let scale = 20.0;
        let delta = ((self.cursor_pos.x - self.init_cursor_pos.x) / 1.0, (self.cursor_pos.y - self.init_cursor_pos.y) / 1.0);
        body.x = self.init_cursor_pos.x / self.scale  - self.camera_pos.x;
        body.y = self.init_cursor_pos.y / self.scale - self.camera_pos.y;
        body.prev_x = body.x + delta.0 / scale; 
        body.prev_y = body.y + delta.1 / scale;
        self.trajectory.borrow_mut().clear();
        self.trajectory.borrow_mut().push(Point::new(body.x, body.y));

        let num_points = 1000;
        let arr = self.bodies.borrow();

        for num in 0..TRAJECTORY_POINTS * num_points {
            let mut net_force: (f64, f64) = (0.0, 0.0);
            let mut state = false;
            for i in 0..arr.len() {
                let dist = ((arr[i].x - body.x).powf(2.0) + (arr[i].y - body.y).powf(2.0)).powf(0.5);
                if dist <= mass_to_radius(arr[i].mass) + mass_to_radius(body.mass) {
                    state = true;
                }
                let result = body.calculate_force(&arr[i]);
                net_force.0 += result.0;
                net_force.1 += result.1;
            }

            if state {
                break;
            }
            net_force.0 /= body.mass;
            net_force.1 /= body.mass;
            body.update_fields_from_force(&net_force, &self.dt);
            if num % 5 == 0 {
                self.trajectory.borrow_mut().push(Point::new(body.x, body.y));
            }
        }
    }

    // performs one tick to update the bodies' positions
    pub fn update(&mut self) {
        let mut accs: Vec<(f64, f64)> = Vec::new();
        let mut merges: Vec<(usize, usize)> = Vec::new();

        {
            let bodies = self.bodies.borrow();
            let mut net_force = (0.0, 0.0);

            for i in 0..bodies.len() {
                for j in 0..bodies.len() {
                    if i == j {
                        continue;
                    }
                    let dist = ((bodies[i].x - bodies[j].x).powf(2.0) + (bodies[i].y - bodies[j].y).powf(2.0)).powf(0.5);
                    if dist <= mass_to_radius(bodies[i].mass) + mass_to_radius(bodies[j].mass) {
                        merges.push((i, j));
                    }
                    let result = bodies[i].calculate_force(&bodies[j]);
                    net_force.0 += result.0;
                    net_force.1 += result.1;
                }
                net_force.0 /= bodies[i].mass;
                net_force.1 /= bodies[i].mass;
                accs.push(net_force);
            }
        }

        for (i, body) in self.bodies.borrow_mut().iter_mut().enumerate() {
            body.update_fields_from_force(&accs[i], &self.dt);
        }

        let mut delete_idxs: Vec<usize> = Vec::new();

        // goes through all the merges and selects the larger mass
        // to be the one that incorporates the smaller one.
        for i in 0..(merges.len() / 2) as usize{
            let body1 = self.bodies.borrow()[merges[i].0].clone();
            let body2 = self.bodies.borrow()[merges[i].1].clone();
            
            let center = (
                (body1.mass * body1.x + body2.mass * body2.x) / (body1.mass + body2.mass), 
                (body1.mass * body1.y + body2.mass * body2.y) / (body1.mass + body2.mass)
            );

            let prev_center = (
                (body1.mass * body1.prev_x + body2.mass * body2.prev_x) / (body1.mass + body2.mass),
                (body1.mass * body1.prev_y + body2.mass * body2.prev_y) / (body1.mass + body2.mass)
            );

            let index;
            let color;
            if body1.mass >= body2.mass {
                index = merges[i].0.clone();
                color = body1.color.clone();
                delete_idxs.push(merges[i].1);
            } else {
                index = merges[i].1.clone();
                color = body2.color.clone();
                delete_idxs.push(merges[i].0);
            }

            if body1.fixed || body2.fixed {
                self.bodies.borrow_mut()[index] = CelestialObject {
                    x: center.0,
                    y: center.1,
                    mass: body1.mass + body2.mass,
                    prev_x: center.0,
                    prev_y: center.1,
                    color: 6,
                    fixed: true,
                }
            } else {
                self.bodies.borrow_mut()[index] = CelestialObject {
                    x: center.0,
                    y: center.1,
                    mass: body1.mass + body2.mass,
                    prev_x: prev_center.0,
                    prev_y: prev_center.1,
                    color: color,
                    fixed: false
                }
            }
        }

        //deletes the merged bodies
        delete_idxs.sort();
        let mut bodies = self.bodies.borrow_mut();
        for (i, ele) in delete_idxs.iter().enumerate() {
            bodies.remove(ele - i);
        }
    }


    pub fn new(bodies: Rc<RefCell<Vec<CelestialObject>>>, dt: f64) -> Self {
        Self {
            bodies,
            dt,
            cursor_pos: Point::new(0.0, 0.0),
            camera_pos: Point::new(0.0, 0.0),
            proposed_body: CelestialObject::new(0.0, 0.0, 10.0),
            left_mouse_pressed: false,
            middle_mouse_pressed: false,
            scale: 1.0,
            init_cursor_pos: Point::new(0.0, 0.0),
            init_camera_pos: Point::new(0.0, 0.0),
            trajectory: Rc::new(RefCell::new(Vec::<Point>::new())),
            play: true,
            up: false,
            down: false,
            left: false,
            right:false,
        }
    }
}
