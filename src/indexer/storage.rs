
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

pub struct Context {
    pub all_tagged: Vec<Box<Tagged>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            all_tagged: vec![],
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        self.all_tagged.append(&mut ctx.all_tagged);
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
