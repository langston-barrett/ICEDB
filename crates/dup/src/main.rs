use std::{
    collections::{HashMap, HashSet},
    fs,
};

use icedb::{IceWithIssues, Issue, IssueState};

fn read_ices_with_issues() -> Result<HashSet<IceWithIssues>, Box<dyn std::error::Error>> {
    let ice_file = fs::read_to_string("./db/ices.jsonl")?;
    let mut ices = HashSet::new();
    for ice_str in ice_file.lines() {
        ices.insert(serde_json::from_str(ice_str)?);
    }
    Ok(ices)
}

fn read_issues() -> Result<HashSet<Issue>, Box<dyn std::error::Error>> {
    let issue_file = fs::read_to_string("./db/issues.jsonl")?;
    let mut issues = HashSet::new();
    for ice_str in issue_file.lines() {
        issues.insert(serde_json::from_str(ice_str)?);
    }
    Ok(issues)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[WARN] This program is not actually great at determining duplicates.");
    let ices = read_ices_with_issues()?;
    let issues = read_issues()?;
    let issue_map = issues
        .into_iter()
        .map(|i| (i.number, i))
        .collect::<HashMap<_, _>>();
    for ice in &ices {
        if ice.issues.len() < 2 || ice.ice.message.is_none() || ice.ice.query_stack.is_none() {
            continue;
        }
        let _some_open = ice.issues.iter().any(|i| {
            issue_map
                .get(i)
                .map(|iss| iss.state == IssueState::Open)
                .unwrap_or(false)
        });
        eprintln!("{:?}", ice.issues);
    }
    Ok(())
}
