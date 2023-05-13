use gloo_timers::future::TimeoutFuture;
use rand::rngs::StdRng;
use rand::SeedableRng;
use sycamore::futures::spawn_local_scoped;
use sycamore::rt::JsCast;
use sycamore::{prelude::*, rt::JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::image::{generate, Direction, Rule, COLORS};
mod image;
mod wfc;

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    sycamore::render(|cx| {
        let display_rules = create_signal(
            cx,
            vec![
                Rule {
                    cols: (2, 1),
                    dir: Direction::Bottom,
                },
                Rule {
                    cols: (1, 2),
                    dir: Direction::TopRight,
                },
                Rule {
                    cols: (1, 0),
                    dir: Direction::TopLeft,
                },
            ],
        );
        let display_colors = create_signal(cx, vec![&COLORS[3], &COLORS[5], &COLORS[1]]);
        let canvas = create_node_ref(cx);
        let generating = create_signal(cx, true);
        let another = move || {
            generating.set(true);
            spawn_local_scoped(cx, async move {
                TimeoutFuture::new(0).await;
                let canvas: DomNode = canvas.get();
                let canvas: HtmlCanvasElement = canvas.to_web_sys().dyn_into().unwrap();
                let context: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                let mut rng = StdRng::from_entropy();
                let (rules, colors, img) = generate(WIDTH, HEIGHT, &mut rng);
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        let color = &img[y * WIDTH + x];
                        context.set_fill_style(&JsValue::from_str(&color.to_hex()));
                        context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                    }
                }
                display_colors.set(colors);
                display_rules.set(rules);
                generating.set(false);
            });
        };
        on_mount(cx, another);
        view! { cx,
            canvas(ref=canvas, width=WIDTH, height=HEIGHT)
            button(on:click=move |_| another(), disabled=*generating.get()){ "🔁 Another?" }
            p(id="generating", aria-hidden=!*generating.get()){ "Generating..." }
            h2 { "Colors:" }
            p { (display_colors.get().iter().map(|c| c.name).collect::<Vec<_>>().join("")) }
            h2 { "Rules:" }
            ul {
                Indexed(
                    iterable=display_rules,
                    view=move |cx, rule| view! { cx,
                        li { (rule.to_emoji(&display_colors.get())) }
                    }
                )
            }
        }
    });
}
