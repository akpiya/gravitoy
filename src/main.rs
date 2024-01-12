mod object;
mod simulation;

use simulation::{Simulation, mass_to_radius};
use object::CelestialObject;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;
use std::sync::RwLock;
use std::rc::Rc;
use druid::kurbo::{Size, Circle, Line};
use druid::widget::prelude::*;
use druid::{
    AppLauncher, LocalizedString, WindowDesc, TimerToken,
    Color, Point, MouseButton, Code
};


static TIMER_INTERVAL: Duration = Duration::from_millis(10);
const ALLCOLORS: [Color; 7] = [Color::RED, Color::BLUE, Color::GREEN, Color::PURPLE, Color::YELLOW, Color::WHITE, Color::BLACK];

struct GravityDisplay {
    timer_id: TimerToken,
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
                    Code::KeyW => data.up = true,
                    Code::KeyA => data.left = true,
                    Code::KeyS => data.down = true,
                    Code::KeyD => data.right = true,
                    Code::Space => data.play = !data.play,
                    _ => {}
                }
            }
            Event::KeyUp(button) => {
                match button.code {
                    Code::KeyW => data.up = false,
                    Code::KeyA => data.left = false,
                    Code::KeyS => data.down = false,
                    Code::KeyD => data.right = false,
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
                    MouseButton::Right => {
                        let cursor = data.cursor_pos.clone();
                        
                        let mut idx = -1;
                        for (i, body) in data.bodies.borrow().iter().enumerate() {
                            let dist = (((body.x + data.camera_pos.x) * data.scale - cursor.x).powf(2.0) + ((body.y + data.camera_pos.y) * data.scale - cursor.y).powf(2.0)).powf(0.5);
                            if dist <= mass_to_radius(body.mass) {
                                idx = i as i32;
                                break;
                            }
                        }
                        if idx != -1 {
                            data.bodies.borrow_mut().remove(idx as usize);
                        }
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

        if data.up {
            data.camera_pos.y += 1.0 / data.scale * 5.0;
        }
        if data.left {
            data.camera_pos.x += 1.0 / data.scale * 5.0;
        }
        if data.right {
            data.camera_pos.x -= 1.0 / data.scale * 5.0;
        }
        if data.down {
            data.camera_pos.y -= 1.0 / data.scale * 5.0;
        }
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
        //Changing background color
        let background = Color::rgb8(60, 60, 60);
        let size = ctx.size();
        ctx.fill(size.to_rect(), &background);
        
        for body in (*data.bodies).borrow().iter() {
            let point = Circle::new(
                ((body.x + data.camera_pos.x) * data.scale, 
                 (body.y + data.camera_pos.y) * data.scale), 
                simulation::mass_to_radius(body.mass) * data.scale
            );
            ctx.fill(point, &ALLCOLORS[body.color]);
        }
        
        ctx.fill(Circle::new((data.cursor_pos.x, data.cursor_pos.y), simulation::mass_to_radius(data.proposed_body.mass) * data.scale), &ALLCOLORS[data.proposed_body.color]);

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
