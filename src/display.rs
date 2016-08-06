use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

pub const PIXEL_SIZE: u32 = 8;

pub fn render(fb: &[u8], renderer: &mut Renderer) {
    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();
    for (i, val) in fb.iter().enumerate() {
        let x = ((i as i32) % 64) * (PIXEL_SIZE as i32);
        let y = ((i as i32) / 64) * (PIXEL_SIZE as i32);

        if *val == 0 {
            renderer.set_draw_color(Color::RGB(0, 0, 0));
        } else {
            renderer.set_draw_color(Color::RGB(255, 255, 255));
        }

        let pixel = Rect::new(x, y, PIXEL_SIZE, PIXEL_SIZE);

        match renderer.fill_rect(pixel) {
            Ok(_) => { },
            Err(err) => debug!("Couldn't fill pixel: {}", err),
        }
    }
    renderer.present();
}
