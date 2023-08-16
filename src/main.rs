use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;
use druid::Data;
use druid::kurbo::{Size, Circle};
use druid::widget::prelude::*;
use druid::{
    AppLauncher, LocalizedString, WindowDesc, TimerToken, Color, Point
};

static TIMER_INTERVAL: Duration = Duration::from_millis(10);
struct GravityDisplay {
    timer_id: TimerToken,
}

#[derive(Clone, Data)]
struct Simulation {
    dt: f64,
    bodies: Rc<RefCell<Vec<CelestialObject>>>,
    cursor_pos: Point,
    cursor_on_window: bool, 
}


#[derive(Clone, PartialEq, Data)]
struct CelestialObject {
    x: f64,
    y: f64,
    mass: f64,
    prev_x: f64,
    prev_y: f64,
}

impl Widget<Simulation> for GravityDisplay {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Simulation, _env: &Env) {
        data.cursor_on_window = true;

        match event {
            Event::WindowConnected => {
                self.timer_id = ctx.request_timer(TIMER_INTERVAL);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    ctx.request_paint();
                    data.update();
                    self.timer_id = ctx.request_timer(TIMER_INTERVAL);
                }
            }
            Event::MouseMove(event) => {
                data.cursor_pos = event.window_pos.clone();
                ctx.request_paint();
            }
            _ => {}
        }        
    }
    
    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &Simulation,
        _env: &Env
    ) {}

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
        
        let radius = 100.0; //radius = [3.0, 100.0]
        
        for body in (*data.bodies).borrow().iter() {
            let point = Circle::new((body.x, body.y), radius);
            ctx.fill(point, &Color::RED);
        }
        
        if data.cursor_on_window {
            ctx.fill(Circle::new((data.cursor_pos.x, data.cursor_pos.y),5.0), &Color::WHITE);
        }
    }
}

impl Simulation {
    pub fn update(&mut self) {
        let mut accs: Vec<(f64, f64)> = Vec::new();
        for i in 0..self.bodies.borrow().len() {
            let mut net_force = (0.0, 0.0);

            for j in 0..self.bodies.borrow().len() {
                if i != j {
                    let result = self.bodies.borrow()[i].calculate_force(&self.bodies.borrow()[j]);
                    net_force.0 += result.0;
                    net_force.1 += result.1;
                }
            }
            net_force.0 /= self.bodies.borrow()[i].mass;
            net_force.1 /= self.bodies.borrow()[i].mass;

            accs.push(net_force);
        }

        for (i, body) in self.bodies.borrow_mut().iter_mut().enumerate() {
            body.update_fields_from_force(&accs[i], &self.dt);
        }
    }

    pub fn new(bodies: Rc<RefCell<Vec<CelestialObject>>>, dt: f64) -> Self {
        Self {
            bodies,
            dt,
            cursor_pos: Point::new(0.0, 0.0),
            cursor_on_window: false,
        }
    }
}

impl CelestialObject {
    fn calculate_force(&self, source: &CelestialObject) -> (f64, f64) {
        let delta_x: f64 = source.x - self.x;
        let delta_y: f64 = source.y - self.y;

        let distance: f64 = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
        let force: f64 =  2.0 * (self.mass * source.mass) / (distance * distance);

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
        }
    }

    fn new_v0(x: f64, y:f64, mass:f64, v_x: f64, v_y: f64) -> Self {
        Self {
            x: x + v_x,
            y: y + v_y,
            mass,
            prev_x: x,
            prev_y: y
        }
    }
}

fn main(){
    let mut bodies: Vec<CelestialObject> = Vec::new();

    bodies.push(CelestialObject::new_v0(200.0, 200.0, 20.0, 0.5, 0.1));
    bodies.push(CelestialObject::new_v0(300.0, 170.0, 1000.0, -0.1, 0.0));
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