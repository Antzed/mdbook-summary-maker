use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::fs;

use mdbook::MDBook;


const SUMMARY_FILE: &str = "SUMMARY.md";


pub struct SummaryMaker;

impl SummaryMaker {
    pub fn new() -> SummaryMaker {
        SummaryMaker
    }

    fn generate_summary(&self, ctx: &PreprocessorContext) -> Result<(), Error> {

        // check if there is a SUMMARY.md file in the src folder
        // if there is, delete it
        let mut src_path = PathBuf::new();
        src_path.push(&ctx.root);
        src_path.push(&ctx.config.book.src);
        
        let mut summary_path = src_path.clone();
        summary_path.push("SUMMARY.md");


        // if there is a SUMMARY.md file, delete it
        if summary_path.exists() {
            std::fs::remove_file(summary_path.clone()).unwrap();
        }

        // create a new SUMMARY.md file
        let summary_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(src_path.clone().join(SUMMARY_FILE))
        .unwrap();
        // write the SUMMARY.md file

        let mut buff = String::new();
        buff.push_str("# Summary\n\n");

        // write buff to file


        let mut file = File::create(summary_path)?;
        file.write_all(buff.as_bytes())?;
        // Walk the directory tree and generate the summary recursively
        walk_directory_tree(&src_path, &src_path, &mut file, 0)?;

        match MDBook::load(&ctx.root) {
            Ok(mdbook) => {
                return Ok(());
            }
            Err(e) => {
                panic!("{}", e);
            }
        };

    }

    
    
}

impl Preprocessor for SummaryMaker {
    fn name(&self) -> &str {
        "summary-maker"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name()) {
            if nop_cfg.contains_key("blow-up") {
                anyhow::bail!("Boom!!1!");
            }
        }
        
        self.generate_summary(ctx)?;
        

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

fn walk_directory_tree(path: &PathBuf, src_path: &PathBuf, file: &mut File, depth: usize) -> Result<(), Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        let file_name = entry_name.to_string_lossy();
        let metadata = fs::metadata(&entry_path)?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        let indent = "    ".repeat(depth);

        if metadata.is_dir() {
            let readme_path = entry_path.join("README.md");
            if readme_path.exists() {
                let link_path = readme_path.strip_prefix(src_path)?;
                let link_path_str = link_path.to_str().ok_or_else(|| {
                    Error::msg(format!("Invalid path: {}", link_path.display()))
                })?;
                write!(file, "{}- [{}](./{})\n", indent, file_name, link_path_str)?;
            } else {
                write!(file, "{}- {}\n", indent, file_name)?;
            }

            walk_directory_tree(&entry_path, src_path, file, depth + 1)?;
        } else if file_name.ends_with(".md")  { //&& file_name != "README.md"
            let link_path = entry_path.strip_prefix(src_path)?;
            let link_path_str = link_path.to_str().ok_or_else(|| {
                Error::msg(format!("Invalid path: {}", link_path.display()))
            })?;

            write!(
                file,
                "{}- [{}](./{})\n",
                indent,
                file_name.trim_end_matches(".md"),
                link_path_str
            )?;
        }
    }

    Ok(())
}
