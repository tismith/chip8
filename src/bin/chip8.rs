//standard includes
extern crate chip8_tismith;
extern crate sdl2;
#[macro_use]
extern crate log;
use chip8_tismith::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::RenderTarget;
use std::fs::read;
use std::time::Duration;

const PIXEL_DIMENSION: u32 = 10;
const TICKS_PER_TIMER: u32 = 10;
const TICK_PERIOD: u32 = 1_000_000_000u32 / (TICKS_PER_TIMER * cpu::TIMER_FREQUENCY as u32);

fn main() -> Result<(), exitfailure::ExitFailure> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    utils::logging::configure_logger(&config)?;
    let mut cpu = cpu::Cpu::new();

    if let Some(path) = config.rom_path {
        let rom = read(path)?;
        cpu.load_rom(&rom);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP-8",
            PIXEL_DIMENSION * cpu::SCREEN_WIDTH as u32,
            PIXEL_DIMENSION * cpu::SCREEN_HEIGHT as u32,
        ).position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut counter = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => *cpu.key_mut(map_key(key)) = true,
                Event::KeyUp {
                    keycode: Some(key), ..
                } => *cpu.key_mut(map_key(key)) = false,
                _ => {}
            }
        }
        counter += 1;
        if counter == TICKS_PER_TIMER {
            //fire off CPU timers
            if cpu.tick_timers() {
                info!("BEEP!");
            }
            counter = 0;
        }
        cpu.tick();

        draw_screen(&mut canvas, &cpu)?;

        std::thread::sleep(Duration::new(0, TICK_PERIOD));
    }

    Ok(())
}

fn map_key(keycode: sdl2::keyboard::Keycode) -> u8 {
    match keycode {
        Keycode::Num0 => 0x00,
        Keycode::Num1 => 0x01,
        Keycode::Num2 => 0x02,
        Keycode::Num3 => 0x03,
        Keycode::Num4 => 0x04,
        Keycode::Num5 => 0x05,
        Keycode::Num6 => 0x06,
        Keycode::Num7 => 0x07,
        Keycode::Num8 => 0x08,
        Keycode::Num9 => 0x09,
        Keycode::A => 0x0A,
        Keycode::B => 0x0B,
        Keycode::C => 0x0C,
        Keycode::D => 0x0D,
        Keycode::E => 0x0E,
        Keycode::F => 0x0F,
        _ => 0xFF,
    }
}

fn draw_screen<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    cpu: &cpu::Cpu,
) -> Result<(), failure::Error> {
    for (i, filled) in cpu.screen().iter().enumerate() {
        if *filled {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
        } else {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
        }

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
