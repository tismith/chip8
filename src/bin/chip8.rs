//standard includes
extern crate chip8_tismith;
extern crate sdl2;
use chip8_tismith::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

fn main() -> Result<(), exitfailure::ExitFailure> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    utils::logging::configure_logger(&config)?;
    let mut _cpu = cpu::Cpu::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8", 640, 320)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // The rest of the game loop goes here...
    }

    Ok(())
}


