//! The Stage-1 capability broker (spec §7.2). In Stage 1 the CLI *is* the human-controlled
//! broker: root capabilities are constructed here from grants the human passes, and `main`
//! receives only that slice. Stage 5 replaces this with a process-separated broker; the grant
//! model and manifest format do not change.

use std::collections::HashMap;

use delulu_check::{Effect, ResourceKind, ScopeInfo};
use delulu_diag::Diagnostic;

use crate::prim::granted_root;
use crate::value::RootVal;

/// A parsed authority manifest (`delulu.toml`, `[authority]` section). Deliberately minimal for
/// Stage 1 — enough to run the DL0701 check and drive the grant reconciliation.
#[derive(Default, Debug)]
pub struct Manifest {
    pub name: Option<String>,
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
    /// `foreign.c` logical lib names the manifest permits (Stage 4, spec §4.1). The manifest names
    /// *what* the program may reach for; the human/broker's `--grant` supplies *which binary*.
    pub foreign_c: Vec<String>,
    /// `foreign.python` import allowlist patterns the manifest permits (Stage 4, spec §5.1): exact
    /// module names (`"numpy"`) or `prefix.*` wildcards (`"numpy.*"`).
    pub foreign_python: Vec<String>,
    /// Stage 10 (invariant 46): the package DECLARES a native-emission request
    /// (`[authority] exec.native = true`). Declaration is reviewable intent, never permission —
    /// and `--grant-manifest` deliberately does NOT confer it (build-order D6): the red-tier
    /// grant must be named explicitly at the prompt, like a secret must exist in the env.
    pub exec_native: bool,
    /// Stage 10 (10c): `[actors] mailbox = N` — the package-default mailbox bound.
    pub actors_mailbox: Option<u64>,
    /// Stage 10 (10c): `[actors] overflow = "block" | "drop-new"` — the overflow policy for
    /// bounded mailboxes (`block` when absent, the spec default).
    pub actors_overflow: Option<String>,
    /// P2 (D-V2-27): `[plugins] allow = ["<blake3-hex>", ...]` — the artifacts this package may load
    /// at run time, named by the hash of their bytes.
    ///
    /// A CEILING, not a source of authority. It says which artifacts may load; it never says what they
    /// may do, which stays the artifact's own declared ceiling intersected with the operator's grant.
    /// The same division `foreign.c` already uses: the manifest names *what*, the operator supplies
    /// *which*. Empty means the package pins nothing, and then only the operator's `--grant plugin=`
    /// decides — pinning is the honest form and is not therefore mandatory.
    pub plugins_allow: Vec<String>,
}

/// Parse the tiny subset of TOML the Stage-1 manifest uses: `[section]` headers and
/// `key = "s"` / `key = ["a", "b"]` lines. Dotted keys (`fs.read`) are supported literally.
pub fn parse_manifest(src: &str) -> Manifest {
    let mut m = Manifest::default();
    let mut section = String::new();
    for raw in src.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = inner.trim().to_string();
            continue;
        }
        let Some((key, val)) = line.split_once('=') else { continue };
        let key = key.trim();
        let val = val.trim();
        let values = parse_toml_strings(val);
        match (section.as_str(), key) {
            ("package", "name") => m.name = values.into_iter().next(),
            ("authority", "effects") => m.effects = values,
            ("authority", "fs.read") => m.fs_read = values,
            ("authority", "fs.write") => m.fs_write = values,
            ("authority", "net") => m.net = values,
            ("authority", "secrets") => m.secrets = values,
            ("authority", "foreign.c") => m.foreign_c = values,
            ("authority", "foreign.python") => m.foreign_python = values,
            // Stage 10 (invariant 46): `exec.native = true` is how a package DECLARES that it
            // requests native-code emission. Declaring is not getting — the human grant
            // (`--grant exec.native`) is a separate decision, and both default to off.
            ("authority", "exec.native") => m.exec_native = val == "true",
            // Stage 10 (10c, spec §3): package-wide mailbox defaults. A per-actor
            // `(mailbox = N)` on the declaration wins over this; nothing = unbounded (1.0).
            ("actors", "mailbox") => m.actors_mailbox = val.parse::<u64>().ok(),
            ("actors", "overflow") => m.actors_overflow = values.into_iter().next(),
            // P2 (D-V2-27): the hash ceiling. Lower-cased on the way in, because a hash is compared
            // to decide whether code may load, and two spellings of one hash deciding differently is
            // the campaign's recurring shape (a security decision on an unnormalized representation).
            ("plugins", "allow") => {
                m.plugins_allow = values.into_iter().map(|h| h.trim().to_ascii_lowercase()).collect()
            }
            _ => {}
        }
    }
    m
}

fn parse_toml_strings(val: &str) -> Vec<String> {
    let body = val.trim();
    let inner = body.strip_prefix('[').and_then(|s| s.strip_suffix(']')).unwrap_or(body);
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl Manifest {
    /// A `ScopeInfo` for the authority report, from the manifest's declared scopes.
    pub fn scope_info(&self) -> ScopeInfo {
        ScopeInfo {
            fs_read: self.fs_read.clone(),
            fs_write: self.fs_write.clone(),
            net: self.net.clone(),
            ..Default::default()
        }
    }

    /// DL0701: `main`'s effect row must be within the manifest's declared effects.
    pub fn check_main_row(&self, main_row: &std::collections::BTreeSet<Effect>) -> Vec<Diagnostic> {
        let declared: std::collections::HashSet<&str> = self.effects.iter().map(|s| s.as_str()).collect();
        let mut diags = Vec::new();
        for e in main_row {
            if !declared.contains(e.name()) {
                diags.push(Diagnostic::error(
                    "DL0701",
                    format!("`main` performs effect `{}` not permitted by the authority manifest", e.name()),
                ));
            }
        }
        diags
    }
}

/// The set of grants collected from `--grant` flags (and/or an accepted manifest).
#[derive(Default)]
pub struct Grants {
    pub console: bool,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub clock: bool,
    pub rand: bool,
    pub declassify: bool,
    pub secrets: HashMap<String, String>,
    /// `foreign.c` grants: logical lib name → the concrete binary path/name the human chose
    /// (Stage 4, spec §4.1). The path is grant data, never program data.
    pub foreign_c: HashMap<String, String>,
    /// `foreign.python` grants: the granted import allowlist patterns (Stage 4, spec §5.1). Non-empty
    /// iff Python is granted; carried into `Cap[Python]`'s scope and checked at `py.import` (DL1305).
    pub foreign_python: Vec<String>,
    /// Stage 10 (invariant 46): permission to EMIT AND RUN native code (`--grant exec.native`).
    /// Off by default, everywhere, forever-until-granted — a human policy decision, never a
    /// program's. v1.x ships no native tier yet; this is the leash built before the animal, and
    /// DL1906 is the honest note that a `@jit` hint was ignored for lack of it.
    pub exec_native: bool,
    /// Stage 10 (10e, Track D): actuator grants — the human constructs the ENVELOPE at the
    /// prompt (`--grant "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40"`), and the
    /// program can never widen it. The envelope IS the scope (spec §5.1).
    pub actuators: Vec<crate::value::ActuatorEnvelope>,
    /// Stage 10 (10e): granted sensor device names (`--grant sensor=arm0/angle`).
    pub sensors: Vec<String>,
    /// Stage 10 (10h, Track F): granted compute devices — the envelope IS the scope (spec §7.1).
    pub computes: Vec<crate::value::ComputeEnvelope>,
    /// P2 (D-V2-27): `--grant plugin=<path-or-dir>` — where a program may load a plugin artifact
    /// FROM. The operator's half of the loading decision; the manifest's `[plugins] allow` hash list
    /// is the other half, and a load needs both when the package pins any.
    ///
    /// Stored as the operator wrote it, refused here if the spelling itself could widen (`..`, a bare
    /// drive letter, the hostile forms `hostile_path` already knows) and resolved against the working
    /// directory at load time by the same containment the primitive table uses for `fs.*`. A grant is
    /// a path spelling too — that is C-11/D-NE-29, and it applies to this dimension for the same
    /// reason it applies to `fs.read`.
    pub plugins: Vec<String>,
}

impl Grants {
    /// Parse one `--grant` argument. Returns an error string on malformed input.
    pub fn add(&mut self, spec: &str) -> Result<(), String> {
        if let Some(rest) = spec.strip_prefix("secret:") {
            let (name, source) = rest.split_once('=').ok_or_else(|| format!("bad secret grant `{spec}` (use secret:NAME=VALUE)"))?;
            let value = if let Some(var) = source.strip_prefix("env:") {
                std::env::var(var).map_err(|_| format!("env var `{var}` for secret `{name}` is not set"))?
            } else {
                source.to_string()
            };
            self.secrets.insert(name.to_string(), value);
            return Ok(());
        }
        match spec.split_once('=') {
            Some((k, v)) => match k.trim() {
                // Campaign finding C87. An EMPTY value is refused here exactly as the five keys
                // below already refuse it. `--grant fs.read=` is not an empty grant: the path is
                // joined onto the working directory, and joining `""` yields the working directory
                // itself — so a `--grant fs.read=$SHARE_DIR` with `SHARE_DIR` unset silently handed
                // over the whole tree the command was run in. An unset variable is the ordinary way
                // this happens, and it is silent in every shell.
                "fs.read" | "fs.write" | "net" | "net.special" => {
                    let val = v.trim();
                    if val.is_empty() {
                        let hint = if k.trim().starts_with("net") { "HOST" } else { "PATH" };
                        return Err(format!(
                            "bad grant `{spec}` — the value is empty (use {}={hint}); an empty path \
                             would grant the whole working directory, so it is refused rather than \
                             guessed",
                            k.trim()
                        ));
                    }
                    // C-11 / D-NE-29: `fs.read=C:` meant "the current directory of drive C:",
                    // silently. A grant is a path spelling too, and is refused on the same rules as a
                    // program's (a `\\server\share` stays the operator's to name).
                    if !k.trim().starts_with("net") {
                        if let Some(why) = crate::prim::hostile_path(val, true) {
                            return Err(format!("bad grant `{spec}` — {why}; refused rather than guessed"));
                        }
                    }
                    // NE-18 / D-NE-28 (owner, 2026-09-18): a special-use address or name is refused under
                    // plain `net=` and granted only by the explicit spelling `net.special=`.
                    if k.trim() == "net" {
                        if let Some(class) = crate::netclass::special_use_class(val) {
                            return Err(format!(
                                "bad grant `{spec}` — `{val}` is a special-use address: {class}. A plain `net=` grant never reaches one; if this program is meant to, grant it explicitly: `--grant net.special={val}`"
                            ));
                        }
                    }
                    match k.trim() {
                        "fs.read" => self.fs_read.push(val.to_string()),
                        "fs.write" => self.fs_write.push(val.to_string()),
                        _ => self.net.push(val.to_string()),
                    }
                }
                // `foreign.c=LOGICAL:PATH` — split on the FIRST colon only, so a Windows path like
                // `mathlib:C:\lib\libm.dll` keeps its drive-letter colon (spec §4.1). A bare name
                // (`mathlib:libm.dll`) resolves via the OS loader rules at bind time.
                "foreign.c" => {
                    let (name, path) = v.trim().split_once(':').ok_or_else(|| {
                        format!("bad foreign grant `{spec}` (use foreign.c=LOGICAL:PATH)")
                    })?;
                    let (name, path) = (name.trim(), path.trim());
                    if name.is_empty() || path.is_empty() {
                        return Err(format!("bad foreign grant `{spec}` (use foreign.c=LOGICAL:PATH)"));
                    }
                    self.foreign_c.insert(name.to_string(), path.to_string());
                }
                // Stage 10 (10e): `actuator=DEVICE:dim=lo..hi[,dim=lo..hi...][,rate_hz=N]` — the
                // human writes the envelope at the prompt; every command is checked against it.
                "actuator" => {
                    let env = crate::value::ActuatorEnvelope::parse(v.trim())
                        .map_err(|e| format!("bad actuator grant `{spec}`: {e}"))?;
                    self.actuators.push(env);
                }
                // Stage 10 (10h): `compute=DEVICE:memory_bytes=N,kernel_ms=lo..hi,...` — the same
                // shape one layer out: the human writes the device envelope, and every dispatch is
                // checked against it. The DL1911 attestation gate runs later, at the pre-flight,
                // because it needs the adapter registry and not just the string.
                "compute" => {
                    let env = crate::value::ComputeEnvelope::parse(v.trim())
                        .map_err(|e| format!("bad compute grant `{spec}`: {e}"))?;
                    self.computes.push(env);
                }
                // Stage 10 (10e): `sensor=DEVICE` — reads are `Read` under this device scope.
                // P2 (D-V2-27): `plugin=<path-or-dir>`. Refused empty for the reason the five
                // scope keys are: joining "" onto the working directory yields the working directory,
                // so `--grant plugin=$PLUGIN_DIR` with the variable unset would grant the whole tree
                // the command ran in — silently, in every shell (campaign finding C87).
                "plugin" => {
                    let val = v.trim();
                    if val.is_empty() {
                        return Err(format!(
                            "bad grant `{spec}` — the value is empty (use plugin=PATH); an empty path \
                             would grant the whole working directory to load code from, so it is \
                             refused rather than guessed"
                        ));
                    }
                    if let Some(why) = crate::prim::hostile_path(val, true) {
                        return Err(format!("bad grant `{spec}` — {why}; refused rather than guessed"));
                    }
                    self.plugins.push(val.to_string());
                }
                "sensor" => {
                    let d = v.trim();
                    if d.is_empty() {
                        return Err(format!("bad sensor grant `{spec}` (use sensor=DEVICE)"));
                    }
                    self.sensors.push(d.to_string());
                }
                // `foreign.python=PATTERN` — an import allowlist pattern (spec §5.1). Repeat the flag
                // to grant several: `--grant foreign.python=numpy --grant "foreign.python=numpy.*"`.
                "foreign.python" => {
                    let pat = v.trim();
                    if pat.is_empty() {
                        return Err(format!("bad python grant `{spec}` (use foreign.python=PATTERN)"));
                    }
                    self.foreign_python.push(pat.to_string());
                }
                other => return Err(format!("unknown grant `{other}`")),
            },
            None => match spec.trim() {
                "console" => self.console = true,
                "clock" => self.clock = true,
                "rand" => self.rand = true,
                "declassify" => self.declassify = true,
                // Invariant 46: native-code emission is a grant like any other — explicit,
                // human-issued, default-off. (No native tier exists in v1.x; granting this today
                // changes nothing but the DL1906 note, and that honesty is deliberate.)
                "exec.native" => self.exec_native = true,
                other => return Err(format!("unknown grant `{other}`")),
            },
        }
        Ok(())
    }

    /// Accept an entire manifest's declared authority as grants (`--grant-manifest`). Secrets
    /// are pulled from environment variables of the same name when available.
    pub fn accept_manifest(&mut self, m: &Manifest) {
        self.console = self.console || m.effects.iter().any(|e| e == "Write" || e == "Read");
        self.fs_read.extend(m.fs_read.iter().cloned());
        self.fs_write.extend(m.fs_write.iter().cloned());
        self.net.extend(m.net.iter().cloned());
        // Accept the manifest's declared `foreign.python` allowlist as grants (`--grant-manifest`).
        self.foreign_python.extend(m.foreign_python.iter().cloned());
        for name in &m.secrets {
            if let Ok(v) = std::env::var(name) {
                self.secrets.insert(name.clone(), v);
            }
        }
    }

    /// Build the root capability slice `main` receives.
    pub fn build_root(&self) -> RootVal {
        RootVal {
            console: self.console,
            fs_read: self.fs_read.iter().map(|p| granted_root(p)).collect(),
            fs_write: self.fs_write.iter().map(|p| granted_root(p)).collect(),
            net: self.net.clone(),
            clock: self.clock,
            rand: self.rand,
            declassify: self.declassify,
            secrets: self.secrets.clone(),
            // `Cap[ForeignLoad]` is available to mint iff at least one foreign lib OR any Python
            // allowlist pattern is granted (both `root.foreign` and `root.python` consume it); the
            // per-lib/per-import authority decision is enforced separately (spec §4.1/§5.1).
            foreign_load: !self.foreign_c.is_empty() || !self.foreign_python.is_empty(),
            python_allowlist: self.foreign_python.clone(),
            // Embedded mode: secrets carry their bytes in `secrets` (above); no broker handles.
            broker_secrets: Vec::new(),
            actuators: self.actuators.clone(),
            sensors: self.sensors.clone(),
            computes: self.computes.clone(),
            // P2 (D-V2-27): resolved the same way every other path grant is, through
            // `granted_root`, so `..` and `.` are gone before the value is ever compared against a
            // program's path. The hash ceiling comes from the manifest and is attached by the caller
            // that read it (`run_cmd`), because `Grants` is the operator's half and the manifest is
            // the package's.
            plugins: self.plugins.iter().map(|p| granted_root(p)).collect(),
            plugins_allow: Vec::new(),
        }
    }

    pub fn scope_info(&self) -> ScopeInfo {
        ScopeInfo {
            fs_read: self.fs_read.clone(),
            fs_write: self.fs_write.clone(),
            net: self.net.clone(),
            ..Default::default()
        }
    }
}

/// Which capability kinds a program needs but the grants do not cover (for the prompt / DL0702).
pub fn missing_kinds(needs: &std::collections::BTreeSet<ResourceKind>, grants: &Grants) -> Vec<ResourceKind> {
    needs
        .iter()
        .filter(|k| match k {
            ResourceKind::Console => !grants.console,
            ResourceKind::FsRead => grants.fs_read.is_empty(),
            ResourceKind::FsWrite => grants.fs_write.is_empty(),
            ResourceKind::Http => grants.net.is_empty(),
            ResourceKind::Clock => !grants.clock,
            ResourceKind::Rand => !grants.rand,
            ResourceKind::Declassify => !grants.declassify,
            // P2 (D-V2-27): covered once the operator names somewhere to load from. It used to be
            // unconditionally `true` — "never covered" — which was correct while no load path
            // existed and would now refuse every legitimate host program before it ran.
            ResourceKind::PluginHost => grants.plugins.is_empty(),
            // `Cap[ForeignLoad]` is covered once any `foreign.c` lib OR `foreign.python` pattern is
            // granted (spec §4.1/§5.1); the per-lib/per-import gate is enforced later.
            ResourceKind::ForeignLoad => grants.foreign_c.is_empty() && grants.foreign_python.is_empty(),
            // `Cap[Python]` is covered once any `foreign.python` pattern is granted (spec §5.1).
            ResourceKind::Python => grants.foreign_python.is_empty(),
            // Stage 10 (10e): physical devices, deny-by-default like everything else.
            ResourceKind::Actuator => grants.actuators.is_empty(),
            ResourceKind::Sensor => grants.sensors.is_empty(),
            // Stage 10 (10h): accelerators, same rule. A program that reaches for silicon it was
            // never granted is refused at the pre-flight, before a line runs.
            ResourceKind::Compute => grants.computes.is_empty(),
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P2 (D-V2-27): the loading grant. A grant is a path spelling too, so every hostile form the
    /// primitive table refuses from a PROGRAM is refused from an OPERATOR here as well — that is
    /// C-11/D-NE-29, and this dimension is the one where getting it wrong means loading code.
    #[test]
    fn the_plugin_grant_refuses_every_spelling_that_could_widen_it() {
        // The empty value first. `--grant plugin=$DIR` with `DIR` unset is silent in every shell, and
        // joining "" onto the working directory yields the working directory — which here would mean
        // "load code from anywhere under where I happened to run this" (campaign finding C87).
        let mut g = Grants::default();
        let e = g.add("plugin=").expect_err("an empty plugin grant must be refused");
        assert!(e.contains("the value is empty"), "{e}");
        assert!(e.contains("load code from"), "the refusal must say what the empty path would mean: {e}");

        // A bare drive letter means "the current directory of that drive", silently (C-11).
        if cfg!(windows) {
            let mut g = Grants::default();
            assert!(g.add("plugin=C:").is_err(), "a bare drive letter must be refused");
        }

        // And the control: an ordinary relative path is accepted, or the assertions above would be
        // passing for a dimension that refuses everything.
        let mut g = Grants::default();
        g.add("plugin=./plugins").expect("an ordinary path is a grant");
        assert_eq!(g.plugins, vec!["./plugins".to_string()]);
        // Repeating the flag adds, as every other scope dimension does.
        g.add("plugin=./more").expect("a second path");
        assert_eq!(g.plugins.len(), 2, "{:?}", g.plugins);
    }

    /// The capability exists only when the operator named somewhere to load from, and `plugin_host`
    /// is no longer the unconditional refusal it was before P2 (NE-01).
    #[test]
    fn plugin_host_is_minted_only_when_loading_was_granted() {
        let mut g = Grants::default();
        let root = g.build_root();
        assert!(root.plugins.is_empty(), "nothing is granted by default");
        assert!(
            crate::broker::missing_kinds(&[ResourceKind::PluginHost].into_iter().collect(), &g)
                .contains(&ResourceKind::PluginHost),
            "an ungranted plugin host must be reported missing at the pre-flight"
        );

        g.add("plugin=./plugins").unwrap();
        let root = g.build_root();
        assert_eq!(root.plugins.len(), 1, "the grant reached the root");
        assert!(root.plugins[0].is_absolute(), "resolved against the working directory: {:?}", root.plugins[0]);
        assert!(
            crate::broker::missing_kinds(&[ResourceKind::PluginHost].into_iter().collect(), &g).is_empty(),
            "a granted plugin host must not be reported missing"
        );
    }

    /// The manifest's `[plugins] allow` is a hash CEILING, and `--grant-manifest` must not turn it
    /// into a grant: accepting a manifest's authority is a different act from obeying its
    /// restrictions, and only one of them is the operator's to opt into.
    #[test]
    fn the_manifest_pins_artifacts_by_hash_and_never_grants_loading() {
        let m = crate::parse_manifest(
            "[package]\nname = \"p\"\n\n[authority]\neffects = [\"Write\"]\n\n[plugins]\nallow = [\"AB12\", \"cd34\"]\n",
        );
        // Lower-cased on the way in: a hash decides whether code may load, and two spellings of one
        // hash deciding differently is the shape this campaign keeps finding.
        assert_eq!(m.plugins_allow, vec!["ab12".to_string(), "cd34".to_string()]);

        let mut g = Grants::default();
        g.accept_manifest(&m);
        assert!(
            g.plugins.is_empty(),
            "`--grant-manifest` must not confer plugin loading — a package cannot grant itself the \
             right to load code, any more than it can grant itself `exec.native`"
        );
    }

    // ----- Stage 4 foreign grants (spec §4.1) -------------------------------

    #[test]
    fn foreign_grant_splits_on_the_first_colon_only() {
        // A Windows path keeps its drive-letter colon: the grant splits LOGICAL from PATH once.
        let mut g = Grants::default();
        g.add(r"foreign.c=mathlib:C:\libs\libm.dll").unwrap();
        assert_eq!(g.foreign_c["mathlib"], r"C:\libs\libm.dll");
        // A bare name resolves via OS loader rules at bind time — also legal grant data.
        g.add("foreign.c=z:libz.so.1").unwrap();
        assert_eq!(g.foreign_c["z"], "libz.so.1");
        // A colon-free absolute unix/macOS path survives intact (spec §4.1's own example, and the
        // shape the Linux/macOS test mirrors grant): the single split leaves the whole path as PATH.
        g.add("foreign.c=mathlib:/usr/lib/libm.so.6").unwrap();
        assert_eq!(g.foreign_c["mathlib"], "/usr/lib/libm.so.6");
        // A macOS `.dylib` bare name, resolved via the dyld shared cache, is likewise untouched.
        g.add("foreign.c=m:libm.dylib").unwrap();
        assert_eq!(g.foreign_c["m"], "libm.dylib");
    }

    #[test]
    fn malformed_foreign_grant_is_an_error_not_a_panic() {
        let mut g = Grants::default();
        assert!(g.add("foreign.c=mathlib").is_err(), "missing path must be rejected");
        assert!(g.add("foreign.c=:only-a-path").is_err(), "missing logical name must be rejected");
        assert!(g.add("foreign.c=name:").is_err(), "empty path must be rejected");
    }

    #[test]
    fn manifest_parses_foreign_c_logical_names() {
        let m = parse_manifest("[authority]\neffects = [\"ForeignCall\"]\nforeign.c = [\"mathlib\", \"z\"]\n");
        assert_eq!(m.foreign_c, vec!["mathlib", "z"]);
        assert_eq!(m.effects, vec!["ForeignCall"]);
    }

    // ----- DL0701: the manifest is a ceiling (Stage 9 release, criterion 1) -------------------

    #[test]
    fn a_main_row_exceeding_the_manifest_is_refused_with_dl0701() {
        // The manifest declares Write only; main also performs Net. The check must produce the
        // DL0701 diagnostic naming the undeclared effect — the manifest is a CEILING the code is
        // checked against, not a claim taken on trust (spec §7.2).
        let m = parse_manifest("[authority]\neffects = [\"Write\"]\n");
        let mut row = std::collections::BTreeSet::new();
        row.insert(Effect::Write);
        row.insert(Effect::Net);
        let diags = m.check_main_row(&row);
        assert_eq!(diags.len(), 1, "exactly one undeclared effect: {diags:?}");
        assert_eq!(diags[0].code, "DL0701");
        assert!(
            diags[0].message.contains("`Net`"),
            "the refusal names the exceeding effect: {}",
            diags[0].message
        );
        // And the honest converse: a row within the ceiling produces nothing.
        let mut ok = std::collections::BTreeSet::new();
        ok.insert(Effect::Write);
        assert!(m.check_main_row(&ok).is_empty(), "a row within the manifest is clean");
    }

    #[test]
    fn foreign_load_root_slice_follows_the_grant() {
        let mut needs = std::collections::BTreeSet::new();
        needs.insert(ResourceKind::ForeignLoad);

        let ungranted = Grants::default();
        assert!(!ungranted.build_root().foreign_load, "no grant, no Cap[ForeignLoad]");
        assert_eq!(missing_kinds(&needs, &ungranted), vec![ResourceKind::ForeignLoad]);

        let mut granted = Grants::default();
        granted.add("foreign.c=mathlib:libm.dll").unwrap();
        assert!(granted.build_root().foreign_load);
        assert!(missing_kinds(&needs, &granted).is_empty());
    }
}
