#![allow(dead_code)]
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use console::Term;
use directories::ProjectDirs;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}
#[derive(Parser, Debug)]
enum Commands {
    #[clap(name = "add")]
    Add {
        label: String,
        username: String,
        password: String,
    },
    // debug only
    #[clap(name = "list")]
    List,
    #[clap(name = "get")]
    Get { label: String },
    #[clap(name = "remove")]
    Remove { label: String },
}

use cocoon::MiniCocoon;
use rusqlite::{params, Connection};
use strsim::levenshtein;

// Define a struct to represent a password entry
struct PasswordEntry {
    label: String,
    description: String,
    username: String,
    password: String,
    // Other fields like URL, additional notes, etc.
}

// Define a struct to represent the password manager
struct PasswordManager {
    conn: Connection,
}

impl PasswordManager {
    fn new() -> Result<Self> {
        let db_dir =
            ProjectDirs::from("com", "phant80m", "amanah").expect("project directory to exist");
        if !db_dir.config_dir().exists() {
            std::fs::create_dir_all(db_dir.config_dir())?;
        }
        let conn = Connection::open(db_dir.config_dir().join("passwords.db"))?;
        conn.execute("PRAGMA key = 'rust';", [])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS passwords (
              id INTEGER PRIMARY KEY,
              label TEXT NOT NULL,
              description TEXT NOT NULL,
              username TEXT NOT NULL,
              password BLOB NOT NULL
          )",
            [],
        )?;

        Ok(PasswordManager { conn })
    }
    fn add_entry(&mut self, entry: &PasswordEntry) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM passwords WHERE label = ?1")?;
        let count: i64 = stmt.query_row(params![entry.label], |row| row.get(0))?;
        if count > 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows.into()); // Entry with the same label already exists
        }
        let mut cocoon = MiniCocoon::from_key(b"0123456789abcdef0123456789abcdef", &[0; 32]);

        let wrapped = cocoon
            .wrap(entry.password.as_bytes())
            .expect("wrap password");
        assert_ne!(&wrapped, &entry.password.as_bytes());

        let b64 = general_purpose::STANDARD.encode(wrapped.as_slice());

        self.conn.execute(
            "INSERT INTO passwords (label, description, username, password) VALUES (?1, ?2, ?3, ?4)",
            params![entry.label, String::default(), entry.username, b64],
        )?;
        Ok(())
    }

    fn get_entries(&mut self) -> Result<Vec<PasswordEntry>> {
        let cocoon = MiniCocoon::from_key(b"0123456789abcdef0123456789abcdef", &[0; 32]);

        let mut stmt = self
            .conn
            .prepare("SELECT label, description, username, password FROM passwords")?;
        let password_iter = stmt.query_map([], |row| {
            Ok(PasswordEntry {
                label: row.get(0)?,
                description: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for password_result in password_iter {
            let mut password = password_result?;
            let b64 = general_purpose::STANDARD.decode(password.password.as_bytes())?;
            let unwrapped = cocoon.unwrap(b64.as_slice()).unwrap();
            password.password = String::from_utf8(unwrapped)?;
            entries.push(password);
        }
        Ok(entries)
    }

    fn remove_entry(&mut self, label: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM passwords WHERE label = ?1", params![label])?;
        Ok(())
    }
    fn search_password(&mut self, query: &str) -> Vec<PasswordEntry> {
        let mut matches: Vec<PasswordEntry> = Vec::new();

        // Retrieve all entries from the database
        let entries = match self.get_entries() {
            Ok(entries) => entries,
            Err(_) => return Vec::new(), // Return an empty vector if an error occurs
        };

        for entry in entries {
            let similarity = levenshtein(&entry.label, query);
            if similarity <= 2 {
                matches.push(entry);
            }
        }

        matches
    }
}
impl Args {
    fn handle(self) -> Result<()> {
        let mut password_manager = PasswordManager::new()?;
        match self.commands {
            Commands::Add {
                label,
                username,
                password,
            } => {
                let entry = PasswordEntry {
                    label,
                    // we dont want it
                    description: String::new(),
                    username,
                    password,
                };
                match password_manager.add_entry(&entry) {
                    Ok(_) => println!("Added: {} to list of passwords", &entry.label),
                    Err(e) => {
                        if let Some(rusqlite_error) = e.downcast_ref::<rusqlite::Error>() {
                            if let rusqlite::Error::QueryReturnedNoRows = rusqlite_error {
                                println!("{} already exists", &entry.label);
                            }
                        }
                    }
                }
            }
            Commands::List => {
                let entries = password_manager.get_entries()?;
                for entry in entries {
                    println!(
                        "Label: {}\n Username: {}\n Password: {}\n",
                        entry.label, entry.username, entry.password
                    );
                }
            }
            Commands::Get { label } => {
                let search_results = password_manager.search_password(&label);
                if search_results.is_empty() {
                    println!("No matching passwords found.");
                } else {
                    for entry in search_results {
                        if label != entry.label {
                            println!("no password for {}, did you mean: {}?", label, entry.label);
                        }
                        println!(
                            "Label: {}\nUsername: {}\nPassword: {}\n",
                            entry.label, entry.username, entry.password
                        );
                    }
                }
            }
            Commands::Remove { label } => {
                println!("Type 'Delete: {}.' to confirm deletion", &label);
                let term = Term::stdout();
                let input = term.read_line()?;
                if input.trim() == format!("Delete: {}.", &label) {
                    password_manager.remove_entry(&label)?;
                    println!("deleted entry {}", &label)
                } else {
                    println!("not deleting entry {}", &label)
                }
            }
        }
        Ok(())
    }
}
fn main() -> Result<()> {
    Args::parse().handle()?;
    Ok(())
}
