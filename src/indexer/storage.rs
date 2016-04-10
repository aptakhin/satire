
use indexer::parser::Tagged;

#[derive(Clone, Copy)]
pub struct FileSource {
    //pub file: String,
    pub line: i32,
    pub id_iter: usize,
    pub lexem_iter: usize,
}

impl FileSource {
    pub fn render_html(&self, name: &str) -> String {
        format!("<a href=\"#l{}\">{}</a>", self.line, name)
    }
}

pub struct Unit {
    pub unit_type: String,
    pub path: Vec<String>,
    pub source: FileSource,
}

// pub trait Tagged {
//     //fn get_type(self) -> String;
//     //fn get_source(self) -> FileSource;
//     fn get_content(&self) -> String;
//     fn render_html(&self) -> String;
// }

pub struct Context {
    pub all_tagged: Vec<Box<Tagged>>,
    //pub use_units: Vec<Unit>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            all_tagged: vec![],
            //use_units: vec![],
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        self.all_tagged.append(&mut ctx.all_tagged);
        //self.use_units.append(&mut ctx.use_units);
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
