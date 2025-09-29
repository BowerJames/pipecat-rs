use anyhow::{anyhow, Context, Result};
use glob::glob;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Spec {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub fail_after: Option<HumanDuration>,
    #[serde(default)]
    pub time_epsilon: Option<HumanDuration>,
    #[serde(default)]
    pub default_within: Option<HumanDuration>,
    pub pipeline: Vec<Node>,
    pub script: Vec<Step>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(alias = "name")]
    pub id: String,
    #[serde(alias = "type")]
    pub kind: String,
    #[serde(default)]
    pub config: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op")]
pub enum Step {
    #[serde(rename = "await")]
    Await {
        node: String,
        direction: Direction,
        pattern: Pattern,
        #[serde(default)]
        within: Option<HumanDuration>,
    },
    #[serde(rename = "send")]
    Send {
        node: String,
        direction: Direction,
        after: HumanDuration,
        pattern: Pattern,
    },
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Downstream,
    Upstream,
}

#[derive(Debug, Deserialize)]
pub struct Pattern {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub body: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct HumanDuration(pub u64); // milliseconds

impl<'de> Deserialize<'de> for HumanDuration {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_duration_ms(&s).map(HumanDuration).map_err(serde::de::Error::custom)
    }
}

fn parse_duration_ms(s: &str) -> std::result::Result<u64, String> {
    // supports: <num>ms or <num>s
    if let Some(ms) = s.strip_suffix("ms") {
        let v: u64 = ms.trim().parse().map_err(|_| format!("Invalid duration: {s}"))?;
        return Ok(v);
    }
    if let Some(secs) = s.strip_suffix('s') {
        let v: u64 = secs.trim().parse().map_err(|_| format!("Invalid duration: {s}"))?;
        return Ok(v * 1000);
    }
    Err(format!("Duration must end with 'ms' or 's': {s}"))
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationErrorKind {
    #[error("unsupported version {0}")]
    UnsupportedVersion(u32),
    #[error("duplicate node id '{0}'")]
    DuplicateNodeId(String),
    #[error("unknown node '{0}' in step {1}")]
    UnknownNode(String, usize),
    #[error("missing within for await at step {0} and no default_within set")]
    MissingWithin(usize),
}

#[derive(Debug, thiserror::Error)]
#[error("validation failed")] 
pub struct ValidationError {
    pub errors: Vec<ValidationErrorKind>,
}

pub fn load_spec_from_file(path: &Path) -> Result<Spec> {
    let s = fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    let spec: Spec = serde_yaml::from_str(&s).with_context(|| format!("parsing YAML in {path:?}"))?;
    Ok(spec)
}

pub fn validate_spec(spec: &Spec) -> std::result::Result<(), ValidationError> {
    let mut errs: Vec<ValidationErrorKind> = Vec::new();

    if spec.version != 1 {
        errs.push(ValidationErrorKind::UnsupportedVersion(spec.version));
    }

    // unique node ids
    let mut seen: HashSet<&str> = HashSet::new();
    for n in &spec.pipeline {
        if !seen.insert(&n.id) {
            errs.push(ValidationErrorKind::DuplicateNodeId(n.id.clone()));
        }
    }

    // known nodes in steps and await-within requirement
    for (i, step) in spec.script.iter().enumerate() {
        match step {
            Step::Await { node, within, .. } => {
                if !seen.contains(node.as_str()) {
                    errs.push(ValidationErrorKind::UnknownNode(node.clone(), i));
                }
                if within.is_none() && spec.default_within.is_none() {
                    errs.push(ValidationErrorKind::MissingWithin(i));
                }
            }
            Step::Send { node, .. } => {
                if !seen.contains(node.as_str()) {
                    errs.push(ValidationErrorKind::UnknownNode(node.clone(), i));
                }
            }
        }
    }

    if errs.is_empty() { Ok(()) } else { Err(ValidationError { errors: errs }) }
}

pub fn validate_all_specs_in(dir_glob: &str) -> Result<()> {
    let mut any_err = Vec::new();
    for entry in glob(dir_glob).context("invalid glob pattern")? {
        let path = entry?;
        if path.is_file() {
            let spec = load_spec_from_file(&path)?;
            match validate_spec(&spec) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Spec {:?} failed validation: {}", path, e);
                    for err in e.errors {
                        eprintln!("  - {}", err);
                    }
                    any_err.push(path);
                }
            }
        }
    }
    if any_err.is_empty() { Ok(()) } else { Err(anyhow!("{} spec(s) failed validation", any_err.len())) }
}


