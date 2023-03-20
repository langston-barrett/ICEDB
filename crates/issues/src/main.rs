use std::{fs::File, io::Write};

use serde_json::Value;

mod github;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = github::Config::from_env().unwrap();
    let issues =
        github::get_labeled_issues(&config, "rust-lang/rust", "I-ICE".to_string()).unwrap();
    let mut f = File::create("./db/issues.jsonl")?;
    for issue_value in issues {
        if let Value::Object(issue_obj) = issue_value {
            f.write_all(serde_json::to_string(&issue_obj)?.as_bytes())?;
            f.write_all(&[u8::try_from('\n').unwrap()])?;
        } else {
            eprintln!("[WARN] issue was not an object");
        }
    }
    Ok(())
}
