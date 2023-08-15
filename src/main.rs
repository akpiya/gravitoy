use std::{thread, time};
use std::rc::Rc;
use druid::Data;
use druid::kurbo::Size;
use druid::piet::InterpolationMode;
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, Data, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};

struct GravityDisplay;

#[derive(Clone, Data)]
struct Simulation {
    dt: f64,
    bodies: Rc<Vec<CelestialObject>>,
}


#[derive(Clone, PartialEq, Data)]
struct CelestialObject {
    x: f64,
    y: f64,
    mass: f64,
    v_x: f64,
    v_y: f64,
    id: u32
}

impl Widget<Simulation> for GravityDisplay {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Simulation, _env: &Env) {
        
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
        ctx: &mut UpdateCtx,
        _old_data: &Simulation,
        _data: &Simulation,
        _env: &Env
    ) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Simulation,
        _env: &Env,
    ) -> Size {
        return bc.max();
    }

    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx,
        data: &Simulation,
        _env: &Env) {
        
    }

    

}

impl GravityDisplay {

    fn time_step(&mut self, mut bodies: Rc<Vec<CelestialObject>> ) {
        for body in bodies.iter() {
            println!("({}, {})", body.x, body.y);
        }

        println!();
        for body in bodies.iter_mut() {
            body.update_state(&(*bodies), &((*data).dt));
        }

        thread::sleep(time::Duration::from_millis(500));
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

    fn update_state(&mut self, bodies: &Vec<CelestialObject>, dt: &f64) {
        let mut net_force = (0.0, 0.0);
        for body in bodies.iter() {
            if self.id != body.id {
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

    bodies.push(CelestialObject { x: 0.0, y: 0.0, mass: 1000.0, v_x: 0.0, v_y: 0.0, id: 0});
    bodies.push(CelestialObject { x: 500.0, y: 0.0, mass: 300.0, v_x: 0.0, v_y: 0.0, id: 1});
    bodies.push(CelestialObject { x: 1000.0, y: 100.0, mass: 300.0, v_x: 0.0, v_y: 0.0, id: 2});
    //bodies.push(CelestialObject { x: 500.0, y: 500.0, mass: 300.0, v_x: 0.5, v_y: 0.5 });
    let copy_bodies = bodies.clone();

    loop {

        
    }

    let window = WindowDesc::new(sim).title(LocalizedString::new("Gravity"));
        AppLauncher::with_window(window)
            .log_to_console()
            .launch("something")
            .expect("launch failed");
}
