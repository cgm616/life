#![allow(dead_code)]

use core::f32;

use chips::{Automata, LifeLike};

use ::rand::{thread_rng, Rng};
use bitvec::prelude::*;
use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};

const HEIGHT: usize = 512;
const WIDTH: usize = 1024;
const INITIAL_RULE: &str = "B3/S23";

fn window_conf() -> Conf {
    Conf {
        window_title: "chips".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: false,
        high_dpi: false,
        ..Default::default()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Speed {
    Normal,
    Overclocked(usize),
    Underclocked(usize),
}

impl Speed {
    fn new() -> Self {
        Self::Normal
    }

    fn new_overclocked(speed: usize) -> Self {
        if speed == 0 || speed == 1 {
            Self::Normal
        } else {
            Self::Overclocked(speed)
        }
    }

    fn new_underclocked(speed: usize) -> Self {
        if speed == 0 || speed == 1 {
            Self::Normal
        } else {
            Self::Underclocked(speed)
        }
    }
}

struct World {
    state: State,
    machine: LifeLike,
    speed: Speed,
    counter: usize,
}

impl World {
    fn new(initial_rule: &str) -> Self {
        World {
            state: State::Normal,
            machine: LifeLike::new(initial_rule).unwrap(),
            speed: Speed::new(),
            counter: 0,
        }
    }

    fn new_rule(&mut self, new: &str) -> Result<(), &'static str> {
        self.machine = LifeLike::new(&new.trim())?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    /// The main state of the simulation. The machine should simulate
    Normal,
    Paused,
    Settings,
}

impl State {
    fn next(&self, input: KeyCode) -> State {
        match (self, input) {
            (State::Normal | State::Paused, KeyCode::Escape) => State::Settings,
            (State::Normal, KeyCode::Space) => State::Paused,
            (State::Settings, KeyCode::Escape) => State::Normal,
            (State::Paused, KeyCode::Space) => State::Normal,
            _ => *self,
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut rng = thread_rng();

    let height: usize = screen_height() as usize;
    let width = screen_width() as usize;
    let resolution = 2usize;
    let grid_height = height / resolution;
    let grid_width = width / resolution;
    let world_size = grid_height * grid_width;

    let mut buffer1: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
    let mut buffer2: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
    let mut change_buffer: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
    buffer1.resize(world_size, false);
    buffer2.resize(world_size, false);
    change_buffer.resize(world_size, false);

    fill_random(&mut buffer1, &mut rng);

    let (mut fresh, mut stale) = (&mut *buffer1, &mut *buffer2);

    // let skin = make_skin();

    let mut world = World::new(INITIAL_RULE);
    let mut rule_input = INITIAL_RULE.to_owned();
    let mut speed_input = "1".to_owned();

    loop {
        clear_background(BLACK); // clear all previous drawings

        // render the fresh information
        render_bits(fresh, (grid_width, grid_height, resolution));

        // process possible state changes
        match get_last_key_pressed() {
            Some(KeyCode::N) => fill_random(fresh, &mut rng),
            Some(input) => world.state = world.state.next(input),
            None => {}
        }

        // proceed according to current state
        match world.state {
            State::Normal => match world.speed {
                Speed::Normal => {
                    simulate_step(
                        fresh,
                        stale,
                        &mut change_buffer,
                        (grid_width, grid_height),
                        &world.machine,
                    );
                    std::mem::swap(&mut fresh, &mut stale);
                }
                Speed::Overclocked(speed) => {
                    for _ in 0..speed {
                        simulate_step(
                            fresh,
                            stale,
                            &mut change_buffer,
                            (grid_width, grid_height),
                            &world.machine,
                        );
                        std::mem::swap(&mut fresh, &mut stale);
                    }
                }
                Speed::Underclocked(speed) => {
                    if world.counter == 0 {
                        simulate_step(
                            fresh,
                            stale,
                            &mut change_buffer,
                            (grid_width, grid_height),
                            &world.machine,
                        );
                        std::mem::swap(&mut fresh, &mut stale);
                    }
                    world.counter = (world.counter + 1) % speed;
                }
            },
            State::Paused => {
                // if paused, don't do anything unless the right arrow key was pressed
                if is_key_pressed(KeyCode::Right) {
                    simulate_step(
                        fresh,
                        stale,
                        &mut change_buffer,
                        (grid_width, grid_height),
                        &world.machine,
                    );
                    std::mem::swap(&mut fresh, &mut stale);
                }
            }
            State::Settings => {
                //root_ui().push_skin(&skin);
                widgets::Window::new(hash!(), vec2(20., 20.), vec2(300., 110.))
                    .movable(true)
                    .label("Settings")
                    .ui(&mut root_ui(), |ui| {
                        ui.input_text(hash!(), "Rule", &mut rule_input);
                        if ui.button(None, "Update rule") {
                            match world.new_rule(&rule_input) {
                                Ok(()) => info!("changed rule to {}", rule_input.trim()),
                                Err(e) => error!("could not change rule! error:\n  {}", e),
                            }
                        }
                        match ui.tabbar(
                            hash!(),
                            vec2(280., 20.),
                            &["Normal", "Overclocked", "Underclocked"],
                        ) {
                            0 => world.speed = Speed::new(),
                            1 => {
                                world.speed =
                                    Speed::new_overclocked(speed_input.parse().unwrap_or(1))
                            }
                            2 => {
                                world.speed =
                                    Speed::new_underclocked(speed_input.parse().unwrap_or(1))
                            }
                            _ => {}
                        }
                        ui.input_text(hash!(), "Speed", &mut speed_input);
                    });
                //root_ui().pop_skin();
            }
        }

        next_frame().await
    }
}

fn simulate_step<O: BitOrder, T: BitStore>(
    fresh: &BitSlice<O, T>,
    stale: &mut BitSlice<O, T>,
    change_buffer: &mut BitVec<O, T>,
    size: (usize, usize),
    machine: &LifeLike,
) {
    // empty the change buffer and fill it with the new changes
    change_buffer.clear();
    change_buffer.extend(fresh.iter().zip(stale.iter()).map(|(a, b)| *a ^ *b));

    // run the simulation one step
    machine.update(fresh, stale, &change_buffer, size);
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

//const FONT: &[u8; 124236] = include_bytes!("../../assets/font/Rubik-Regular.ttf");
/*
fn make_skin() -> Skin {
    let label_style = root_ui()
        .style_builder()
        .font(FONT)
        .unwrap()
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .font_size(20)
        .build();

    let window_style = root_ui()
        .style_builder()
        .background_margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
        .color(Color::from_rgba(255, 255, 255, 150))
        .font(FONT)
        .unwrap()
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .font_size(20)
        .build();

    let button_style = root_ui()
        .style_builder()
        .margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
        .font(FONT)
        .unwrap()
        .text_color(Color::from_rgba(255, 255, 255, 255))
        .font_size(20)
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .margin(RectOffset::new(20., 20., 20., 20.))
        .background_margin(RectOffset::new(0., 0., 0., 0.))
        .font(FONT)
        .unwrap()
        .text_color(Color::from_rgba(0, 0, 0, 255))
        .color_selected(Color::from_rgba(190, 190, 190, 255))
        .font_size(20)
        .build();

    Skin {
        editbox_style,
        window_style,
        button_style,
        label_style,
        ..root_ui().default_skin()
    }
*/
