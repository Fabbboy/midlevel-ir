use crate::function::{FuncId, Function};

#[derive(Debug)]
pub struct Module<'ctx> {
    name: &'ctx str,
    funcs: Vec<Function<'ctx>>,
}

impl<'ctx> Module<'ctx> {
    pub fn new(name: &'ctx str) -> Self {
        Module {
            name,
            funcs: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &'ctx str {
        self.name
    }

    pub fn add_function(&mut self, function: Function<'ctx>) -> FuncId {
        let func_id = FuncId(self.funcs.len());
        self.funcs.push(function);
        func_id
    }

    pub fn get_functions(&self) -> &[Function<'ctx>] {
        &self.funcs
    }

    pub fn get_function(&self, id: FuncId) -> Option<&Function<'ctx>> {
        self.funcs.get(id.0)
    }

    pub fn get_function_mut(&mut self, id: FuncId) -> Option<&mut Function<'ctx>> {
        self.funcs.get_mut(id.0)
    }
}
