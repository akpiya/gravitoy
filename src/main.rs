use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::max;
use druid::Data;
use druid::kurbo::{Size, Circle, Line};
use druid::widget::prelude::*;
use druid::{
    AppLauncher, LocalizedString, WindowDesc, TimerToken,
    Color, Point, MouseButton, Code, Modifiers
};


static TIMER_INTERVAL: Duration = Duration::from_millis(10);
const ALLCOLORS: [Color; 7] = [Color::RED, Color::BLUE, Color::GREEN, Color::PURPLE, Color::YELLOW, Color::WHITE, Color::BLACK];
const TRAJECTORY_POINTS: u32 = 5;

struct GravityDisplay {
    timer_id: TimerToken,
}

#[derive(Clone, Data)]
struct Simulation {
    dt: f64,
    bodies: Rc<RefCell<Vec<CelestialObject>>>,
    proposed_body: CelestialObject,
    left_mouse_pressed: bool,
    middle_mouse_pressed: bool,
    init_cursor_pos: Point,
    cursor_pos: Point,
    scale: f64, 
    camera_pos: Point, 
    init_camera_pos: Point,
    trajectory: Rc<RefCell<Vec<Point>>>,
    play: bool,
}


#[derive(Clone, PartialEq, Data)]
struct CelestialObject {
    x: f64,
    y: f64,
    mass: f64,
    prev_x: f64,
    prev_y: f64,
    color: usize,
    fixed: bool,
}


fn mass_to_radius(mass:f64) -> f64{
    // mass = [1.0, infty]
    // radius = [3.0, 100.0]
    mass.sqrt() + 2.0
}


impl Widget<Simulation> for GravityDisplay {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Simulation, _env: &Env) {
        match event {
            Event::WindowConnected => {
                self.timer_id = ctx.request_timer(TIMER_INTERVAL);
                ctx.request_focus();
            }

            Event::KeyDown(button) => {
                match button.code {
                    Code::Comma => data.scale -= 0.03,
                    Code::Period => data.scale += 0.03,
                    Code::ArrowRight => {
                        data.proposed_body.color = (data.proposed_body.color + 1) % ALLCOLORS.len();
                    },
                    Code::ArrowLeft => {
                        if data.proposed_body.color == 0 {
                            data.proposed_body.color = ALLCOLORS.len() - 1;
                        } else {
                            data.proposed_body.color -= 1;
                        }
                    },
                    Code::KeyW => data.camera_pos.y += 1.0 / data.scale * 30.0,
                    Code::KeyA => data.camera_pos.x += 1.0 / data.scale * 30.0,
                    Code::KeyS => data.camera_pos.y -= 1.0 / data.scale * 30.0,
                    Code::KeyD => data.camera_pos.x -= 1.0 / data.scale * 30.0,
                    Code::Space => data.play = !data.play,
                    _ => {}
                }
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    ctx.request_paint();
                    if data.play {
                        data.update();
                    }
                    if data.left_mouse_pressed {
                        data.calculate_trajectory();
                    }
                    self.timer_id = ctx.request_timer(TIMER_INTERVAL);
                }
            }
            Event::MouseDown(mouse) => {
                match mouse.button {
                    MouseButton::Middle => {
                        data.middle_mouse_pressed = true;
                        data.init_cursor_pos = data.cursor_pos.clone();
                        data.init_camera_pos = data.camera_pos.clone();
                    }
                    MouseButton::Left => {
                        // Normal case if mouse button is pressed
                        data.left_mouse_pressed = true;
                        data.init_cursor_pos = data.cursor_pos.clone();
                    }
                    _ => {}
                }
            }
            Event::MouseMove(event) => {
                data.cursor_pos = event.window_pos.clone();
                if !data.left_mouse_pressed{
                    data.proposed_body.x = data.cursor_pos.x.clone();
                    data.proposed_body.y = data.cursor_pos.y.clone();
                }
                ctx.request_paint();
            }
            Event::Wheel(event) => {
                data.proposed_body.mass += 0.5 * event.wheel_delta.y / data.scale;
                ctx.request_paint();
            }
            Event::MouseUp(mouse) => {
                match mouse.button {
                    MouseButton::Middle => {
                        data.middle_mouse_pressed = false;
                        data.init_cursor_pos = Point::ZERO;
                        data.init_camera_pos = Point::ZERO;
                    }
                    MouseButton::Left => {
                        data.left_mouse_pressed = false;
                        let mut body = data.proposed_body.clone();
                        let scale = 20.0;
                        let delta = ((data.cursor_pos.x - data.init_cursor_pos.x) / 1.0, (data.cursor_pos.y - data.init_cursor_pos.y) / 1.0);
                        body.x = data.init_cursor_pos.x / data.scale  - data.camera_pos.x;
                        body.y = data.init_cursor_pos.y / data.scale - data.camera_pos.y;
                        body.prev_x = body.x + delta.0 / scale; 
                        body.prev_y = body.y + delta.1 / scale;
                        if body.color == 6 {
                            body.fixed = true;
                        }
                        data.proposed_body = CelestialObject::new(0.0, 0.0, body.mass.clone());
                        data.bodies.borrow_mut().push(body);
                        data.init_cursor_pos = Point::ZERO;
                        data.trajectory.borrow_mut().clear();
                    }
                    _ => {}
                }
            }
            
            _ => {}
        };

        //Calculating the camera position
        if data.middle_mouse_pressed {
            let delta = (
                (data.cursor_pos.x - data.init_cursor_pos.x) / data.scale,
                (data.cursor_pos.y - data.init_cursor_pos.y) / data.scale,
            );
            data.camera_pos = Point::new(
                data.init_camera_pos.x + delta.0,
                data.init_camera_pos.y + delta.1,
            );
        } 

        // Calculating proposed trajectory

    }
    
    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        _lifestyle: &LifeCycle,
        _data: &Simulation,
        _env: &Env
    ) {
        ctx.register_for_focus();
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &Simulation,
        _data: &Simulation,
        _env: &Env
    ) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Simulation,
        _env: &Env,
    ) -> Size {
        bc.constrain((700.0, 500.0))
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        data: &Simulation,
        _env: &Env) {
        
        for body in (*data.bodies).borrow().iter() {
            let point = Circle::new(
                ((body.x + data.camera_pos.x) * data.scale, 
                 (body.y + data.camera_pos.y) * data.scale), 
                mass_to_radius(body.mass) * data.scale
            );
            ctx.fill(point, &ALLCOLORS[body.color]);
        }
        
        ctx.fill(Circle::new((data.cursor_pos.x, data.cursor_pos.y), mass_to_radius(data.proposed_body.mass) * data.scale), &ALLCOLORS[data.proposed_body.color]);

        if data.left_mouse_pressed {
            ctx.stroke(Line::new(data.cursor_pos.clone(), Point::new(data.init_cursor_pos.x, data.init_cursor_pos.y)), &Color::GRAY, 2.5); 
            
            if data.trajectory.borrow().len() > 1 {
                for i in 0..(data.trajectory.borrow().len() - 1){
                    ctx.stroke(
                        Line::new(
                        ((data.trajectory.borrow()[i].x + data.camera_pos.x) * data.scale,
                        (data.trajectory.borrow()[i].y + data.camera_pos.y) * data.scale),
                        ((data.trajectory.borrow()[i + 1].x + data.camera_pos.x) * data.scale,
                        (data.trajectory.borrow()[i + 1].y + data.camera_pos.y) * data.scale)
                        ),
                        &Color::GREEN,
                        3.0
                    );
                }
            }
        }
    }
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

        let num_points = 500;

        for num in 0..TRAJECTORY_POINTS * num_points {
            let mut net_force: (f64, f64) = (0.0, 0.0);
            let mut state = false;
            for i in 0..self.bodies.borrow().len() {
                let dist = ((self.bodies.borrow()[i].x - body.x).powf(2.0) + (self.bodies.borrow()[i].y - body.y).powf(2.0)).powf(0.5);
                if dist <= mass_to_radius(self.bodies.borrow()[i].mass) + mass_to_radius(body.mass) {
                    state = true;
                }
                let result = body.calculate_force(&self.bodies.borrow()[i]);
                net_force.0 += result.0;
                net_force.1 += result.1;
            }

            if state {
                break;
            }
            net_force.0 /= body.mass;
            net_force.1 /= body.mass;
            body.update_fields_from_force(&net_force, &self.dt);
            if num % 1 == 0 {
                self.trajectory.borrow_mut().push(Point::new(body.x, body.y));
            }
        }
    }
    pub fn update(&mut self) {
        let mut accs: Vec<(f64, f64)> = Vec::new();
        let mut merges: Vec<(usize, usize)> = Vec::new();

        for i in 0..self.bodies.borrow().len() {
            let mut net_force = (0.0, 0.0);


            for j in 0..self.bodies.borrow().len() {
                //Checking for merges
                if i == j {
                    continue;
                }

                let dist = ((self.bodies.borrow()[i].x - self.bodies.borrow()[j].x).powf(2.0) + (self.bodies.borrow()[i].y - self.bodies.borrow()[j].y).powf(2.0)).powf(0.5);

                if dist <= mass_to_radius(self.bodies.borrow()[i].mass) + mass_to_radius(self.bodies.borrow()[j].mass) {
                    merges.push((i, j))
                }

                let result = self.bodies.borrow()[i].calculate_force(&self.bodies.borrow()[j]);
                net_force.0 += result.0;
                net_force.1 += result.1;
            }
            net_force.0 /= self.bodies.borrow()[i].mass;
            net_force.1 /= self.bodies.borrow()[i].mass;

            if self.bodies.borrow()[i].fixed {
                accs.push((0.0, 0.0));
            } else {
                accs.push(net_force);
            }
        }

        for (i, body) in self.bodies.borrow_mut().iter_mut().enumerate() {
            body.update_fields_from_force(&accs[i], &self.dt);
        }

        let mut delete_idxs: Vec<usize> = Vec::new();

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

        delete_idxs.sort();

        for (i, ele) in delete_idxs.iter().enumerate() {
            self.bodies.borrow_mut().remove(ele - i);
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
        }
    }
}

impl CelestialObject {
    fn calculate_force(&self, source: &CelestialObject) -> (f64, f64) {
        let delta_x: f64 = source.x - self.x;
        let delta_y: f64 = source.y - self.y;

        let distance: f64 = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
        let force: f64 =  20.0 * (self.mass * source.mass) / (distance * distance);

        return (force * (delta_x / distance), force * (delta_y / distance));
    }

    fn update_fields_from_force(&mut self, acc: &(f64, f64), dt: &f64) {
        let prev_x = self.x;
        let prev_y = self.y;

        self.x = 2.0 * self.x - self.prev_x + acc.0 * dt * dt;
        self.y = 2.0 * self.y - self.prev_y + acc.1 * dt * dt;
        
        self.prev_x = prev_x;
        self.prev_y = prev_y;
    }

    fn new(x: f64, y:f64, mass:f64) -> Self {
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

    fn new_v0(x: f64, y:f64, mass:f64, v_x: f64, v_y: f64) -> Self {
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

fn main(){
    let bodies: Vec<CelestialObject> = Vec::new();

    // bodies.push(CelestialObject::new_v0(200.0, 200.0, 20.0, 0.5, 0.1));
    // bodies.push(CelestialObject::new_v0(300.0, 170.0, 1000.0, -0.1, 0.0));
    // bodies.push(CelestialObject::new_v0(200.0, 300.0, 1.0, -1.0, -1.0));

    let sim = Simulation::new(
        Rc::new(RefCell::new(bodies)),
        0.1,
    ) ;

    let window = WindowDesc::new(GravityDisplay {timer_id : TimerToken::INVALID}).title(
        LocalizedString::new("Gravity Sim")
            .with_placeholder("Gravity Sim")
    );

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(sim)
        .expect("launch failed");
}
