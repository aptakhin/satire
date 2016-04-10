
pub struct FileSource {
    pub file: String,
    pub line: i32,
    pub id_iter: usize,
    pub lexem_iter: usize,
}

pub struct Unit {
    pub unit_type: String,
    pub path: Vec<String>,
    pub source: FileSource,
}

pub trait Marked {
    fn get_type(self) -> String;
    fn get_source(self) -> FileSource;
}

pub struct Context {
    pub units: Vec<Unit>,
    pub use_units: Vec<Unit>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            units: vec![],
            use_units: vec![],
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        self.units.append(&mut ctx.units);
        self.use_units.append(&mut ctx.use_units);
    }

    pub fn gen(&mut self) {

    }
}

pub struct Storage {
    pub ctx: Context,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            ctx: Context::new(),
        }
    }

    pub fn merge(&mut self, mut merge_ctx: Context) {
        self.ctx.merge(merge_ctx);
    }

    pub fn gen(&mut self) {
        self.ctx.gen();
    }
}
