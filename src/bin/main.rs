use chips::{Automata, ConwaysLife};

use ::rand::{thread_rng, Rng};
use bitvec::prelude::*;
use macroquad::prelude::*;

const HEIGHT: usize = 600;
const WIDTH: usize = 800;

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

    let mut world1: BitArr!(for HEIGHT * WIDTH) = bitarr![0; HEIGHT * WIDTH];
    let mut world2: BitArr!(for HEIGHT * WIDTH) = bitarr![0; HEIGHT * WIDTH];

    fill_random(&mut world1, &mut rng);

    let (mut fresh, mut stale) = (&mut world1, &mut world2);

    let mut image = Image::gen_image_color(WIDTH as u16, HEIGHT as u16, BLACK);
    let texture = Texture2D::from_image(&image);

    let mut counter = 0u16;

    loop {
        clear_background(BLACK);
        counter += 1;

        if counter == 20 {
            counter = 0;
            info!("current fps: {}", get_fps());
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();
            let (x, y) = (x as usize, y as usize);
            fresh.set(x + (WIDTH * y), true);
            println!("mouse pressed at {:?}", (x, y));
        }

        let simulate = !is_key_down(KeyCode::Space);

        if is_key_released(KeyCode::Enter) {
            fill_random(fresh, &mut rng);
        }

        let changes = *fresh ^ *stale;

        render_bits(fresh, &changes, &mut image);

        if simulate {
            ConwaysLife::update(fresh, stale, &changes, (WIDTH, HEIGHT));
            let temp = fresh;
            fresh = stale;
            stale = temp;
        }

        texture.update(&image);
        draw_texture(texture, 0.0, 0.0, WHITE);

        next_frame().await
    }
}

fn fill_random<O: BitOrder, T: BitStore>(slice: &mut BitSlice<O, T>, rng: &mut impl Rng) {
    slice.iter_mut().for_each(|i| i.set(rng.gen()));
}

fn render_bits<O: BitOrder, T: BitStore>(
    bits: &BitSlice<O, T>,
    changes: &BitSlice<O, T>,
    image: &mut Image,
) {
    for index in changes.iter_ones() {
        image.set_pixel(
            (index % image.width()) as u32,
            (index / image.width()) as u32,
            if bits[index] { WHITE } else { BLACK },
        );
    }
}
