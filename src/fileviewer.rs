use super::*;
use blocks::BlockInputs;
use fileinput::FileInput;
use savefile::ScratchFile;
use vm::VM;
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub struct FileViewer {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    block_inputs: Vec<Vec<BlockInputs>>,
}

pub enum Msg {
    LoadFile(ScratchFile),
    SetBlockInputs(Vec<Vec<BlockInputs>>),
}

impl FileViewer {
    fn sprite(block_inputs: &[Vec<BlockInputs>]) -> Vec<VNode> {
        block_inputs
            .iter()
            .enumerate()
            .map(|(sprite_id, thread)| {
                html! {
                    <>
                        <h1><strong>{format!("Sprite {}", sprite_id)}</strong></h1>
                        {FileViewer::thread(thread)}
                    </>
                }
            })
            .collect()
    }

    fn thread(thread_blocks: &[BlockInputs]) -> Vec<VNode> {
        thread_blocks
            .iter()
            .enumerate()
            .map(|(thread_id, block_inputs)| {
                html! {
                    <>
                        <h2><strong>{format!("Thread {}", thread_id)}</strong></h2>
                        <Diagram block_inputs={block_inputs} />
                    </>
                }
            })
            .collect()
    }
}

impl Component for FileViewer {
    type Message = Msg;
    type Properties = ();

    fn create(_: (), link: ComponentLink<Self>) -> Self {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .set_title("Scratch File Viewer");

        Self {
            link,
            canvas_ref: NodeRef::default(),
            block_inputs: Vec::new(),
        }
    }

    fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::LoadFile(file) => {
                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let set_block_inputs = self.link.callback(Msg::SetBlockInputs);
                wasm_bindgen_futures::spawn_local(async move {
                    let vm = match VM::start(ctx, &file).await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    };

                    set_block_inputs.emit(vm.block_inputs());
                });

                false
            }
            Msg::SetBlockInputs(block_inputs) => {
                self.block_inputs = block_inputs;
                true
            }
        }
    }

    fn change(&mut self, _: ()) -> bool {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <>
                <FileInput onchange={self.link.callback(Msg::LoadFile)} />
                <canvas // Dummy canvas for Sprite
                    ref={self.canvas_ref.clone()}
                    width="0"
                    height="0"
                    hidden={true}
                />
                <style>
                    {"br { margin-bottom: 2px; }"}
                </style>
                <span style="font-family: monospace;">
                    {FileViewer::sprite(&self.block_inputs)}
                </span>
            </>
        }
    }
}

struct Diagram {
    block_inputs: RefCell<BlockInputs>,
}

#[derive(Clone, Properties, PartialEq)]
struct DiagramProps {
    block_inputs: BlockInputs,
}

impl Component for Diagram {
    type Message = ();
    type Properties = DiagramProps;

    fn create(props: DiagramProps, _: ComponentLink<Self>) -> Self {
        Self {
            block_inputs: RefCell::new(props.block_inputs),
        }
    }

    fn update(&mut self, _: ()) -> bool {
        false
    }

    fn change(&mut self, props: DiagramProps) -> bool {
        if *self.block_inputs.borrow() != props.block_inputs {
            self.block_inputs = RefCell::new(props.block_inputs);
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let mut block_inputs = self.block_inputs.borrow_mut();
        let next_html = if let Some(next) = block_inputs.stacks.remove("next") {
            html! {<><Diagram block_inputs={next} /></>}
        } else {
            html! {}
        };

        let mut substacks: Vec<(&'static str, BlockInputs)> = block_inputs.stacks.drain().collect();
        substacks.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        html! {
            <>
                <strong>{block_inputs.info.name.to_string()}</strong>
                {String::from(" ") + &block_inputs.info.id}<br />

                <div style="padding-left: 10px; border-left: 1px solid #B3B3B3; margin-bottom: 1px;">
                    {
                        for block_inputs.fields.iter().map(|field| {
                            html! {
                                <>
                                    {field.0.to_string() + ": " + &field.1}
                                    <br />
                                </>
                            }
                        })
                    }
                    {
                        for block_inputs.inputs.drain().map(|input| {
                            html! {
                                <>
                                    {input.0.to_string() + ": "}
                                    <Diagram block_inputs={input.1} />
                                </>
                            }
                        })
                    }
                    {
                        for substacks.drain(..).map(|substack| {
                            html! {
                                <>
                                    {substack.0}<br />
                                    <Diagram block_inputs={substack.1} />
                                </>
                            }
                        })
                    }
                </div>
                {next_html}
            </>
        }
    }
}
