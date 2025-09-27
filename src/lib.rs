use regex::Regex;
use std::collections::HashMap;
use std::process::Command;

use arboard::Clipboard;
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

fn copy_to_clipboard(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(content.to_string())?;
    Ok(())
}

pub fn clip_commit_prompt(chinese: bool) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = get_commit_prompt(chinese)?;
    copy_to_clipboard(&prompt)?;
    Ok(())
}

pub fn add_commit_push(commit_msg: String) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("git").arg("add").arg(".").output()?;
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(commit_msg)
        .output()?;
    Command::new("git").arg("push").output()?;
    Ok(())
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
