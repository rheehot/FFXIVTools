use nalgebra::Point3;
use tokio::runtime::Runtime;
use winit::{
    event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

use ffxiv_model::Character;
use renderer::{Camera, Renderer};

fn main() {
    let mut rt = Runtime::new().unwrap();
    let event_loop = EventLoop::new();

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("test");
    let window = builder.build(&event_loop).unwrap();

    let size = window.inner_size();
    let mut renderer = rt.block_on(async { Renderer::new(&window, size.width, size.height).await });
    let mut character = rt.block_on(async { Character::new(&renderer).await });

    let camera = Camera::new(Point3::new(0.0, 0.8, 2.5), Point3::new(0.0, 0.8, 0.0));
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            event::Event::RedrawRequested(_) => {
                rt.block_on(async { renderer.render(&mut character.model, &camera).await });
            }
            _ => {}
        }
    });
}