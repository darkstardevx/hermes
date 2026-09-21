//! The portfolio registry -- every public darkstardevx repo that's actually
//! part of the Cybercore family (queried live via `gh api
//! users/darkstardevx/repos`, not guessed from memory). Deliberately
//! excludes: forks (OmNote), the profile README and hub-site repos
//! (darkstardevx, darkstardevx.github.io -- not binary tools with
//! releases), and ferraxis/vexlang (real projects, but a separate
//! compiler/language-design line of work, not Cybercore). A private repo
//! here would just make every command report "unknown," so keep this in
//! sync with reality rather than adding speculatively.

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
    ("apexdaemon", "darkstardevx/apexdaemon"),
    ("cybercore", "darkstardevx/cybercore"),
    ("cyberdeck", "darkstardevx/cyberdeck"),
    ("cyberplug", "darkstardevx/cyberplug"),
    ("cyberplug-bar-widget", "darkstardevx/cyberplug-bar-widget"),
    ("diagprint", "darkstardevx/diagprint"),
    ("gateflow", "darkstardevx/gateflow"),
    ("agentforge", "darkstardevx/agentforge"),
];

const FEATURED_TOOLS: &[(&str, &str, &str)] = &[
    (
        "cybercore",
        "The shared design system every tool above is built on.",
        "https://darkstardevx.github.io/cybercore/",
    ),
    (
        "cyberdeck",
        "Systems intelligence framework.",
        "https://darkstardevx.github.io/cyberdeck/",
    ),
    (
        "diagprint",
        "Rust diagnostics lifecycle framework.",
        "https://darkstardevx.github.io/diagprint/",
    ),
    (
        "gateflow",
        "Kernel-sandbox testing with netns, chaos, and veth.",
        "https://darkstardevx.github.io/gateflow/",
    ),
    (
        "cybermeta",
        "TUI EXIF metadata tool.",
        "https://darkstardevx.github.io/cybermeta/",
    ),
];

pub fn resolve(name: &str) -> Option<&'static str> {
    let name = name.trim().to_lowercase();
    REPOS.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

pub fn all() -> impl Iterator<Item = &'static str> {
    REPOS.iter().map(|(_, r)| *r)
}

pub fn all_named() -> impl Iterator<Item = (&'static str, &'static str)> {
    REPOS.iter().copied()
}

pub fn featured() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
    FEATURED_TOOLS.iter().copied()
}

pub fn names_list() -> String {
    REPOS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}

pub fn description(name: &str) -> &'static str {
    match name {
        "agentforge" => "Deterministic AI-assisted workflow orchestration.",
        "apexdaemon" => "Background automation for theme sync, fleet health, and backups.",
        "aetherscope" => "Packet capture and BPF-filtered network inspection.",
        "cybercore" => "The shared Cybercore design system and foundations.",
        "cyberdeck" => "Systems intelligence framework.",
        "cybermeta" => "Terminal EXIF metadata tooling.",
        "cyberplug" => "Plugin manager for Omarchy.",
        "cyberplug-bar-widget" => "Companion bar widget for Cyberplug.",
        "cyberterm" => "GPU-rendered terminal emulator.",
        "cybervault" => "Encrypted secrets vault using modern authenticated cryptography.",
        "diagprint" => "Rust diagnostics lifecycle framework.",
        "echo" => "Traffic inspector TUI for WraithFlow captures.",
        "gateflow" => "Kernel-sandbox testing with namespaces, chaos, and veth.",
        "ghostport" => "Encrypted NAT-traversing port forwarder.",
        "keysmith" => "Password, passphrase, and hash generation companion.",
        "sentrygrid" => "Network exposure auditor with host and container correlation.",
        "vortexwall" => "Active-blackholing firewall using nftables.",
        "wraithflow" => "Config-driven TCP proxy and traffic-analysis pipelines.",
        _ => "Cybercore project.",
    }
}

pub fn site(name: &str) -> Option<&'static str> {
    FEATURED_TOOLS
        .iter()
        .find(|(project, _, _)| *project == name)
        .map(|(_, _, url)| *url)
}
