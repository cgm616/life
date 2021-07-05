use core::f32;

use chips::{Automata, LifeLike};

use ::rand::{thread_rng, Rng};
use bitvec::prelude::*;
use macroquad::prelude::*;

const HEIGHT: usize = 512;
const WIDTH: usize = 1024;
// const RESOLUTION: usize = 8;
// const M_HEIGHT: usize = HEIGHT / RESOLUTION;
// const M_WIDTH: usize = WIDTH / RESOLUTION;

fn window_conf() -> Conf {
    Conf {
        window_title: "chips".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut rng = thread_rng();

    let height: usize = screen_height() as usize;
    let width = screen_width() as usize;
    let resolution = 8usize;
    let grid_height = height / resolution;
    let grid_width = width / resolution;

    // let mut world1: BitArr!(for grid_height * grid_width) = bitarr![0; grid_height * grid_width];
    // let mut world2: BitArr!(for grid_height * grid_width) = bitarr![0; grid_height * grid_width];

    let mut world1: BitVec<Lsb0, usize> = BitVec::with_capacity(grid_width * grid_height);
    let mut world2: BitVec<Lsb0, usize> = BitVec::with_capacity(grid_width * grid_height);
    let mut changes: BitVec<Lsb0, usize> = BitVec::with_capacity(grid_width * grid_height);
    world1.resize(grid_width * grid_height, false);
    world2.resize(grid_width * grid_height, false);
    changes.resize(grid_width * grid_height, false);

    fill_random(&mut world1, &mut rng);

    let (mut fresh, mut stale) = (&mut world1, &mut world2);

    // let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
    // let texture = Texture2D::from_image(&image);

    let life = LifeLike::new("B4678/S35678").unwrap();

    loop {
        clear_background(BLACK);

        if is_key_released(KeyCode::Enter) {
            fill_random(fresh, &mut rng);
        }

        changes.clear();
        changes.extend(fresh.iter().zip(stale.iter()).map(|(a, b)| *a ^ *b));

        render_bits(
            fresh,
            /*&changes,*/ (grid_width, grid_height, resolution),
        );

        if !is_key_down(KeyCode::Space) || is_key_released(KeyCode::Right) {
            life.update(fresh, stale, &changes, (grid_width, grid_height));
            std::mem::swap(&mut fresh, &mut stale);
        }

        // texture.update(&image);
        // draw_texture(texture, 0.0, 0.0, WHITE);

        next_frame().await
    }
}

fn fill_random<O: BitOrder, T: BitStore>(slice: &mut BitSlice<O, T>, rng: &mut impl Rng) {
    slice.iter_mut().for_each(|i| i.set(rng.gen()));
}

fn render_bits<O: BitOrder, T: BitStore>(
    bits: &BitSlice<O, T>,
    // changes: &BitSlice<O, T>,
    window_info: (usize, usize, usize),
) {
    for index in bits.iter_ones() {
        /* image.set_pixel(
            ((index % M_WIDTH) * RESOLUTION) as u32,
            ((index / M_WIDTH) * RESOLUTION) as u32,
            if bits[index] { WHITE } else { BLACK },
        );*/
        let upper_left = (
            (index % window_info.0) * window_info.2,
            (index / window_info.0) * window_info.2,
        );
        draw_rectangle(
            upper_left.0 as f32,
            upper_left.1 as f32,
            window_info.2 as f32,
            window_info.2 as f32,
            WHITE,
        );
    }
}
