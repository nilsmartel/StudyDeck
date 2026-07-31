//! Loading question scenarios from a directory of JSON documents.

use std::fs;
use std::path::Path;

use crate::question::QuestionScenario;

/// Read every `*.json` document in `dir`, deserializing each into a
/// [`QuestionScenario`]. Files that cannot be read or parsed are reported on
/// stderr via `eprintln!` and skipped, so a single malformed document never
/// aborts the whole load. Results are returned sorted by file name for a
/// deterministic question order.
pub fn read_scenarios(dir: &Path) -> Vec<QuestionScenario> {
    let mut scenarios = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("could not read directory {}: {err}", dir.display());
            return scenarios;
        }
    };

    // Collect the JSON paths first so we can sort them deterministically.
    let mut paths: Vec<_> = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    paths.push(path);
                }
            }
            Err(err) => eprintln!("could not read a directory entry in {}: {err}", dir.display()),
        }
    }
    paths.sort();

    for path in paths {
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("could not read {}: {err}", path.display());
                continue;
            }
        };

        match serde_json::from_str::<QuestionScenario>(&contents) {
            Ok(scenario) => scenarios.push(scenario),
            Err(err) => eprintln!("could not parse {}: {err}", path.display()),
        }
    }

    scenarios
}
