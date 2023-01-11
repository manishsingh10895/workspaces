use std::cell::RefCell;

use dirs::home_dir;
use rusqlite::{params, Connection, Error, Result};

use crate::workspace::{Dir, Workspace};

pub fn initialize_db() -> Result<()> {
    let conn = connect_db()?;
    let table_exists;
    let res = conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
        params!["workspaces"],
        |_| {
            return Ok(());
        },
    );

    match res {
        Ok(_) => {
            table_exists = true;
        }
        Err(e) => {
            table_exists = false;
            eprintln!("{}", e);
        }
    }

    if table_exists {
        return Ok(());
    }

    conn.execute(
        "
    CREATE TABLE workspaces (
        id      INTEGER PRIMARY KEY AUTOINCREMENT,
        name    TEXT UNIQUE NOT NULL
    )
    ",
        params![],
    )?;

    conn.execute(
        "
    CREATE TABLE dirs (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        workspaceId     INTEGER,
        path            TEXT NOT NULL,
        script          TEXT,
        FOREIGN KEY(workspaceId) REFERENCES workspaces(id)
        ON DELETE CASCADE
    );
    ",
        params![],
    )?;

    Ok(())
}

fn connect_db() -> Result<Connection> {
    let path = home_dir().unwrap();
    let path = path.to_str().unwrap();
    let path = format!("{}{}", path, "/workspaces.db");
    let conn = Connection::open(path)?;

    Ok(conn)
}

pub fn insert_new_workspace(workspace: Workspace) -> Result<usize> {
    let conn = connect_db()?;

    conn.execute(
        "
        INSERT INTO workspaces(name) VALUES (
            ?1
        )
    ",
        params![workspace.name],
    )?;

    let mut inserted_id = 0;
    conn.query_row("SELECT last_insert_rowid()", params![], |row| {
        let id: usize = row.get(0)?;
        inserted_id = id;
        return Ok(());
    })?;

    Ok(inserted_id)
}

/// Delete a workspace from database
pub fn delete_workspace(name: String) -> Result<()> {
    let conn = connect_db()?;

    let mut stmt = conn.prepare("DELETE from workspaces WHERE name = ?1")?;

    stmt.execute(params![name]).map(|_| ())
}

#[allow(dead_code)]
pub fn get_dirs_for_workspace(workspace_id: i32) -> Result<Vec<(i32, String)>> {
    let conn = connect_db()?;

    let mut stmt = conn.prepare("SELECT d.id, d.path from dirs d where workspaceId = ?1")?;

    let paths = stmt.query_map([workspace_id], |row| {
        let path: String = row.get(1).unwrap();
        let id: i32 = row.get(0).unwrap();
        return Ok((id, path));
    })?;

    let paths: Vec<(i32, String)> = paths.map(|x| x.unwrap()).collect();

    Ok(paths)
}

#[allow(dead_code)]
pub fn remove_dir_from_workspace(dir_id: i32) -> Result<()> {
    let conn = connect_db()?;

    let rows = conn.execute("DELETE from dirs where id = ?1", params![dir_id])?;

    if rows == 0 {
        return Err(rusqlite::Error::InvalidQuery);
    }

    Ok(())
}

pub fn insert_new_dir_for_workspace(workspace_id: i32, path: String) -> Result<usize, Error> {
    let conn = connect_db()?;

    conn.execute(
        "
        INSERT INTO dirs(workspaceId, path) VALUES(
            ?1, ?2
        );
    ",
        params![workspace_id, path],
    )?;

    Ok(get_last_insert_id(conn)?)
}

fn get_last_insert_id(conn: Connection) -> Result<usize> {
    let mut inserted_id = 0;
    conn.query_row("SELECT last_insert_rowid()", params![], |row| {
        let id: usize = row.get(0)?;
        inserted_id = id;
        return Ok(());
    })?;

    Ok(inserted_id)
}

#[allow(dead_code)]
pub fn fetch_all_workspaces() -> Result<Vec<(i32, String)>, Error> {
    let conn = connect_db().unwrap();

    let mut stmt = conn.prepare("SELECT w.name, w.id from workspaces w;")?;

    let values = stmt.query_map([], |x| {
        let name: String = x.get(0).unwrap();
        let id: i32 = x.get(1).unwrap();

        return Ok((id, name));
    })?;

    let values = values.map(|x| x.unwrap()).collect();

    return Ok(values);
}

pub fn fetch_workspace_with_dirs_by_name(name: &str) -> Option<Workspace> {
    let conn = connect_db().unwrap();

    let workspace_name = name;

    let mut stmt = conn
        .prepare(
            "SELECT w.name, w.id, d.path, d.id as did FROM workspaces w
        INNER JOIN dirs d on w.id = d.workspaceId
        WHERE w.name == ?1
        ",
        )
        .expect("Statement Failed");

    let id: RefCell<i32> = RefCell::new(0);
    let res = stmt.query_map([workspace_name], |row| {
        let w_name: String = row.get("name").expect("Can't get name");
        let _id: i32 = row.get("id").expect("Can't get id");
        let path: String = row.get("path").expect("Can't get path");
        let did: i32 = row.get("did").expect("did error");
        let script: String = row.get("script").unwrap_or(String::from("code ."));

        *id.borrow_mut() = _id;
        return Ok((w_name, _id, path.clone(), did, script));
    });

    match res {
        Ok(res) => {
            let mut ws = Workspace::new(String::from(workspace_name));
            let mut count = 0;
            for p in res {
                match p {
                    Ok(value) => {
                        let dir = Dir {
                            path: value.2,
                            id: value.3,
                            init: Some(String::from("asd")),
                        };
                        ws.add_dir(dir);
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            if count == 0 {
                return None;
            }

            ws.set_id(*id.borrow());

            Some(ws)
        }
        Err(e) => {
            eprintln!("{}", e);
            None
        }
    }
}

#[allow(dead_code)]
pub fn fetch_all_workspaces_with_dirs() -> Vec<Workspace> {
    let conn = connect_db().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT w.name, w.id, d.path from workspaces w
            LEFT JOIN dirs d
            ON d.workspaceId == w.id
        ",
        )
        .unwrap();

    let mut workspaces: Vec<Workspace> = vec![];

    let _ = stmt
        .query_map([], |x| {
            let val: String = x.get(0).unwrap();
            let id: i32 = x.get(1).unwrap();
            let path: String = x.get(2).unwrap_or(String::from("None"));

            if let Some(position) = workspaces.iter().position(|w| w.get_id() == id) {
                let ws = workspaces.get_mut(position).unwrap();
                ws.add_dir(Dir::new(path));
            } else {
                let space: Workspace;
                let mut ws = Workspace::new(val.clone());
                ws.set_id(id);
                ws.add_dir(Dir::new(path));
                space = ws;
                workspaces.push(space);
            }

            return Ok(val);
        })
        .unwrap();

    return workspaces;
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::db::*;

    #[ctor::ctor]
    fn init() {
        let _ = fs::remove_file("./test.db");
    }

    fn connect_test_db() -> Result<Connection> {
        let path = format!("./test.db");
        let conn = Connection::open(path)?;

        conn.execute(
            "
        CREATE TABLE if not exists workspaces (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            name    TEXT UNIQUE NOT NULL
        )
        ",
            params![],
        )?;

        conn.execute(
            "
        CREATE TABLE if not exists dirs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            workspaceId     INTEGER,
            path            TEXT NOT NULL,
            
            FOREIGN KEY(workspaceId) REFERENCES workspaces(id)
        );
        ",
            params![],
        )?;

        Ok(conn)
    }

    #[test]
    fn insert_a_workspace() -> Result<()> {
        let conn = connect_test_db()?;
        let w = Workspace::new(String::from("test1"));
        conn.execute(
            "
            INSERT INTO workspaces(name) VALUES (
                ?1
            )
        ",
            params![w.name],
        )?;
        let mut inserted_id = 0;
        conn.query_row("SELECT last_insert_rowid()", params![], |row| {
            let id: usize = row.get(0)?;
            println!("INSERT WORKSPACE ID {}", id);
            inserted_id = id;
            return Ok(());
        })?;
        assert_eq!(inserted_id, 1);
        Ok(())
    }

    #[test]
    fn should_find_workspace_by_name() {
        let _ = insert_a_workspace();
        let name = "OpenAlexa";
        let workspace = fetch_workspace_with_dirs_by_name(name);

        match workspace {
            Some(w) => {
                println!("Worksapce {:?}", w);
            }
            None => {
                eprintln!("Workspace Not found");
            }
        }
    }
}
