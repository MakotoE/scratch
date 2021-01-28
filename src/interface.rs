use super::*;
use crate::broadcaster::Broadcaster;
use crate::file::ScratchFile;
use crate::vm::VM;
use conrod_core::image::Id;
use conrod_core::widget::{id, Button};
use conrod_core::UiCell;
use conrod_core::{Positionable, Sizeable, Widget};
use graphics::math::Matrix2d;
use graphics::DrawState;
use piston_window::{G2d, G2dTextureContext};
use tokio::sync::mpsc;

pub struct Interface {
    scratch_file: ScratchFile,
    ids: Ids,
    green_flag_image: Id,
    stop_image: Id,
    vm: VM,
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
        })
    }

    pub fn widgets(&mut self, ui_cell: &mut UiCell) {
        Button::image(self.green_flag_image)
            .top_left_with_margins(10.0, 25.0)
            .w_h(30.0, 30.0)
            .set(self.ids.green_flag_button, ui_cell);

        Button::image(self.stop_image)
            .top_left_with_margins(10.0, 70.0)
            .w_h(30.0, 30.0)
            .set(self.ids.stop_button, ui_cell);
    }

    pub fn draw_2d(draw_state: &DrawState, transform: Matrix2d, graphics: &mut G2d) {}
}
