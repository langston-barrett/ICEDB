#[derive(
    Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct Label {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
}

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct Issue {
    pub id: usize,
    pub number: usize,
    pub state: IssueState,
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<Label>,
}

/// Information returned by rustc --version --verbose
#[derive(
    Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct RustcVersion {
    pub commit_hash: String,
    pub commit_date: String,
    pub host: String,
    pub release: String,
    pub llvm_version: String,
}

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct Ice {
    pub backtrace: Option<Vec<String>>,
    pub flags: Option<Vec<String>>,
    pub issue: usize,
    pub message: Option<String>,
    pub query_stack: Option<Vec<String>>,
    pub version: Option<RustcVersion>,
}
