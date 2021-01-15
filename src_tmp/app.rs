use yew::prelude::*;
use yew_router::prelude::*;

use interface::ScratchInterface;

use crate::fileviewer::FileViewer;

use super::*;

#[derive(Switch, Debug, Clone)]
pub enum Route {
    #[to = "/fileviewer"]
    FileViewer,
    #[to = "/"]
    Index,
}

pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: ()) -> bool {
        false
    }

    fn change(&mut self, _: ()) -> bool {
        false
    }

    fn view(&self) -> Html {
        let render = Router::render(|switch: Route| match switch {
            Route::Index => html! {<ScratchInterface />},
            Route::FileViewer => html! {<FileViewer />},
        });
        let redirect = Router::redirect(|route: yew_router::route::Route| {
            log::warn!("page not found: {}", route);
            Route::Index
        });
        html! {
            <Router<Route>
                render = render,
                redirect = redirect,
            />
        }
    }
}
