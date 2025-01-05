use crate::gba::cpu::CPU;
use crate::ppu::PPU;
use crate::gba::system::SystemMemory;
use std::time::Instant;
use tracing::{event, Level};
use tracing_subscriber::{filter, reload::Handle, Registry};
use tracing_subscriber::filter::LevelFilter;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::WindowBuilder,
    keyboard::KeyCode,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 240;
const HEIGHT: u32 = 160;

pub fn run_gui(mut cpu: CPU, mut memory: SystemMemory, reload_handle: Handle<LevelFilter, Registry>)  -> Result<(), Box<dyn std::error::Error> >{
    event!(Level::INFO, "Runing GUI");
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let mut ppu = PPU::default();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("Crusty Gameboy")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let _res = event_loop.run(|event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let current = Instant::now();
                loop {
                    cpu.tick(&mut memory);
                    if ppu.tick(cpu.cycles(), &mut memory) {
                        break;
                    }

                }
                {
                    let ppu_buffer = ppu.get_next_frame(&mut memory);
                    let frame = pixels.frame_mut();
                    let mut i = 0;
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel[0] = ppu_buffer[i];
                        pixel[1] = ppu_buffer[i + 1];
                        pixel[2] = ppu_buffer[i + 2];
                        pixel[3] = ppu_buffer[i + 3];
                        i += 4;
                    }
                }
                let _ = pixels.render();
                // TODO: This seems wrong?
                let dt = Instant::now() - current;
                if dt.as_secs_f64() > 0.0 {
                    std::thread::sleep(dt);
                }

            },
            _ => (),
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
            if input.key_pressed(KeyCode::Space) {
                let _ = reload_handle.modify(|filter| *filter = filter::LevelFilter::DEBUG);
            }
            window.request_redraw();
        }
    });
    Ok(())
}

