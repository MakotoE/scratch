use super::*;
use crate::app::WINDOW_SIZE;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::{canvas_const, CanvasCoordinate};
use crate::event_sender::EventSender;
use crate::file::ScratchFile;
use crate::vm::VM;
use conrod_core::image::Id;
use conrod_core::position::Relative;
use conrod_core::widget::button::Flat;
use conrod_core::widget::Button;
use conrod_core::{Borderable, Color, Colorable, Labelable, UiCell};
use conrod_core::{Positionable, Sizeable, Widget};
use graphics::Context;
use graphics::{rectangle, Transformed};
use input::{mouse, Motion};
use piston_window::{G2d, G2dTextureContext, Glyphs, Input};

pub struct Interface {
    ids: Ids,
    green_flag_image: Id,
    stop_image: Id,
    vm: VM,
    pause_state: PauseState,
    event_sender: EventSender,
}

widget_ids! {
    pub struct Ids {
        green_flag_button,
        stop_button,
        pause_continue_button,
        step_button,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PauseState {
    Paused,
    Running,
}

impl Interface {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        scratch_file: ScratchFile,
        ids: Ids,
        green_flag_image: Id,
        stop_image: Id,
    ) -> Result<Self> {
        let broadcaster = Broadcaster::new();
        let vm = VM::new(texture_context, scratch_file, broadcaster.clone()).await?;
        Ok(Self {
            ids,
            green_flag_image,
            stop_image,
            vm,
            pause_state: PauseState::Paused,
            event_sender: EventSender::new(broadcaster),
        })
    }

    pub async fn widgets(&mut self, ui_cell: &mut UiCell<'_>) {
        let green_flag_event = Button::image(self.green_flag_image)
            .top_left_with_margins(10.0, 25.0)
            .w_h(30.0, 30.0)
            .image_color_with_feedback(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(self.ids.green_flag_button, ui_cell);

        if green_flag_event.was_clicked() {
            self.vm.continue_().await;
            self.pause_state = PauseState::Running;
        }

        let stop_flag_event = Button::image(self.stop_image)
            .top_left_with_margins(10.0, 70.0)
            .w_h(30.0, 30.0)
            .image_color_with_feedback(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(self.ids.stop_button, ui_cell);

        if stop_flag_event.was_clicked() {
            self.vm.stop().await;
        }

        let pause_button_text = match self.pause_state {
            PauseState::Paused => "Continue",
            PauseState::Running => "Pause",
        };
        let pause_continue_event =
            Interface::button(19.0, pause_button_text).set(self.ids.pause_continue_button, ui_cell);
        if pause_continue_event.was_clicked() {
            match self.pause_state {
                PauseState::Paused => {
                    self.vm.continue_().await;
                    self.pause_state = PauseState::Paused;
                }
                PauseState::Running => {
                    self.vm.pause().await;
                    self.pause_state = PauseState::Paused;
                }
            }
        }

        let step_event = Interface::button(155.0, "Step").set(self.ids.step_button, ui_cell);
        if step_event.was_clicked() {
            self.vm.step().await;
        }
    }

    fn button(left: f64, label: &str) -> Button<Flat> {
        Button::new()
            .color(Color::Hsla(0.0, 0.0, 0.9, 1.0))
            .hover_color(Color::Hsla(0.0, 0.0, 0.87, 1.0))
            .press_color(Color::Hsla(0.0, 0.0, 0.83, 1.0))
            .border(1.5)
            .border_color(Color::Hsla(0.0, 0.0, 0.5, 1.0))
            .top_left_with_margins(425.0, left)
            .label(label)
            .label_font_size(15)
            .label_hsl(0.15, 0.15, 0.15)
            .label_y(Relative::Scalar(1.0))
            .w_h(120.0, 30.0)
    }

    pub async fn draw_2d(
        &mut self,
        context: &Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        rectangle::Rectangle {
            color: [1.0, 1.0, 1.0, 1.0],
            shape: rectangle::Shape::Square,
            border: None,
        }
        .draw(
            [20.0, 50.0, canvas_const::X_MAX, canvas_const::Y_MAX],
            &context.draw_state,
            context.transform,
            graphics,
        );

        self.vm
            .redraw(&context.trans(20.0, 50.0), graphics, character_cache)
            .await?;

        draw_border(context, graphics);
        Ok(())
    }

    pub async fn input(&mut self, input: Input) -> Result<()> {
        self.event_sender.input(input).await
    }
}

pub const CANVAS_TOP_LEFT: CanvasCoordinate = CanvasCoordinate { x: 20.0, y: 50.0 };

fn draw_border(context: &Context, graphics: &mut G2d) {
    let rectangle = rectangle::Rectangle {
        color: [1.0, 1.0, 1.0, 1.0],
        shape: rectangle::Shape::Square,
        border: None,
    };
    // Top
    rectangle.draw(
        [0.0, 0.0, WINDOW_SIZE.width, CANVAS_TOP_LEFT.y],
        &context.draw_state,
        context.transform,
        graphics,
    );
    // Bottom
    rectangle.draw(
        [
            0.0,
            CANVAS_TOP_LEFT.y + canvas_const::Y_MAX,
            WINDOW_SIZE.width,
            canvas_const::X_MAX,
        ],
        &context.draw_state,
        context.transform,
        graphics,
    );
    // Left
    rectangle.draw(
        [
            0.0,
            CANVAS_TOP_LEFT.y,
            CANVAS_TOP_LEFT.x,
            CANVAS_TOP_LEFT.y + canvas_const::Y_MAX,
        ],
        &context.draw_state,
        context.transform,
        graphics,
    );
    // Right
    rectangle.draw(
        [
            CANVAS_TOP_LEFT.x + canvas_const::X_MAX,
            CANVAS_TOP_LEFT.y,
            WINDOW_SIZE.width,
            CANVAS_TOP_LEFT.y + canvas_const::Y_MAX,
        ],
        &context.draw_state,
        context.transform,
        graphics,
    );
    // Inner border
    rectangle::Rectangle {
        color: [0.0, 0.0, 0.0, 0.0],
        shape: rectangle::Shape::Square,
        border: Some(rectangle::Border {
            color: [0.0, 0.0, 0.0, 1.0],
            radius: 0.5,
        }),
    }
    .draw(
        [
            CANVAS_TOP_LEFT.x,
            CANVAS_TOP_LEFT.y,
            canvas_const::X_MAX,
            canvas_const::Y_MAX,
        ],
        &context.draw_state,
        context.transform,
        graphics,
    );
}
