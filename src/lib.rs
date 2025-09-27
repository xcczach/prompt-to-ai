use std::collections::HashMap;

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
            changes.insert(file_path.to_string(), format!("Add {}", file_path));
        } else if status.contains(Status::WT_MODIFIED) {
            changes.insert(file_path.to_string(), format!("Modify {}", file_path));
        } else if status.contains(Status::WT_DELETED) {
            changes.insert(file_path.to_string(), format!("Delete {}", file_path));
        }
    }

    let index = repo.index()?;
    let diff = repo.diff_index_to_workdir(Some(&index), None)?;
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let sign = line.origin();
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        if sign == '+' || sign == '-' {
            changes.push(format!("{} {}\n", sign, content.trim_end()));
        }
        true
    })?;

    Ok(changes.join("\n"))
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
