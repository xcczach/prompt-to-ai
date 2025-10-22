use regex::Regex;
use std::collections::HashMap;
use std::process::Command;

use arboard::Clipboard;
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use git2::{DiffFormat, Repository, Status, StatusOptions};

fn get_change_str() -> Result<String, Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .renames_from_rewrites(true);
    let statuses = repo.statuses(Some(&mut opts))?;

    let mut changes: HashMap<String, String> = HashMap::new();

    for entry in statuses.iter() {
        let status = entry.status();
        let file_path = entry.path().unwrap_or("unknown");

        if status.contains(Status::WT_NEW) {
            changes.insert(file_path.to_string(), format!("Add {}\n", file_path));
        } else if status.contains(Status::WT_MODIFIED) {
            changes.insert(file_path.to_string(), format!("Modify {}\n", file_path));
        } else if status.contains(Status::WT_DELETED) {
            changes.insert(file_path.to_string(), format!("Delete {}\n", file_path));
        }
    }

    let mut current_filename: Option<String> = None;
    let mut file_content_starts = false;
    let filename_re = Regex::new(r"diff --git a/(.+?) b/(.+?)").unwrap();
    let index = repo.index()?;
    let diff = repo.diff_index_to_workdir(Some(&index), None)?;
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        // Capture file name from diff header
        if let Some(caps) = filename_re.captures(content) {
            current_filename = Some((&caps[1]).to_owned());
            file_content_starts = false;
        }
        // Push changes to the according entry
        else if let Some(filename) = &current_filename {
            // Match @@ to start file content
            if content.starts_with("@@") {
                file_content_starts = true;
            }
            if file_content_starts {
                let entry = changes
                    .entry(filename.to_string())
                    .or_insert_with(String::new);
                let sign = line.origin();
                entry.push_str(&format!("{} {}", sign, content));
            }
        }
        true
    })?;

    Ok(changes.values().cloned().collect::<Vec<_>>().join("\n"))
}

fn get_commit_prompt(chinese: bool) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = r#"You are an AI assistant that creates commit messages.
Given the following changes in a git repository, provide a concise summary of what has been changed, added, or removed. Focus on the overall purpose and impact of the changes rather than line-by-line details.
Choose one of the following types for the commit message:
feature: <A brief description of the feature added or modified>
fix: <A brief description of the bug fixed>
refactor: <A brief description of the code refactored>
docs: <A brief description of the documentation updated>
chore: <A brief description of the maintenance task performed>

Only provide the commit message without any additional commentary or explanation. Strictly follow the format: <type>: <description>. If multiple types apply, choose the most significant one.
"#;
    let mut prompt = prompt.to_string();
    if chinese {
        prompt.push_str("Please respond in Chinese for the body of commit messages.\n");
    }
    let change_str = get_change_str()?;
    Ok(format!("{}\n\nChanges:\n{}", prompt, change_str))
}

fn clip_or_print(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(content.to_string())?;
        println!("Commit prompt copied to clipboard.");
    } else {
        let path = Path::new(".commit_prompt.txt");
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!(
            "Clipboard not available; commit prompt saved to .commit_prompt.txt"
        );
        // Auto-register a cleaner so the file is deleted on exit
        register_prompt_cleaner();
    }
    Ok(())
}

pub fn clip_commit_prompt(chinese: bool) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = get_commit_prompt(chinese)?;
    clip_or_print(&prompt)?;
    Ok(())
}

pub fn add_commit(commit_msg: String) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("git").arg("add").arg(".").output()?;
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(commit_msg)
        .output()?;
    // Best-effort cleanup of temporary prompt file after commit
    let _ = fs::remove_file(".commit_prompt.txt");
    Ok(())
}

// RAII guard to clean up the temporary commit prompt file on drop
pub struct TempPromptCleaner(PathBuf);

impl Drop for TempPromptCleaner {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

// Create a cleaner for the default prompt file path
pub fn temp_prompt_cleaner() -> TempPromptCleaner {
    TempPromptCleaner(PathBuf::from(".commit_prompt.txt"))
}

thread_local! {
    static PROMPT_CLEANER: RefCell<Option<TempPromptCleaner>> = RefCell::new(None);
}

fn register_prompt_cleaner() {
    PROMPT_CLEANER.with(|cell| {
        if cell.borrow().is_none() {
            cell.replace(Some(temp_prompt_cleaner()));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_change_str() {
        match get_change_str() {
            Ok(changes) => {
                println!("Changes:\n{}", changes);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
