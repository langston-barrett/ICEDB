use std::{collections::HashSet, fs, io::Write};

use regex::Regex;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Label {
    id: usize,
    name: String,
    description: Option<String>,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum IssueState {
    Open,
    Closed,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Issue {
    id: usize,
    number: usize,
    state: IssueState,
    title: String,
    body: Option<String>,
    labels: Vec<Label>,
}

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Serialize)]
struct Ice {
    message: Option<String>,
    query_stack: Option<Vec<String>>,
}

fn message_regex() -> Regex {
    Regex::new(r"(?m)^error: internal compiler error: (?P<file>[^:]+):\d+:\d+: (?P<msg>.+)$")
        .unwrap()
}

fn query_stack_regex() -> Regex {
    Regex::new(r"(?ms)^query stack during panic:\n(?P<stack>.+)end of query stack$").unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let issues = String::from_utf8(fs::read("./db/issues.jsonl")?)?;
    let message_rx = message_regex();
    let stack_rx = query_stack_regex();
    let mut ices = HashSet::new();
    for issue_str in issues.lines() {
        let issue: Issue = serde_json::from_str(issue_str)?;
        debug_assert!(issue.labels.iter().any(|l| l.name == "I-ICE"));
        let body_string = issue.body.unwrap_or_default();
        let message = message_rx
            .captures(&body_string)
            .map(|m| m.name("msg").unwrap().as_str().to_owned());
        let query_stack = stack_rx.captures(&body_string).map(|m| {
            m.name("stack")
                .unwrap()
                .as_str()
                .to_owned()
                .lines()
                .map(|s| s.to_string())
                .collect()
        });
        let ice = Ice {
            message,
            query_stack,
        };
        ices.insert(ice);
    }

    let mut sorted_ices = Vec::from_iter(ices.iter());
    sorted_ices.sort();
    let mut ice_file = fs::File::create("./db/ices.jsonl")?;
    for ice in sorted_ices {
        ice_file.write_all(serde_json::to_string(&ice)?.as_bytes())?;
        ice_file.write_all(&[u8::try_from('\n').unwrap()])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn message_regex() {
        let rx = super::message_regex();
        assert!(rx.is_match("error: internal compiler error: compiler/rustc_infer/src/infer/region_constraints/mod.rs:568:17: cannot relate bound region: ReLateBound(DebruijnIndex(0), BoundRegion { var: 1, kind: BrNamed(DefId(0:8 ~ prefix[b2cc]::longest_common_prefix::'_#1), '_) }) <= '_#29r"))
    }

    #[test]
    fn stack_regex() {
        let rx = super::query_stack_regex();
        assert!(rx.is_match(
            "query stack during panic:
#0 [typeck] type-checking `longest_common_prefix`
#1 [typeck_item_bodies] type-checking all item bodies
#2 [analysis] running analysis passes on this crate
#0 [typeck] type-checking `longest_common_prefix`
#1 [typeck_item_bodies] type-checking all item bodies
#2 [analysis] running analysis passes on this crate
end of query stack"
        ))
    }
}
