use crate::{
    error::Result,
    models::{ActionEntry, Memory, Message, Project, Task},
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Storage(Arc<Mutex<Connection>>);

impl Storage {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)?;
        let db = Self(Arc::new(Mutex::new(connection)));
        db.migrate()?;
        Ok(db)
    }
    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let db = Self(Arc::new(Mutex::new(Connection::open_in_memory()?)));
        db.migrate()?;
        Ok(db)
    }
    fn migrate(&self) -> Result<()> {
        self.0.lock().unwrap().execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
      CREATE TABLE IF NOT EXISTS projects(id TEXT PRIMARY KEY,name TEXT NOT NULL,root TEXT NOT NULL UNIQUE,cloud_code_allowed INTEGER NOT NULL DEFAULT 0,authorized_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS actions(id INTEGER PRIMARY KEY AUTOINCREMENT,timestamp TEXT NOT NULL,actor TEXT NOT NULL,action_type TEXT NOT NULL,project TEXT,target TEXT,result TEXT NOT NULL,success INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS messages(id TEXT PRIMARY KEY,role TEXT NOT NULL,content TEXT NOT NULL,created_at TEXT NOT NULL,project_id TEXT);
      CREATE TABLE IF NOT EXISTS tasks(id TEXT PRIMARY KEY,title TEXT NOT NULL,project_id TEXT,description TEXT NOT NULL,status TEXT NOT NULL,priority TEXT NOT NULL,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS memories(id TEXT PRIMARY KEY,project_id TEXT,category TEXT NOT NULL,content TEXT NOT NULL,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);")?;
        Ok(())
    }
    pub fn projects(&self) -> Result<Vec<Project>> {
        let db = self.0.lock().unwrap();
        let mut q=db.prepare("SELECT id,name,root,cloud_code_allowed,authorized_at FROM projects ORDER BY authorized_at")?;
        let rows = q
            .query_map([], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    root: r.get(2)?,
                    cloud_code_allowed: r.get(3)?,
                    authorized_at: r.get(4)?,
                    branch: None,
                })
            })?
            .filter_map(|x| x.ok())
            .collect();
        Ok(rows)
    }
    pub fn upsert_project(&self, p: &Project) -> Result<()> {
        self.0.lock().unwrap().execute("INSERT INTO projects(id,name,root,cloud_code_allowed,authorized_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(root) DO UPDATE SET cloud_code_allowed=excluded.cloud_code_allowed",params![p.id,p.name,p.root,p.cloud_code_allowed,p.authorized_at])?;
        Ok(())
    }
    pub fn revoke_project(&self, id: &str) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .execute("DELETE FROM projects WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn audit(
        &self,
        actor: &str,
        action: &str,
        project: Option<&str>,
        target: Option<&str>,
        result: &str,
        success: bool,
    ) -> Result<()> {
        self.0.lock().unwrap().execute("INSERT INTO actions(timestamp,actor,action_type,project,target,result,success) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![Utc::now().to_rfc3339(),actor,action,project,target,result,success])?;
        Ok(())
    }
    pub fn actions(&self) -> Result<Vec<ActionEntry>> {
        let db = self.0.lock().unwrap();
        let mut q=db.prepare("SELECT id,timestamp,actor,action_type,project,target,result,success FROM actions ORDER BY id DESC LIMIT 500")?;
        let rows = q
            .query_map([], |r| {
                Ok(ActionEntry {
                    id: r.get(0)?,
                    timestamp: r.get(1)?,
                    actor: r.get(2)?,
                    action_type: r.get(3)?,
                    project: r.get(4)?,
                    target: r.get(5)?,
                    result: r.get(6)?,
                    success: r.get(7)?,
                })
            })?
            .filter_map(|x| x.ok())
            .collect();
        Ok(rows)
    }
    pub fn add_message(&self, m: &Message, project: Option<&str>) -> Result<()> {
        self.0.lock().unwrap().execute(
            "INSERT INTO messages(id,role,content,created_at,project_id) VALUES(?1,?2,?3,?4,?5)",
            params![m.id, m.role, m.content, m.created_at, project],
        )?;
        Ok(())
    }
    pub fn messages(&self) -> Result<Vec<Message>> {
        let db = self.0.lock().unwrap();
        let mut q = db.prepare(
            "SELECT id,role,content,created_at FROM messages ORDER BY created_at LIMIT 500",
        )?;
        let rows = q
            .query_map([], |r| {
                Ok(Message {
                    id: r.get(0)?,
                    role: r.get(1)?,
                    content: r.get(2)?,
                    created_at: r.get(3)?,
                })
            })?
            .filter_map(|x| x.ok())
            .collect();
        Ok(rows)
    }
    pub fn tasks(&self) -> Result<Vec<Task>> {
        let db = self.0.lock().unwrap();
        let mut q=db.prepare("SELECT id,title,project_id,description,status,priority,updated_at FROM tasks ORDER BY updated_at DESC")?;
        let rows = q
            .query_map([], |r| {
                Ok(Task {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    project_id: r.get(2)?,
                    description: r.get(3)?,
                    status: r.get(4)?,
                    priority: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?
            .filter_map(|x| x.ok())
            .collect();
        Ok(rows)
    }
    pub fn memories(&self) -> Result<Vec<Memory>> {
        let db = self.0.lock().unwrap();
        let mut q=db.prepare("SELECT id,project_id,category,content,updated_at FROM memories ORDER BY updated_at DESC")?;
        let rows = q
            .query_map([], |r| {
                Ok(Memory {
                    id: r.get(0)?,
                    project_id: r.get(1)?,
                    category: r.get(2)?,
                    content: r.get(3)?,
                    updated_at: r.get(4)?,
                })
            })?
            .filter_map(|x| x.ok())
            .collect();
        Ok(rows)
    }
    pub fn save_task(&self, t: &Task) -> Result<()> {
        self.0.lock().unwrap().execute("INSERT INTO tasks(id,title,project_id,description,status,priority,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET title=excluded.title,description=excluded.description,status=excluded.status,priority=excluded.priority,updated_at=excluded.updated_at",params![t.id,t.title,t.project_id,t.description,t.status,t.priority,t.updated_at])?;
        Ok(())
    }
    pub fn save_memory(&self, m: &Memory) -> Result<()> {
        self.0.lock().unwrap().execute("INSERT INTO memories(id,project_id,category,content,updated_at) VALUES(?1,?2,?3,?4,?5)",params![m.id,m.project_id,m.category,m.content,m.updated_at])?;
        Ok(())
    }
    pub fn delete_memory(&self, id: &str) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .execute("DELETE FROM memories WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let db = self.0.lock().unwrap();
        let mut q = db.prepare("SELECT value FROM settings WHERE key=?1")?;
        let mut rows = q.query([key])?;
        Ok(rows.next()?.map(|r| r.get(0)).transpose()?)
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.0.lock().unwrap().execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value])?;
        Ok(())
    }
    pub fn reset(&self) -> Result<()> {
        self.0.lock().unwrap().execute_batch("DELETE FROM messages;DELETE FROM memories;DELETE FROM tasks;DELETE FROM actions;DELETE FROM projects;DELETE FROM settings;")?;
        Ok(())
    }
}
