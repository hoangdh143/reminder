// src/main.rs
use chrono::{DateTime, Duration, Local};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "reminder")]
#[command(about = "A spaced repetition reminder system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Trim the reminder content to a specific number of characters
    #[arg(long, value_name = "NUMBER")]
    trim: Option<usize>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new reminder
    Add {
        /// The content to remember
        #[arg(value_name = "CONTENT")]
        content: String,
    },
    /// Check for due reminders
    Check,
    /// List all reminders
    List,
    /// Mark a reminder as reviewed
    Review {
        /// The ID of the reminder to mark as reviewed
        #[arg(value_name = "ID")]
        id: u32,
    },
    /// Remove a reminder
    Remove {
        /// The ID of the reminder to remove
        #[arg(value_name = "ID")]
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Reminder {
    id: u32,
    content: String,
    created_at: DateTime<Local>,
    next_review: DateTime<Local>,
    review_count: u32,
    completed: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct ReminderStore {
    reminders: HashMap<u32, Reminder>,
    next_id: u32,
}

impl ReminderStore {
    fn load() -> Self {
        let file_path = get_data_file_path();

        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .expect("Failed to read reminder file");

            serde_json::from_str(&content)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Could not parse reminder file ({}). Starting fresh.", e);
                    Self::default()
                })
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let file_path = get_data_file_path();

        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .expect("Failed to create data directory");
        }

        let content = serde_json::to_string_pretty(self)
            .expect("Failed to serialize reminders");

        fs::write(&file_path, content)
            .expect("Failed to write reminder file");
    }

    fn add_reminder(&mut self, content: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let now = Local::now();
        let reminder = Reminder {
            id,
            content,
            created_at: now,
            next_review: now + Duration::days(1), // First review after 1 day
            review_count: 0,
            completed: false,
        };

        self.reminders.insert(id, reminder);
        id
    }

    fn review_reminder(&mut self, id: u32) -> Result<(), String> {
        let reminder = self.reminders.get_mut(&id)
            .ok_or_else(|| format!("Reminder with ID {} not found", id))?;

        if reminder.completed {
            return Err("Reminder is already completed".to_string());
        }

        reminder.review_count += 1;

        // Schedule next review based on spaced repetition intervals
        let next_interval = match reminder.review_count {
            1 => Duration::days(3),   // 3 days after first review
            2 => Duration::weeks(1),  // 1 week after second review
            3 => Duration::days(30),  // 1 month after third review
            _ => {
                reminder.completed = true;
                return Ok(());
            }
        };

        reminder.next_review = Local::now() + next_interval;
        Ok(())
    }

    fn get_due_reminders(&self) -> Vec<&Reminder> {
        let now = Local::now();
        self.reminders
            .values()
            .filter(|r| !r.completed && r.next_review <= now)
            .collect()
    }

    fn get_all_reminders(&self) -> Vec<&Reminder> {
        let mut reminders: Vec<&Reminder> = self.reminders.values().collect();
        reminders.sort_by_key(|r| r.next_review);
        reminders
    }

    fn remove_reminder(&mut self, id: u32) -> Result<(), String> {
        self.reminders.remove(&id)
            .ok_or_else(|| format!("Reminder with ID {} not found", id))?;
        Ok(())
    }
}

fn get_data_file_path() -> PathBuf {
    let mut path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".")); // Fallback to current dir if data_dir is not available
    path.push("reminder"); // Changed to avoid potential conflict with other apps
    path.push("reminders.json");
    path
}

fn format_duration_until(datetime: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = datetime.signed_duration_since(now);

    if duration.num_seconds() < 0 {
        let abs_duration = -duration;
        if abs_duration.num_days() > 0 {
            format!("{} days ago", abs_duration.num_days())
        } else if abs_duration.num_hours() > 0 {
            format!("{} hours ago", abs_duration.num_hours())
        } else if abs_duration.num_minutes() > 0 {
            format!("{} minutes ago", abs_duration.num_minutes())
        } else {
             format!("{} seconds ago", abs_duration.num_seconds())
        }
    } else {
        if duration.num_days() > 0 {
            format!("in {} days", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("in {} hours", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("in {} minutes", duration.num_minutes())
        } else {
            format!("in {} seconds", duration.num_seconds())
        }
    }
}

// Helper function to trim content
fn trim_content(content: &str, max_len: Option<usize>) -> String {
    match max_len {
        Some(len) => {
            if content.chars().count() > len {
                content.chars().take(len).collect::<String>() + "..."
            } else {
                content.to_string()
            }
        }
        None => content.to_string(),
    }
}


fn main() {
    let cli = Cli::parse();
    let mut store = ReminderStore::load();

    match cli.command {
        Commands::Add { content } => {
            let id = store.add_reminder(content.clone());
            store.save();
            let display_content = trim_content(&content, cli.trim);
            println!("Added reminder with ID {}: \"{}\"", id, display_content);
            println!("Next review: 1 day from now");
        }

        Commands::Check => {
            let due_reminders = store.get_due_reminders();

            if due_reminders.is_empty() {
                println!("No reminders due for review!");
            } else {
                println!("Reminders due for review:");
                println!("{}", "=".repeat(50));

                for reminder in due_reminders {
                    let display_content = trim_content(&reminder.content, cli.trim);
                    println!("ID: {}", reminder.id);
                    println!("Content: {}", display_content);
                    println!("Review count: {}", reminder.review_count);
                    println!("Due: {}", format_duration_until(reminder.next_review));
                    println!("{}", "-".repeat(30));
                }

                println!("\nUse 'reminder review <ID>' to mark a reminder as reviewed");
            }
        }

        Commands::List => {
            let reminders = store.get_all_reminders();

            if reminders.is_empty() {
                println!("No reminders found!");
            } else {
                println!("All reminders:");
                println!("{}", "=".repeat(70));

                for reminder in reminders {
                    let status = if reminder.completed {
                        "✓ Completed"
                    } else {
                        "⏳ Active"
                    };

                    let display_content = trim_content(&reminder.content, cli.trim);
                    println!("ID: {} | {} | Reviews: {}",
                             reminder.id, status, reminder.review_count);
                    println!("Content: {}", display_content);

                    if !reminder.completed {
                        println!("Next review: {}", format_duration_until(reminder.next_review));
                    }

                    println!("{}", "-".repeat(50));
                }
            }
        }

        Commands::Review { id } => {
            match store.review_reminder(id) {
                Ok(()) => {
                    store.save(); // Save before accessing reminder to ensure it's up-to-date
                    let reminder = &store.reminders[&id]; // Re-fetch to get updated state
                    if reminder.completed {
                        println!("Reminder {} completed! 🎉", id);
                        println!("You've successfully reviewed this {} times.", reminder.review_count);
                    } else {
                        println!("Reminder {} reviewed!", id);
                        println!("Next review: {}", format_duration_until(reminder.next_review));
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::Remove { id } => {
            match store.remove_reminder(id) {
                Ok(()) => {
                    println!("Reminder {} removed successfully", id);
                    store.save();
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
