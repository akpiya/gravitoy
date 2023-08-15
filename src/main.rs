use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;
use druid::Data;
use druid::kurbo::{Size, Circle};
use druid::widget::prelude::*;
use druid::{
    AppLauncher, LocalizedString, WindowDesc, TimerToken, Color
};

static TIMER_INTERVAL: Duration = Duration::from_millis(25);
struct GravityDisplay {
    timer_id: TimerToken,
}

#[derive(Clone, Data)]
struct Simulation {
    dt: f64,
    bodies: Rc<RefCell<Vec<CelestialObject>>>,
}


#[derive(Clone, PartialEq, Data)]
struct CelestialObject {
    x: f64,
    y: f64,
    mass: f64,
    v_x: f64,
    v_y: f64,
}

impl Widget<Simulation> for GravityDisplay {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Simulation, _env: &Env) {
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
        bc.constrain((100.0, 100.0))
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        data: &Simulation,
        _env: &Env) {
        
        let radius = 20.0;
        
        for body in (*data.bodies).borrow().iter() {
            let point = Circle::new((body.x, body.y), radius);
            ctx.fill(point, &Color::RED);
        }
    }
}

impl Simulation {
    pub fn update(&mut self) {
        let mut forces: Vec<(f64, f64)> = Vec::new();
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

            forces.push(net_force);
        }

        for (i, body) in self.bodies.borrow_mut().iter_mut().enumerate() {
            body.update_fields_from_force(&forces[i], &self.dt);
        }
    }
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
}

fn main(){
    let mut bodies: Vec<CelestialObject> = Vec::new();

    bodies.push(CelestialObject { x: 100.0, y: 100.0, mass: 1000.0, v_x: 0.0, v_y: 0.0});
    bodies.push(CelestialObject { x: 250.0, y: 150.0, mass: 300.0, v_x: 10.0, v_y: 0.0});
    bodies.push(CelestialObject { x: 200.0, y: 100.0, mass: 300.0, v_x: 0.0, v_y: 0.0});

    let sim = Simulation {
        bodies: Rc::new(RefCell::new(bodies)),
        dt: 0.25,
    };

    let window = WindowDesc::new(GravityDisplay {timer_id : TimerToken::INVALID}).title(
        LocalizedString::new("gravity_sim")
            .with_placeholder("placeholder")
    );

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(sim)
        .expect("launch failed");
}