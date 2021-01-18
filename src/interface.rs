use crate::file::ScratchFile;
use conrod_core::image::Id;
use conrod_core::widget::{id, Button};
use conrod_core::UiCell;
use conrod_core::{Positionable, Sizeable, Widget};

pub struct Interface {
    scratch_file: ScratchFile,
    ids: Ids,
    green_flag_image: Id,
    stop_image: Id,
}

widget_ids! {
    struct Ids {
        green_flag_button,
        stop_button,
    }
}

impl Interface {
    pub fn new(
        scratch_file: ScratchFile,
        generator: id::Generator,
        green_flag_image: Id,
        stop_image: Id,
    ) -> Self {
        Self {
            scratch_file,
            ids: Ids::new(generator),
            green_flag_image,
            stop_image,
        }
    }

    pub fn widgets(&self, ui_cell: &mut UiCell) {
        Button::image(self.green_flag_image)
            .top_left_with_margins(10.0, 25.0)
            .w_h(30.0, 30.0)
            .set(self.ids.green_flag_button, ui_cell);

        Button::image(self.stop_image)
            .top_left_with_margins(10.0, 70.0)
            .w_h(30.0, 30.0)
            .set(self.ids.stop_button, ui_cell);
    }
}
