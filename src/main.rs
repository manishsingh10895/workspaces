#[macro_use]
extern crate prettytable;

use std::path::PathBuf;
use structopt::StructOpt;

mod command_handlers;
mod db;
mod errors;
mod utils;
mod workspace;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    command: Operation,
}

#[derive(StructOpt, Debug)]
struct WorkspaceOperation {
    #[allow(unused)]
    #[structopt(subcommand)]
    operation: Operation,
}

#[derive(StructOpt, Debug)]
enum Operation {
    #[structopt(about = "Open a workspace")]
    Open {
        #[structopt(short = "w", long = "workspace")]
        workspace: String,
    },
    #[structopt(about = "Add new workspace")]
    Add {
        #[structopt(short = "p", parse(from_os_str))]
        path: Option<PathBuf>,

        #[structopt(short = "n", long = "name")]
        name: Option<String>,
    },
    #[structopt(about = "deletes a workspace")]
    Del {
        #[structopt(short = "n", long = "name")]
        name: String,
    },
    #[structopt(about = "list all workspaces")]
    List,
    #[structopt(about = "Dir operations")]
    Dir {
        #[structopt(short = "w", long = "workspace")]
        workspace: String,

        #[structopt(subcommand)]
        dir_operation: DirOperation,
    },
}

#[derive(StructOpt, Debug)]
enum DirOperation {
    #[structopt(about = "about add a directory to a workspace")]
    Add {
        #[structopt(short = "p", parse(from_os_str))]
        path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    db::initialize_db()?;

    let options: Options = Options::from_args();

    match options.command {
        Operation::Add { name, path } => {
            command_handlers::add_workspace(name, path)?;
            println!("Workspace added")
        }
        Operation::Del { name } => {
            println!("Deleting workspace");
            command_handlers::delete_workspace(name);
            println!("Workspace deleted");
        }
        Operation::List => {
            command_handlers::print_workspaces()?;
        }
        Operation::Open { workspace } => {
            command_handlers::open_workspace(workspace).expect("Error opening workspace")
        }
        Operation::Dir {
            workspace,
            dir_operation,
        } => match dir_operation {
            DirOperation::Add { path } => {
                command_handlers::add_dir_to_workspace(workspace, path)?;
            }
        },
    }

    Ok(())
}
