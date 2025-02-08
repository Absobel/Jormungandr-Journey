#![feature(let_chains)]

mod game;

use game::Draw;

use ruscii::app::{App, Config, State};
use ruscii::drawing::Pencil;
use ruscii::drawing::RectCharset;
use ruscii::gui::FPSCounter;
use ruscii::keyboard::{Key, KeyEvent};
use ruscii::spatial::Vec2;
use ruscii::terminal::Color;
use ruscii::terminal::Window;

fn main() {
    let mut app = App::config(Config::new().fps(20));
    let size = app.window().size() - Vec2::xy(1, 1);
    let mut fps_counter = FPSCounter::default();

    // dx  dy dz = 1
    let max_x = size.x as isize / 2;
    let max_y = max_x;
    let max_z = size.y as isize;
    let mut grid = game::Grid::empty((max_x, max_y, max_z));
    grid.set((max_x / 2, max_y / 2, 1), game::Cell::Food)
        .unwrap();
    for x in 0..max_x {
        for y in 0..max_y {
            grid.set((x, y, 0), game::Cell::Block).unwrap();
        }
    }
    let mut game = game::GameState::new((0, 0, 1), grid);

    app.run(|app_state: &mut State, window: &mut Window| {
        for key_event in app_state.keyboard().last_key_events() {
            if let KeyEvent::Pressed(Key::Esc) = key_event {
                app_state.stop()
            }
        }

        let dir = app_state
            .keyboard()
            .last_key_events()
            .iter()
            .rev()
            .find_map(|event| event.pressed())
            .unwrap_or(Key::Unknown)
            .into();

        fps_counter.update();
        game.update(dir).expect("PERDU");

        let mut pencil = Pencil::new(window.canvas_mut());
        let pencil = pencil
            .set_origin(size / 4)
            .set_foreground(Color::Grey)
            .draw_rect(&RectCharset::double_lines(), Vec2::zero(), size)
            .set_foreground(Color::Yellow);

        game.draw(pencil);
    });
}
