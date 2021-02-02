use super::*;
use crate::broadcaster::Broadcaster;
use crate::file::ScratchFile;
use crate::vm::{DebugInfo, VM};
use conrod_core::image::Id;
use conrod_core::widget::{id, Button};
use conrod_core::{Color, UiCell};
use conrod_core::{Positionable, Sizeable, Widget};
use graphics::math::Matrix2d;
use graphics::Transformed;
use graphics::{Context, DrawState};
use piston_window::{G2d, G2dTextureContext, Glyphs};
use tokio::sync::mpsc;

pub struct Interface {
    scratch_file: ScratchFile,
    ids: Ids,
    green_flag_image: Id,
    stop_image: Id,
    vm: VM,
    debug_receiver: mpsc::Receiver<DebugInfo>,
}

widget_ids! {
    pub struct Ids {
        green_flag_button,
        stop_button,
    }
}

impl Interface {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        scratch_file: ScratchFile,
        ids: Ids,
        green_flag_image: Id,
        stop_image: Id,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let broadcaster = Broadcaster::new();
        let vm = VM::new(texture_context, scratch_file.clone(), sender, broadcaster).await?;
        Ok(Self {
            scratch_file,
            ids,
            green_flag_image,
            stop_image,
            vm,
            debug_receiver: receiver,
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
        }

        Button::image(self.stop_image)
            .top_left_with_margins(10.0, 70.0)
            .w_h(30.0, 30.0)
            .image_color_with_feedback(Color::Rgba(1.0, 1.0, 1.0, 1.0))
            .set(self.ids.stop_button, ui_cell);
    }

    pub async fn draw_2d(
        &mut self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        context.transform = context.transform.trans_pos([20.0, 50.0]);
        self.vm.redraw(context, graphics, character_cache).await
    }
}
