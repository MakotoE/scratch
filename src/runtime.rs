use super::*;


use sprite_runtime::SpriteRuntime;




#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Rc<RwLock<SpriteRuntime>>,
    pub global: Global,
}

#[derive(Debug, Clone)]
pub struct Global {
    pub variables: Rc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl Global {
    pub fn new(scratch_file_variables: &HashMap<String, savefile::Variable>) -> Self {
        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, v) in scratch_file_variables {
            variables.insert(key.clone(), v.value.clone());
        }

        Self {
            variables: Rc::new(RwLock::new(variables)),
        }
    }
}
