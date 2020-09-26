use super::*;
use yew::prelude::*;

pub struct FileViewer {}

impl Component for FileViewer {
    type Message = ();
    type Properties = ();

    fn create(_: (), _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: ()) -> bool {
        true
    }

    fn change(&mut self, _: ()) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <p>{"FileViewer"}</p>
        }
    }
}
