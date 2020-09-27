use super::*;
use fileinput::FileInput;
use savefile::ScratchFile;
use yew::prelude::*;

pub struct FileViewer {
    link: ComponentLink<Self>,
}

pub enum Msg {
    Noop,
}

impl Component for FileViewer {
    type Message = Msg;
    type Properties = ();

    fn create(_: (), link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _: Msg) -> bool {
        true
    }

    fn change(&mut self, _: ()) -> bool {
        unreachable!()
    }

    fn view(&self) -> Html {
        let fileinput_cb: fn(ScratchFile) -> Msg = |file| {
            log::info!("{:?}", file);
            Msg::Noop
        };

        html! {
            <p><FileInput onchange={self.link.callback(fileinput_cb)} /></p>
        }
    }
}
