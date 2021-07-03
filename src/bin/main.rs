use ::rand::{thread_rng, Fill};
use chips::{Algorithm, Anneal, ConwaysLife};
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

    let mut world1: [bool; HEIGHT * WIDTH] = [false; HEIGHT * WIDTH];
    let mut world2: [bool; HEIGHT * WIDTH] = [false; HEIGHT * WIDTH];

    world1
        .try_fill(&mut rng)
        .expect("could not init random state");

    let (mut fresh, mut stale) = (&mut world1, &mut world2);

    let mut image = Image::gen_image_color(WIDTH as u16, HEIGHT as u16, BLACK);
    let texture = Texture2D::from_image(&image);

    loop {
        clear_background(BLACK);

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();
            let (x, y) = (x as usize, y as usize);
            fresh[x + (y * WIDTH)] = true;
            println!("mouse pressed at {:?}", (x, y));
        }

        let simulate = !is_key_down(KeyCode::Space);

        if is_key_released(KeyCode::Enter) {
            fresh.try_fill(&mut rng).expect("could not refresh state");
        }

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                if fresh[i + (j * WIDTH)] {
                    image.set_pixel(i as u32, j as u32, WHITE);
                } else {
                    image.set_pixel(i as u32, j as u32, BLACK);
                }

                if simulate {
                    stale[i + (j * WIDTH)] =
                        Anneal::next_state((i, j).into(), fresh, (WIDTH, HEIGHT));
                }
            }
        }

        if simulate {
            let temp = fresh;
            fresh = stale;
            stale = temp;
        }

        texture.update(&image);
        draw_texture(texture, 0.0, 0.0, WHITE);

        next_frame().await
    }
}
