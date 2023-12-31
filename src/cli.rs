use std::{
    error::Error,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug, Default)]
pub struct Arg {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Debug, Default)]
pub enum Command {
    #[default]
    Lsp,
    Manually(Options),
}

#[derive(Args, Clone, Debug)]
pub struct Options {
    pub input: String,
    pub output: Option<String>,
    #[arg(long, short)]
    pub exclude_node_modules: bool,
}

impl Options {
    pub fn get_all_input_files(&self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let input_path = {
            let path = Path::new(&self.input);
            path.exists().then_some(path).ok_or_else(|| {
                io::Error::new(
                    ErrorKind::NotFound,
                    "The input you specified is not available.",
                )
            })
        }?;

        if input_path.is_dir() {
            return Ok(read_dir_recursive(input_path, self.exclude_node_modules));
        }
        return check_availability(input_path.to_path_buf());

        fn read_dir_recursive(path: &Path, exclude_node_modules: bool) -> Vec<PathBuf> {
            path.read_dir()
                .into_par_iter()
                .map(|read_dir| {
                    read_dir
                        .flatten()
                        .flat_map(|entity| {
                            let path_buf = entity.path();
                            if path_buf.is_dir() {
                                if exclude_node_modules {
                                    if path_buf.to_string_lossy().contains("node_modules") {
                                        vec![]
                                    } else {
                                        read_dir_recursive(&path_buf, exclude_node_modules)
                                    }
                                } else {
                                    read_dir_recursive(&path_buf, exclude_node_modules)
                                }
                            } else {
                                check_availability(path_buf).unwrap_or_default()
                            }
                        })
                        .collect::<Vec<PathBuf>>()
                })
                .flatten()
                .collect()
        }

        fn check_availability(path: PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
            if let Some(_file_extension @ ("ts" | "js" | "jsx" | "tsx")) =
                path.extension().and_then(|os_str| os_str.to_str())
            {
                Ok(vec![path])
            } else {
                Err(Box::from(io::Error::new(
                    ErrorKind::Unsupported,
                    "The file you specified is unsupported.",
                )))
            }
        }
    }
}
