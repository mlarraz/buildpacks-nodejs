use crate::vrs::Version;
use anyhow::anyhow;
use serde::Deserialize;

use std::hash::Hash;
use std::collections::HashMap;

const NODE_UPSTREAM_LIST_URL: &str = "https://nodejs.org/download/release/index.json";

#[derive(Deserialize, Eq, PartialEq, Hash, Debug)]
pub struct NodeJSRelease {
    pub version: Version,
    pub files: Vec<String>,
}

pub(crate) fn list_releases() -> anyhow::Result<Vec<NodeJSRelease>> {
    ureq::get(NODE_UPSTREAM_LIST_URL)
        .call()
        .map_err(|e| anyhow!("Couldn't fetch nodejs.org release list: {e}"))?
        .into_json::<Vec<NodeJSRelease>>()
        .map_err(|e| anyhow!("Couldn't serialize nodejs.org release list from json: {e}"))
}

pub(crate) fn fetch_checksums(version: &Version) -> anyhow::Result<HashMap<String, String>> {
    ureq::get(&format!(
        "https://nodejs.org/download/release/v{version}/SHASUMS256.txt"
    ))
    .call()?
    .into_string()
    .map_err(anyhow::Error::from)
    .map(|x| parse_shasums(&x))
}

// Parses a SHASUMS256.txt file into a map of filename to checksum.
// Lines are expected to be of the form `<checksum> <filename>`.
fn parse_shasums(input: &str) -> HashMap<String, String> {
    input
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next(), parts.next()) {
                (Some(checksum), Some(filename), None) => Some((
                    // Some of the checksum filenames contain a leading `./` (e.g.
                    // https://nodejs.org/download/release/v0.11.6/SHASUMS256.txt)
                    filename.trim_start_matches("./").to_string(),
                    checksum.to_string(),
                )),
                _ => None,
            }
        })
        .collect()
}