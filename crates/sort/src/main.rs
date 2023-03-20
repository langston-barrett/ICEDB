use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
};

use regex::Regex;

use icedb::{Ice, IceWithIssues, Issue, RustcVersion};

fn flags_regex() -> Regex {
    Regex::new(r"(?m)^note: compiler flags: (?P<flags>.+)$").unwrap()
}

fn message_regex() -> Regex {
    Regex::new(r"(?m)^error: internal compiler error: (?P<file>[^:]+):\d+:\d+: (?P<msg>.+)$")
        .unwrap()
}

fn query_stack_regex() -> Regex {
    Regex::new(r"(?ms)^query stack during panic:\n(?P<stack>.+)end of query stack$").unwrap()
}

fn version_regex() -> Regex {
    Regex::new(
        r"(?m)^binary: .+\ncommit-hash: (?P<commit_hash>.+)\ncommit-date: (?P<commit_date>.+)\nhost: (?P<host>.+)\nrelease: (?P<release>.+)\nLLVM version: (?P<llvm_version>.+)$",
    )
    .unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let issues = String::from_utf8(fs::read("./db/issues.jsonl")?)?;
    let flags_rx = flags_regex();
    let message_rx = message_regex();
    let stack_rx = query_stack_regex();
    let version_rx = version_regex();
    let mut ices = HashMap::new();
    for issue_str in issues.lines() {
        let issue: Issue = serde_json::from_str(issue_str)?;
        debug_assert!(issue.labels.iter().any(|l| l.name == "I-ICE"));
        let body_string = issue.body.unwrap_or_default();
        let flags = flags_rx.captures(&body_string).map(|m| {
            m.name("flags")
                .unwrap()
                .as_str()
                .to_owned()
                .split(' ')
                .map(|s| s.to_string())
                .collect()
        });
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
        let version = version_rx.captures(&body_string).map(|m| RustcVersion {
            commit_hash: m.name("commit_hash").unwrap().as_str().to_owned(),
            commit_date: m.name("commit_date").unwrap().as_str().to_owned(),
            host: m.name("host").unwrap().as_str().to_owned(),
            release: m.name("release").unwrap().as_str().to_owned(),
            llvm_version: m.name("llvm_version").unwrap().as_str().to_owned(),
        });
        let ice = Ice {
            flags,
            message,
            query_stack,
            version,
        };
        if ice.message.is_none() && ice.query_stack.is_none() && ice.version.is_none() {
            continue;
        }
        ices.entry(ice)
            .or_insert_with(HashSet::new)
            .insert(issue.number);
    }

    let mut sorted_ice_issues = Vec::from_iter(ices.into_iter().map(|(ice, issue_set)| {
        let mut issue_vec = Vec::from_iter(issue_set.into_iter());
        issue_vec.sort();
        (ice, issue_vec)
    }));
    sorted_ice_issues.sort();
    let mut ice_issues_file = fs::File::create("./db/ices.jsonl")?;
    for (ice, issues) in sorted_ice_issues {
        ice_issues_file
            .write_all(serde_json::to_string(&IceWithIssues { ice, issues })?.as_bytes())?;
        ice_issues_file.write_all(&[u8::try_from('\n').unwrap()])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn flags_regex() {
        let rx = super::flags_regex();
        assert!(rx.is_match("note: compiler flags: -C embed-bitcode=no -C split-debuginfo=unpacked -C debuginfo=2 -C incremental=[REDACTED]"))
    }

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

    #[test]
    fn version_regex() {
        let rx = super::version_regex();
        assert!(rx.is_match(
            "rustc 1.68.0 (2c8cc3432 2023-03-06) (built from a source tarball)
binary: rustc
commit-hash: 2c8cc343237b8f7d5a3c3703e3a87f2eb2c54a74
commit-date: 2023-03-06
host: aarch64-apple-darwin
release: 1.68.0
LLVM version: 15.0.6"
        ))
    }
}
