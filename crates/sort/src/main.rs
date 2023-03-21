use std::{collections::HashSet, fs, io::Write};

use regex::Regex;

use icedb::{Ice, Issue, RustcVersion};

fn backtrace_regex() -> Regex {
    Regex::new(r"(?m)^stack backtrace:(?P<backtrace>(\r?\n +\d+:.+$|\r?\n +at .+$)+)$").unwrap()
}

fn flags_regex() -> Regex {
    Regex::new(r"(?m)^note: compiler flags: (?P<flags>.+)$").unwrap()
}

fn ice_message_regex() -> Regex {
    Regex::new(r"(?m)^error: internal compiler error: (?P<file>[^:]+):\d+:\d+: (?P<msg>.+)$")
        .unwrap()
}

fn panic_message_regex() -> Regex {
    Regex::new(r"(?m)^thread 'rustc' panicked at '(?P<msg>[^']+)', .+$").unwrap()
}

fn query_stack_regex() -> Regex {
    Regex::new(r"(?ms)^query stack during panic:\r?\n(?P<stack>.+)end of query stack$").unwrap()
}

fn version_regex() -> Regex {
    Regex::new(
        r"(?m)^binary: .+\r?\ncommit-hash: (?P<commit_hash>.+)\r?\ncommit-date: (?P<commit_date>.+)\nhost: (?P<host>.+)\nrelease: (?P<release>.+)\nLLVM version: (?P<llvm_version>.+)$",
    )
    .unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let issues = String::from_utf8(fs::read("./db/issues.jsonl")?)?;
    let backtrace_rx = backtrace_regex();
    let flags_rx = flags_regex();
    let ice_message_rx = ice_message_regex();
    let panic_message_rx = panic_message_regex();
    let stack_rx = query_stack_regex();
    let version_rx = version_regex();
    let mut ices = HashSet::new();
    for issue_str in issues.lines() {
        let issue: Issue = serde_json::from_str(issue_str)?;
        debug_assert!(issue.labels.iter().any(|l| l.name == "I-ICE"));
        let body_string = issue.body.unwrap_or_default();
        let backtrace = backtrace_rx.captures(&body_string).map(|m| {
            m.name("backtrace")
                .unwrap()
                .as_str()
                .to_owned()
                .split('\n')
                .map(|s| s.trim().to_string())
                .collect()
        });
        let flags = flags_rx.captures(&body_string).map(|m| {
            m.name("flags")
                .unwrap()
                .as_str()
                .to_owned()
                .split(' ')
                .map(|s| s.to_string())
                .collect()
        });
        let ice_message = ice_message_rx
            .captures(&body_string)
            .map(|m| m.name("msg").unwrap().as_str().to_owned());
        let panic_message = panic_message_rx
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
            backtrace,
            flags,
            ice_message,
            issue: issue.number,
            panic_message,
            query_stack,
            version,
        };
        if ice.backtrace.is_none()
            && ice.flags.is_none()
            && ice.ice_message.is_none()
            && ice.query_stack.is_none()
            && ice.panic_message.is_none()
            && ice.version.is_none()
        {
            continue;
        }
        ices.insert(ice);
    }

    let mut sorted_ices = Vec::from_iter(ices.into_iter());
    sorted_ices.sort_by(|i, j| i.issue.cmp(&j.issue));
    let mut ice_issues_file = fs::File::create("./db/ices.jsonl")?;
    for ice in sorted_ices {
        ice_issues_file.write_all(serde_json::to_string(&ice)?.as_bytes())?;
        ice_issues_file.write_all(&[u8::try_from('\n').unwrap()])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use icedb::Issue;

    fn get_issue(number: usize) -> Issue {
        // TODO: read per line
        let issue_file = std::fs::read_to_string(format!(
            "{}/../.././db/issues.jsonl",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        for ice_str in issue_file.lines() {
            let issue: Issue = serde_json::from_str(ice_str).unwrap();
            if issue.number == number {
                return issue;
            }
        }
        unreachable!()
    }

    #[test]
    fn backtrace_regex() {
        let rx = super::backtrace_regex();
        assert!(rx.is_match("stack backtrace:
   0:        0x10373bd1c - <std::sys_common::backtrace::_print::DisplayBacktrace as core::fmt::Display>::fmt::h5c2d00a9fd17401b
  80:        0x1a3be9240 - __pthread_deallocate"
        ));
        assert!(rx.is_match("stack backtrace:
   0:     0x7f1304f6ec90 - std::backtrace_rs::backtrace::libunwind::trace::h4c56f7c1d2b54c49
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/std/src/../../backtrace/src/backtrace/mod.rs:66:5
   1:     0x7f1304f6ec90 - std::backtrace_rs::backtrace::trace_unsynchronized::h43647f7dfa7709b7
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/std/src/../../backtrace/src/backtrace/mod.rs:66:5
   2:     0x7f1304f6ec90 - std::sys_common::backtrace::_print_fmt::hb05bf7e901883977
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/std/src/sys_common/backtrace.rs:66:5
   3:     0x7f1304f6ec90 - <std::sys_common::backtrace::_print::DisplayBacktrace as core::fmt::Display>::fmt::hd3f800102e692f91
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/std/src/sys_common/backtrace.rs:45:22
   4:     0x7f1304fc99ee - core::fmt::write::h7e5f4e1d134bd366
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/core/src/fmt/mod.rs:1202:17
   5:     0x7f1304f5f7a5 - std::io::Write::write_fmt::h51d5f9bde508a4b0
                               at /rustc/98ad6a5519651af36e246c0335c964dd52c554ba/library/std/src/io/mod.rs:1679:15"
        ));
        eprintln!("{}", get_issue(107990).body.unwrap());
        assert!(rx.is_match(&get_issue(107990).body.unwrap()))
    }

    #[test]
    fn flags_regex() {
        let rx = super::flags_regex();
        assert!(rx.is_match("note: compiler flags: -C embed-bitcode=no -C split-debuginfo=unpacked -C debuginfo=2 -C incremental=[REDACTED]"))
    }

    #[test]
    fn ice_message_regex() {
        let rx = super::ice_message_regex();
        assert!(rx.is_match("error: internal compiler error: compiler/rustc_infer/src/infer/region_constraints/mod.rs:568:17: cannot relate bound region: ReLateBound(DebruijnIndex(0), BoundRegion { var: 1, kind: BrNamed(DefId(0:8 ~ prefix[b2cc]::longest_common_prefix::'_#1), '_) }) <= '_#29r"));
    }

    #[test]
    fn panic_message_regex() {
        let rx = super::panic_message_regex();
        assert!(rx.is_match(r#"thread 'rustc' panicked at 'no resolution for "Path" MacroNS DefId(0:2566 ~ skia_safe[0ba6]::core::path_types)', src/librustdoc/passes/collect_intra_doc_links.rs:393:32"#));
    }

    #[test]
    fn stack_regex() {
        let rx = super::query_stack_regex();
        assert!(rx.is_match(
            "query stack during panic:
#0 [typeck] type-checking `longest_commggon_prefix`
#1 [typeck_item_bodies] type-checking all item bodies
#2 [analysis] running analysis passes on this crate
#0 [typeck] type-checking `longest_common_prefix`
#1 [typeck_item_bodies] type-checking all item bodies
#2 [analysis] running analysis passes on this crate
end of query stack"
        ));
        assert!(rx.is_match(
            "query stack during panic:
#0 [typeck] type-checking `b`
#1 [typeck_item_bodies] type-checking all item bodies
#2 [analysis] running analysis passes on this crate
end of query stack"
        ));
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
