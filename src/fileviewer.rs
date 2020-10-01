use super::*;
use blocks::Inputs;
use fileinput::FileInput;
use savefile::ScratchFile;
use yew::prelude::*;

pub struct FileViewer {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    inputs: Vec<Inputs>,
}

pub enum Msg {
    LoadFile(ScratchFile),
}

impl Component for FileViewer {
    type Message = Msg;
    type Properties = ();

    fn create(_: (), link: ComponentLink<Self>) -> Self {
        Self {
            link,
            canvas_ref: NodeRef::default(),
            inputs: Vec::new(),
        }
    }

    fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::LoadFile(file) => {
                let target = &file.project.targets[1];
                let hats = sprite::find_hats(&target.blocks);

                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();
                let runtime = runtime::SpriteRuntime::new(ctx, HashMap::new());
                let runtime_ref: Rc<RwLock<runtime::SpriteRuntime>> = Rc::new(RwLock::new(runtime));

                self.inputs.clear();
                for hat in hats {
                    let block =
                        blocks::new_block(hat, runtime_ref.clone(), &target.blocks).unwrap();
                    self.inputs.push(block.inputs());
                }
            }
        }
        true
    }

    fn change(&mut self, _: ()) -> bool {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <>
                <FileInput onchange={self.link.callback(Msg::LoadFile)} />
                <canvas // Dummy canvas
                    ref={self.canvas_ref.clone()}
                    width="0"
                    height="0"
                />
                <span style="font-family: monospace;">
                    {
                        for self.inputs.iter().enumerate().map(|(i, inputs)| {
                            html! {
                                <>
                                    <p><strong>{String::from("Thread ") + &i.to_string()}</strong></p>
                                    <Diagram inputs={inputs} />
                                </>
                            }
                        })
                    }
                </span>
            </>
        }
    }
}

#[derive(Clone, Properties, PartialEq)]
struct Diagram {
    inputs: Inputs,
}

impl Component for Diagram {
    type Message = ();
    type Properties = Self;

    fn create(props: Self, _: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _: ()) -> bool {
        false
    }

    fn change(&mut self, props: Self) -> bool {
        if *self != props {
            *self = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <p>
                    <strong>{self.inputs.info.name.to_string()}</strong>
                    {String::from(" ") + &self.inputs.info.id}
                </p>
                <div style="margin-left: 20px;">
                    {
                        for self.inputs.fields.iter().map(|field| {
                            html! {
                                <>
                                    <p>{field.0}</p>
                                    <p>{field.1.clone()}</p>
                                </>
                            }
                        })
                    }
                    {
                        for self.inputs.inputs.iter().map(|input_row| {
                            html! {
                                <>
                                    <p>{input_row.0}</p>
                                    <Diagram inputs={input_row.1.clone()} />
                                </>
                            }
                        })
                    }
                    {
                        for self.inputs.stacks.iter().map(|stack_row| {
                            html! {
                                <>
                                    <p>{stack_row.0}</p>
                                    <Diagram inputs={stack_row.1.clone()} />
                                </>
                            }
                        })
                    }
                </div>
            </div>
        }
    }
}
