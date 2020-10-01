use super::*;
use blocks::BlockInputs;
use fileinput::FileInput;
use savefile::ScratchFile;
use yew::prelude::*;

pub struct FileViewer {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    block_inputs: Vec<BlockInputs>,
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
            block_inputs: Vec::new(),
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

                self.block_inputs.clear();
                for hat in hats {
                    let block =
                        blocks::new_block(hat, runtime_ref.clone(), &target.blocks).unwrap();
                    self.block_inputs.push(block.block_inputs());
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
                        for self.block_inputs.iter().enumerate().map(|(i, block)| {
                            html! {
                                <>
                                    <p><strong>{String::from("Thread ") + &i.to_string()}</strong></p>
                                    <Diagram block_inputs={block} />
                                </>
                            }
                        })
                    }
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

impl Diagram {
    fn stacks_html(block_inputs: &mut HashMap<&'static str, BlockInputs>) -> Html {
        let next_option = block_inputs.remove("next");

        let mut blocks_vec: Vec<(&'static str, BlockInputs)> = block_inputs.drain().collect();
        blocks_vec.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        let mut list = yew::virtual_dom::VList::new();
        for block in blocks_vec {
            list.add_child(html! {
                <>
                    <p>{block.0}</p>
                    <Diagram block_inputs={block.1} />
                </>
            });
        }

        if let Some(next) = next_option {
            list.add_child(html! {
                <>
                    <p>{"next"}</p>
                    <Diagram block_inputs={next} />
                </>
            });
        }

        yew::virtual_dom::VNode::VList(list)
    }
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
        html! {
            <div>
                <p>
                    <strong>{block_inputs.info.name.to_string()}</strong>
                    {String::from(" ") + &block_inputs.info.id}
                </p>
                <div style="margin-left: 20px;">
                    {
                        for block_inputs.fields.iter().map(|field| {
                            html! {
                                <p>{field.0.to_string() + " " + &field.1}</p>
                            }
                        })
                    }
                    {
                        for block_inputs.inputs.drain().map(|input_row| {
                            html! {
                                <>
                                    <p>{input_row.0}</p>
                                    <Diagram block_inputs={input_row.1} />
                                </>
                            }
                        })
                    }
                </div>
                {
                    Diagram::stacks_html(&mut block_inputs.stacks)
                }
            </div>
        }
    }
}
