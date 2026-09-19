//! The portfolio registry. Deliberately just the repos verified public and
//! release-pipelined this session (plus a few known-public flagships from
//! memory) rather than every repo in the ecosystem -- a private repo here
//! would just make every command report "unknown," which is more confusing
//! than a short, accurate list. Add entries as more repos get the full
//! treatment and go public.

const REPOS: &[(&str, &str)] = &[
    ("echo", "darkstardevx/echo"),
    ("cybermeta", "darkstardevx/cybermeta"),
    ("cybervault", "darkstardevx/cybervault"),
    ("keysmith", "darkstardevx/keysmith"),
    ("cyberterm", "darkstardevx/cyberterm"),
    ("sentrygrid", "darkstardevx/sentrygrid"),
    ("wraithflow", "darkstardevx/wraithflow"),
    ("vortexwall", "darkstardevx/vortexwall"),
    ("aetherscope", "darkstardevx/aetherscope"),
    ("ghostport", "darkstardevx/ghostport"),
];

pub fn resolve(name: &str) -> Option<&'static str> {
    let name = name.trim().to_lowercase();
    REPOS.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

pub fn all() -> impl Iterator<Item = &'static str> {
    REPOS.iter().map(|(_, r)| *r)
}

pub fn names_list() -> String {
    REPOS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}
