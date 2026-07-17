#![allow(linker_messages)]

use std::env;
use std::error::Error;

use lab1_poligono::rasterizer::{export_outputs, render_scene, HEIGHT, WIDTH};
use raylib::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let export_only = env::args().skip(1).any(|arg| arg == "--export-only");
    let framebuffer = render_scene();

    export_outputs(&framebuffer)?;

    if export_only {
        return Ok(());
    }

    let rgba = framebuffer.as_rgba8();
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 1 - Scanline polygon fill")
        .build();

    rl.set_target_fps(60);

    let image = Image::gen_image_color(WIDTH as i32, HEIGHT as i32, Color::WHITE);
    let mut texture = rl.load_texture_from_image(&thread, &image)?;
    texture.update_texture(&rgba)?;

    while !rl.window_should_close() {
        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::RAYWHITE);
        drawing.draw_texture(&texture, 0, 0, Color::WHITE);
    }

    Ok(())
}
