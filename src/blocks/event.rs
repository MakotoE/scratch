use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
    })
}

#[derive(Debug)]
pub struct WhenFlagClicked {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl WhenFlagClicked {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for WhenFlagClicked {
    fn block_name(&self) -> &'static str {
        "WhenFlagClicked"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        Next::continue_(self.next.clone())
    }
}
