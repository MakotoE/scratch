use super::*;
use savefile::ScratchFile;
use yew::prelude::*;
use yew::services::reader::{FileData, ReaderService, ReaderTask};

pub struct FileInput {
    link: ComponentLink<Self>,
    props: Props,
    task: Option<ReaderTask>,
}

pub enum Msg {
    Noop,
    ImportFile(web_sys::File),
    ParseFile(FileData),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub onchange: Callback<ScratchFile>,
}

impl FileInput {
    fn import_cb(event: ChangeData) -> Msg {
        if let ChangeData::Files(files) = event {
            if let Some(file) = files.get(0) {
                return Msg::ImportFile(file);
            }
        }
        Msg::Noop
    }
}

impl Component for FileInput {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            task: None,
        }
    }

    fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Noop => {}
            Msg::ImportFile(file) => {
                let mut reader = ReaderService::new();
                let cb = self.link.callback(Msg::ParseFile);
                self.task = Some(reader.read_file(file, cb).unwrap());
            }
            Msg::ParseFile(file) => {
                let reader = std::io::Cursor::new(file.content);
                self.props
                    .onchange
                    .emit(ScratchFile::parse(reader).unwrap());
            }
        }
        false
    }

    fn change(&mut self, props: Props) -> bool {
        self.props = props;
        false
    }

    fn view(&self) -> Html {
        html! {
            <input type="file" accept=".sb3" onchange={self.link.callback(FileInput::import_cb)} />
        }
    }
}
