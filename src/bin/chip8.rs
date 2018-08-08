//standard includes
extern crate chip8_tismith;
extern crate sdl2;
//extern crate failure;
use chip8_tismith::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::RenderTarget;
use std::time::Duration;

const PIXEL_DIMENSION: u32 = 10;

fn main() -> Result<(), exitfailure::ExitFailure> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    utils::logging::configure_logger(&config)?;
    let mut cpu = cpu::Cpu::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP-8",
            cpu::SCREEN_WIDTH as u32,
            cpu::SCREEN_HEIGHT as u32,
        ).position_centered()
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
        draw_screen(&mut canvas, &cpu)?;
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn draw_screen<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    cpu: &cpu::Cpu,
) -> Result<(), failure::Error> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, _) in cpu.screen().iter().enumerate().filter(|(_,&p)| p) {
        let x = i % cpu::SCREEN_WIDTH;
        let y = i / cpu::SCREEN_WIDTH;
        canvas
            .fill_rect(Rect::new(
                (PIXEL_DIMENSION * x as u32) as i32,
                (PIXEL_DIMENSION * y as u32) as i32,
                PIXEL_DIMENSION,
                PIXEL_DIMENSION,
            )).map_err(failure::err_msg)?;;
    }

    canvas.present();
    Ok(())
}
