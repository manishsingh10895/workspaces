use crate::errors::Error;
use crate::utils;
use crate::workspace::Workspace;
use crate::{db, workspace};
use prettytable::Table;
use std::fs;
use std::path::PathBuf;

/// Open a workspace
/// opens all the directories in a code editor
pub fn open_workspace(name: String) -> Result<(), Error> {
    let ws = db::fetch_workspace_with_dirs_by_name(&name);

    if let Some(space) = ws {
        workspace::open_workspace(space);
        // space.dir_iter().for_each(|dir| {
        //     utils::open_code_editor(&dir.path);
        // });
    } else {
        eprintln!("Workspace not found");
        return Err(Error::DbError(String::from("Not Found")));
    }

    Ok(())
}

/// List all workspaces
/// and prints them to std out
pub fn print_workspaces() -> Result<(), Error> {
    let spaces = db::fetch_all_workspaces_with_dirs();

    println!("{:?}", spaces);

    let mut table = Table::new();

    table.add_row(row!["Workspace", "Directory"]);

    for space in spaces {
        let mut dir_table = table!();

        space.dir_iter().for_each(|dir| {
            dir_table.add_row(row![format!("{}", dir.path)]);
        });

        table.add_row(row![format!("{}", space.name), dir_table]);
    }

    table.printstd();

    Ok(())
}

/// Delete a workspace
pub fn delete_workspace(w_name: String) {
    db::delete_workspace(w_name).expect("Error deleting workspace");
}

/// Add a new workspace
pub fn add_workspace(w_name: Option<String>, path: Option<PathBuf>) -> Result<usize, Error> {
    let path = path.unwrap_or(PathBuf::from("."));

    let canonical = fs::canonicalize(path).unwrap();

    // get the current directory name
    let dir_name = canonical.to_str().unwrap().split("/").last().unwrap();

    let w_name = w_name.unwrap_or(dir_name.to_string());

    match db::insert_new_workspace(Workspace::new(w_name)) {
        Ok(id) => {
            match db::insert_new_dir_for_workspace(
                id as i32,
                canonical.to_str().unwrap().to_string(),
            ) {
                Ok(_) => {
                    return Ok(id);
                }
                Err(err) => {
                    eprintln!("Error {:?}", err);
                    return Err(Error::DbError(String::from(
                        "Cannot Insert dir into database",
                    )));
                }
            };
        }
        Err(err) => {
            eprintln!("Error {:?}", err);
            return Err(Error::DbError(String::from("Cannot Insert database")));
        }
    }
}

/// Add a directory to a workspace
pub fn add_dir_to_workspace(w_name: String, path: PathBuf) -> Result<(), Error> {
    if let Some(workspace) = db::fetch_workspace_with_dirs_by_name(&w_name) {
        println!("Workspace {:?}", workspace);
        let canonical = utils::get_canonical_path(path);
        if let Ok(_) = db::insert_new_dir_for_workspace(workspace.get_id(), canonical) {
            ()
        } else {
            return Err(Error::DbError(String::from("Error inserting directory")));
        }
    } else {
        eprintln!("Cannot find workspace with name {}", w_name);
        return Err(Error::DbError(String::from("Not found")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db;

    use super::print_workspaces;

    #[test]
    fn test_list_workspaces() {
        db::initialize_db().expect("Error Initializing databsae");

        print_workspaces().expect("Priting error");
    }

    #[test]
    fn test_open_workspace() {
        let name = "workspaces";

        let res = super::open_workspace(String::from(name));

        assert!(res.is_ok());
    }
}
