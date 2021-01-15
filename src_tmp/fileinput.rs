use crate::file::ScratchFile;
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
                let cb = self.link.callback(Msg::ParseFile);
                match ReaderService::new().read_file(file, cb) {
                    Ok(task) => self.task = Some(task),
                    Err(e) => log::error!("error occurred while reading file: {}", e),
                };
            }
            Msg::ParseFile(file) => {
                match ScratchFile::parse(std::io::Cursor::new(file.content)) {
                    Ok(f) => self.props.onchange.emit(f),
                    Err(e) => log::error!("error occurred while parsing Scratch file: {}", e),
                };
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
