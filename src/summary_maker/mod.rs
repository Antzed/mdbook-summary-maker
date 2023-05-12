use mdbook::errors::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::fs;
use mdbook::config::Config;

const SUMMARY_FILE: &str = "SUMMARY.md";

pub fn generate_summary(root: &PathBuf, config: &Config) -> Result<(), Error> {
    let mut src_path = PathBuf::new();
    src_path.push(root);
    src_path.push(&config.book.src);
    
    let mut summary_path = src_path.clone();
    summary_path.push("SUMMARY.md");

    if summary_path.exists() {
        std::fs::remove_file(summary_path.clone()).unwrap();
    }

    let summary_file = std::fs::OpenOptions::new()
    .write(true)
    .read(true)
    .create(true)
    .truncate(true)
    .open(src_path.clone().join(SUMMARY_FILE))
    .unwrap();

    let mut buff = String::new();
    buff.push_str("# Summary\n\n");
    buff.push_str("# Content\n\n");
    let mut file = File::create(summary_path)?;
    file.write_all(buff.as_bytes())?;

    walk_directory_tree(&src_path, &src_path, &mut file, 0)?;

    Ok(())
}

fn walk_directory_tree(path: &PathBuf, src_path: &PathBuf, file: &mut File, depth: usize) -> Result<(), Error> {
    let mut top_level_readme = None;
    let mut top_level_files = Vec::new();

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

        // if the entry is a directory and it does not contain "(draft)" in its name
        if metadata.is_dir() && file_name.contains("(draft)") == false {
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
        } else if file_name == "README.md" && entry_path.parent().unwrap() == src_path {
            top_level_readme = Some(entry_path);
        } else if file_name.ends_with(".md") && file_name != "README.md" && file_name != SUMMARY_FILE && file_name.contains("(draft)") == false{
            if entry_path.parent().unwrap() == src_path {
                top_level_files.push(entry_path);
            } else {
                let link_path = entry_path.strip_prefix(src_path)?;
                let link_path_str = link_path.to_str().ok_or_else(|| {
                    Error::msg(format!("Invalid path: {}", link_path.display()))
                })?;

                //impelement: input_name is the first line of the file if it starts with #, otherwise the file name
                // get the first line of the file
                let contents = std::fs::read_to_string(&entry_path)?;

                // puncuation vector
                let puncuation = vec!['.', '!', '?', '\'', '"', '[', ']', '(', ')', '{', '}', ':', ';', '-', '_'];

                let input_name = if let Some(first_line) = contents.lines().next() {
                    // if the first line starts with # with a space, then use it as the input name 
                    if first_line.starts_with("# ") {
                        first_line.trim_start_matches('#').trim()
                        // get rid of punctuation
                        .trim_end_matches(|c| puncuation.contains(&c))

                    } else {
                        file_name.trim_end_matches(".md")
                    }
                } else {
                    file_name.trim_end_matches(".md")
                };
                
            


                write!(
                    file,
                    "{}- [{}](./{})\n",
                    indent,
                    input_name,
                    link_path_str
                )?;
            }
        }
    }

    if let Some(readme_path) = top_level_readme {
        let link_path = readme_path.strip_prefix(src_path)?;
        let link_path_str = link_path.to_str().ok_or_else(|| {
            Error::msg(format!("Invalid path: {}", link_path.display()))
        })?;


        // write the welcome page on beginning of the file

        

        write!(file, "---\n")?;
        write!(file, "# Welcome\n\n")?;
        write!(file, "- [About](./{})\n", link_path_str)?;
    }

    for entry_path in top_level_files {
        let entry_name = entry_path.file_name().unwrap();
        let file_name = entry_name.to_string_lossy();
        let link_path = entry_path.strip_prefix(src_path)?;
        let link_path_str = link_path.to_str().ok_or_else(|| {
            Error::msg(format!("Invalid path: {}", link_path.display()))
        })?;

        write!(
            file,
            "- [{}](./{})\n",
            file_name.trim_end_matches(".md"),
            link_path_str
        )?;
    }

    Ok(())
}

    