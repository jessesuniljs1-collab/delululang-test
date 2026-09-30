//! PS-E-06 (D-V2-78): `delulu audit export --format ocsf` and `delulu audit verify --ocsf FILE`, end to
//! end. The chain is written through the real `delulu-broker` stack (Broker + AuditLog) into a temp
//! `--dir`, as `audit_cli.rs` does, plus the host's sandbox records; the export is then read, edited and
//! verified through the binary.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::Rc;

use delulu_broker::audit::{AuditEntry, AuditSink as _};
use delulu_broker::{AuditLog, Authority, Broker, Holder, ManualClock, Op, Scopes, SecretStore, SeqIdSource};
use delulu_check::Effect;
use serde_json::Value;

/// The secret's bytes: an export must never carry them.
const CANARY: &str = "hunter2-ocsf-canary-5b1e";

/// Two runs' generations, spelled as the host spells them (PS-E-01: 32 random bytes in hex).
const GEN_A: &str = "4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f4f";
const GEN_B: &str = "a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7";

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu")).args(args).output().expect("failed to run delulu")
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).to_string()
}

fn eff(names: &[&str]) -> BTreeSet<Effect> {
    names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
}

fn names(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_ocsf_cli_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A chain holding every class the export maps to: a root issued, a delegation, an allowed and a refused
/// write, an allowed network use, a secret exposed (its bytes are the canary), a revocation, a use recorded
/// before a use named its effect, a guest's launch and death as the host writes them, and a refused guest —
/// thirteen records.
fn seeded(tag: &str) -> PathBuf {
    let root_dir = scratch(tag);
    let dir = root_dir.join("audit");
    {
        let log = AuditLog::open(&dir).unwrap();
        let clock = Rc::new(ManualClock::new(1_790_000_000_000));
        let mut b = Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock)).with_sink(Box::new(log));
        let root = b
            .issue_root(
                Holder::new("process", "ocsf-test", "pid:0"),
                Authority::new(
                    eff(&["Write", "Declassify", "Net"]),
                    Scopes {
                        fs_write: names(&["./out"]),
                        secrets: names(&["API_KEY"]),
                        net: names(&["example.com"]),
                        ..Default::default()
                    },
                ),
                None,
            )
            .expect("a non-strict broker issues a root");
        let (child, _token) = b
            .delegate(
                &root,
                Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
                Holder::new("process", "agent", "pid:1"),
                None,
                false,
            )
            .expect("a narrower delegation");
        assert!(b.check(&child, Op::FsWrite, Some("./out/a.txt")).is_allow());
        assert!(!b.check(&child, Op::FsWrite, Some("./elsewhere")).is_allow());
        assert!(b.check(&root, Op::Net, Some("example.com")).is_allow());
        let store = SecretStore::in_memory();
        store.set("API_KEY", CANARY).unwrap();
        assert_eq!(b.expose(&root, "API_KEY", &store, Some("app.delulu:3:9".into())).unwrap(), CANARY);
        b.revoke(&root, &child).unwrap();
    }
    // The host's own records, appended the way `guest.rs` appends them: a new writer continuing the seq.
    let mut log = AuditLog::open(&dir).unwrap();
    let mut seq = delulu_broker::tail(&dir, 1).unwrap().last().unwrap().seq;
    // A use as the broker recorded it before D-V2-80: no payload, so no effect named.
    seq += 1;
    log.append(AuditEntry {
        seq,
        ts: 1_790_000_050_000,
        actor_node: Some("g_legacy".into()),
        action: "use".into(),
        target: Some("./out/old.txt".into()),
        authority: None,
        span: None,
        decision: "allow".into(),
    })
    .unwrap();
    let mut add = |action: &str, decision: &str, target: &str, authority: Value| {
        seq += 1;
        log.append(AuditEntry {
            seq,
            ts: 1_790_000_100_000 + seq as i64,
            actor_node: None,
            action: action.into(),
            target: Some(target.into()),
            authority: Some(authority),
            span: None,
            decision: decision.into(),
        })
        .unwrap();
    };
    add("sandbox-launch", "allow", "a1b2c3", serde_json::json!({ "generation": GEN_A, "level": 1 }));
    add("sandbox-death", "allow", "exit 0", serde_json::json!({ "generation": GEN_A, "confirmed": true }));
    add("sandbox-launch", "allow", "d4e5f6", serde_json::json!({ "generation": GEN_B, "level": 1 }));
    add("channel-violation", "deny", "2 refusal(s) on the channel", serde_json::json!({ "denied_total": 2 }));
    add("sandbox-death", "deny", "refused (DL1408): the boundary", serde_json::json!({ "generation": GEN_B }));
    dir
}

fn export(dir: &Path, extra: &[&str]) -> (Output, PathBuf) {
    let out = dir.parent().unwrap().join(format!("export-{}.jsonl", extra.len()));
    let d = dir.to_string_lossy().to_string();
    let o = out.to_string_lossy().to_string();
    let mut args = vec!["audit", "export", "--format", "ocsf", "--dir", &d, "--out", &o, "--device-name", "robot-7"];
    args.extend_from_slice(extra);
    (delulu(&args), out)
}

/// `ocsf.yml` validates the witnesses' exports against the published schema, which the tests cannot reach:
/// with `DELULU_OCSF_CORPUS_OUT` set, a test leaves a copy of its export there, under its own suffix.
fn keep_corpus(export: &Path, suffix: &str) {
    if let Ok(dir) = std::env::var("DELULU_OCSF_CORPUS_OUT") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(export, Path::new(&dir).join(format!("{suffix}.jsonl"))).unwrap();
    }
}

fn lines(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn verify_ocsf(path: &Path, extra: &[&str]) -> Output {
    let p = path.to_string_lossy().to_string();
    let mut args = vec!["audit", "verify", "--ocsf", &p];
    args.extend_from_slice(extra);
    delulu(&args)
}

fn rewrite(path: &Path, f: impl Fn(Vec<String>) -> Vec<String>) {
    let kept: Vec<String> = std::fs::read_to_string(path).unwrap().lines().map(str::to_string).collect();
    std::fs::write(path, f(kept).join("\n") + "\n").unwrap();
}

#[test]
fn every_record_becomes_one_event_of_its_class_and_the_export_verifies_to_the_chains_head() {
    let dir = seeded("whole");
    let (o, file) = export(&dir, &[]);
    assert_eq!(o.status.code(), Some(0), "export failed: {}", text(&o.stderr));
    keep_corpus(&file, "corpus");
    let events = lines(&file);
    let chain = delulu_broker::verify(&dir).expect("the seeded chain verifies");
    assert_eq!(events.len(), chain.records, "one event per record");

    let class: BTreeMap<String, u64> = events
        .iter()
        .map(|e| {
            let r = &e["unmapped"]["delulu"];
            let op = r["authority"]["op"].as_str().map(|o| format!("/{o}")).unwrap_or_default();
            (format!("{}/{}{op}", r["action"].as_str().unwrap(), r["decision"].as_str().unwrap()), e["class_uid"].as_u64().unwrap())
        })
        .collect();
    let want: BTreeMap<String, u64> = [
        ("issue/allow", 3005),
        ("delegate/allow", 3005),
        ("use/allow/FsWrite", 1001),
        ("use/deny/FsWrite", 2004),
        ("use/allow/Net", 4002),
        ("use/allow", 0),
        ("expose/allow", 0),
        ("revoke/allow", 3005),
        ("sandbox-launch/allow", 1007),
        ("sandbox-death/allow", 1007),
        ("channel-violation/deny", 2004),
        ("sandbox-death/deny", 1007),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    assert_eq!(class, want, "each record maps to the class the table names");

    for e in &events {
        let class_uid = e["class_uid"].as_u64().unwrap();
        let activity = e["activity_id"].as_u64().unwrap();
        assert_eq!(e["type_uid"].as_u64().unwrap(), class_uid * 100 + activity, "type_uid is class_uid*100 + activity_id");
        assert_eq!(e["category_uid"].as_u64().unwrap(), class_uid / 1000, "the category is the class's thousands");
        assert_eq!(e["metadata"]["version"], "1.8.0");
        assert_eq!(e["metadata"]["uid"], e["unmapped"]["delulu"]["hash"], "the event's uid is its record's hash");
        assert_eq!(e["metadata"]["sequence"], e["unmapped"]["delulu"]["seq"]);
        assert_eq!(e["time"], e["unmapped"]["delulu"]["ts"]);
        assert_eq!(e["device"]["hostname"], "robot-7");
        for key in ["activity_id", "category_uid", "class_uid", "metadata", "severity_id", "time", "type_uid"] {
            assert!(e.get(key).is_some(), "every event carries the base event's required `{key}`: {e}");
        }
    }
    let write = events.iter().find(|e| e["class_uid"] == 1001).unwrap();
    assert_eq!((write["activity_id"].as_u64(), write["file"]["path"].as_str()), (Some(3), Some("./out/a.txt")), "a write updates the path it was decided on");
    assert_eq!(write["file"]["name"], "a.txt");
    let net = events.iter().find(|e| e["class_uid"] == 4002).unwrap();
    assert_eq!(net["dst_endpoint"]["hostname"], "example.com", "a network use names its host");
    assert_eq!(net["http_request"]["http_method"], "GET");
    let refused = events.iter().find(|e| e["class_uid"] == 2004 && e["unmapped"]["delulu"]["action"] == "use").unwrap();
    assert_eq!(refused["finding_info"]["types"], serde_json::json!(["use", "FsWrite"]), "a refused use's finding names its effect");
    let revoke = events.iter().find(|e| e["unmapped"]["delulu"]["action"] == "revoke").unwrap();
    assert_eq!(revoke["activity_id"], 2, "a revocation revokes privileges");
    let delegation = events.iter().find(|e| e["unmapped"]["delulu"]["action"] == "delegate").unwrap();
    assert_eq!(delegation["privileges"], serde_json::json!(["Write"]), "a delegation's privileges are its effects");
    assert_eq!(delegation["user"]["uid"], delegation["unmapped"]["delulu"]["target"], "the child is the subject");
    let death = events.iter().find(|e| e["unmapped"]["delulu"]["action"] == "sandbox-death").unwrap();
    assert_eq!(death["process"]["uid"], GEN_A, "a guest is named by its run's generation");
    let launch = events.iter().find(|e| e["unmapped"]["delulu"]["action"] == "sandbox-launch").unwrap();
    assert_eq!(launch["process"]["uid"], death["process"]["uid"], "so its launch and its death pair");

    let v = verify_ocsf(&file, &["--json"]);
    assert_eq!(v.status.code(), Some(0), "the export verifies: {}", text(&v.stderr));
    let report: Value = serde_json::from_str(&text(&v.stdout)).unwrap();
    assert_eq!(report["head"].as_str().unwrap(), chain.head, "the export's head is the chain's head");
    assert_eq!(report["events"].as_u64().unwrap() as usize, chain.records);
    assert_eq!(report["from_genesis"], true);
    assert_eq!(report["device"], "robot-7");
}

#[test]
fn a_removed_event_is_detected_from_the_export_alone() {
    let dir = seeded("removed");
    let (o, file) = export(&dir, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    // The refused use, from the middle — the event a person editing the evidence would remove first.
    rewrite(&file, |ls| ls.into_iter().filter(|l| !l.contains("\"use deny")).collect());
    let v = verify_ocsf(&file, &[]);
    assert_eq!(v.status.code(), Some(1), "a removed event must fail: {}", text(&v.stderr));
    assert!(text(&v.stderr).contains("DL1405"), "{}", text(&v.stderr));

    // The first event, from an export that must begin at the chain's beginning.
    let (_, file) = export(&dir, &[]);
    rewrite(&file, |ls| ls.into_iter().skip(1).collect());
    assert_eq!(verify_ocsf(&file, &[]).status.code(), Some(0), "without a start named, a missing first event is not visible");
    let v = verify_ocsf(&file, &["--expect-start", delulu_broker::GENESIS_HASH]);
    assert_eq!(v.status.code(), Some(1), "named, it is: {}", text(&v.stderr));
}

#[test]
fn an_edited_event_or_record_is_detected() {
    let dir = seeded("edited");
    let (o, file) = export(&dir, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let pristine = std::fs::read_to_string(&file).unwrap();

    // An OCSF field only — the record untouched: the refusal reported as informational.
    let edited = pristine.replacen("\"severity\":\"Medium\",\"severity_id\":3", "\"severity\":\"Informational\",\"severity_id\":1", 1);
    assert_ne!(edited, pristine, "the edit landed");
    std::fs::write(&file, &edited).unwrap();
    let v = verify_ocsf(&file, &[]);
    assert_eq!(v.status.code(), Some(1), "an edited OCSF field must fail: {}", text(&v.stderr));
    assert!(text(&v.stderr).contains("not the event its record maps to"), "{}", text(&v.stderr));

    // The record and its event both, consistently: a refusal turned into an allow. The chain catches it.
    let mut events: Vec<Value> = pristine.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let i = events.iter().position(|e| e["unmapped"]["delulu"]["decision"] == "deny").unwrap();
    let mut rec = events[i]["unmapped"]["delulu"].clone();
    rec["decision"] = "allow".into();
    let forged = delulu_broker::ocsf::event(
        &delulu_broker::AuditRecord {
            seq: rec["seq"].as_u64().unwrap(),
            ts: rec["ts"].as_i64().unwrap(),
            prev_hash: rec["prev_hash"].as_str().unwrap().into(),
            hash: rec["hash"].as_str().unwrap().into(),
            actor_node: rec["actor_node"].as_str().map(str::to_string),
            action: rec["action"].as_str().unwrap().into(),
            target: rec["target"].as_str().map(str::to_string),
            authority: rec.get("authority").cloned(),
            span: rec["span"].as_str().map(str::to_string),
            decision: "allow".into(),
        },
        &delulu_broker::ocsf::Labels { product_version: env!("CARGO_PKG_VERSION").into(), device: "robot-7".into() },
    );
    events[i] = forged;
    let body: Vec<String> = events.iter().map(delulu_broker::canonical_json).collect();
    std::fs::write(&file, body.join("\n") + "\n").unwrap();
    let v = verify_ocsf(&file, &[]);
    assert_eq!(v.status.code(), Some(1), "a forged record must fail: {}", text(&v.stderr));
    assert!(text(&v.stderr).contains("tampered"), "{}", text(&v.stderr));

    // One line relabelled to another machine.
    std::fs::write(&file, pristine.replacen("\"hostname\":\"robot-7\"", "\"hostname\":\"robot-8\"", 1)).unwrap();
    let v = verify_ocsf(&file, &[]);
    assert_eq!(v.status.code(), Some(1), "a relabelled line must fail: {}", text(&v.stderr));
}

#[test]
fn the_export_never_carries_a_secrets_bytes() {
    let dir = seeded("secret");
    let (o, file) = export(&dir, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let all = std::fs::read_to_string(&file).unwrap();
    assert!(all.contains("API_KEY"), "the exposure is exported, by the secret's name");
    assert!(!all.contains(CANARY), "the secret's bytes are in the export");
    // Nor on stdout, with no file named.
    let d = dir.to_string_lossy().to_string();
    let o = delulu(&["audit", "export", "--format", "ocsf", "--dir", &d, "--device-name", "robot-7"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(text(&o.stdout).contains("API_KEY") && !text(&o.stdout).contains(CANARY));
}

#[test]
fn since_exports_the_rest_of_the_chain_and_it_verifies_from_the_head_before_it() {
    let dir = seeded("since");
    let records = delulu_broker::tail(&dir, 100).unwrap();
    let cut = &records[5];
    let (o, file) = export(&dir, &["--since", &cut.seq.to_string()]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let events = lines(&file);
    assert_eq!(events.len(), records.len() - 5);
    assert_eq!(events[0]["metadata"]["sequence"].as_u64(), Some(cut.seq));
    let before = &records[4].hash;
    assert_eq!(verify_ocsf(&file, &["--expect-start", before]).status.code(), Some(0), "it chains onto the head before it");
    let v = verify_ocsf(&file, &["--expect-start", delulu_broker::GENESIS_HASH]);
    assert_eq!(v.status.code(), Some(1), "and onto nothing else: {}", text(&v.stderr));
}

#[test]
fn a_chain_that_does_not_verify_is_not_exported_and_the_flags_are_checked() {
    let dir = seeded("broken");
    let day = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .unwrap();
    let t = std::fs::read_to_string(&day).unwrap();
    std::fs::write(&day, t.replacen("\"decision\":\"deny\"", "\"decision\":\"allow\"", 1)).unwrap();
    let (o, file) = export(&dir, &[]);
    assert_eq!(o.status.code(), Some(1), "a broken chain is not exported: {}", text(&o.stderr));
    assert!(!file.exists(), "nothing was written");

    let d = dir.to_string_lossy().to_string();
    assert_eq!(delulu(&["audit", "export", "--dir", &d]).status.code(), Some(2), "a format must be named");
    assert_eq!(delulu(&["audit", "export", "--format", "cef", "--dir", &d]).status.code(), Some(2), "one format");
    assert_eq!(delulu(&["audit", "export", "--format", "ocsf", "--since", "x", "--dir", &d]).status.code(), Some(2));
    assert_eq!(delulu(&["audit", "tail", "--format", "ocsf", "--dir", &d]).status.code(), Some(2), "export's flags only");
    assert_eq!(delulu(&["audit", "tail", "--ocsf", "f.jsonl"]).status.code(), Some(2), "verify's flag only");
}

/// Not a fixture: a real sandboxed run, twice — a refused read and a clean exit — through the host's own
/// records, exported. The fixture above spelled the generation as the host does only because this run
/// showed how (a fake peer keeps the protocol it fakes, `HANDOFF.md` §11.5).
#[test]
fn a_real_sandboxed_runs_records_export_and_verify_its_launch_and_death_paired() {
    let d = scratch("real");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&d)
            .env("DELULU_STATE_DIR", d.join("s"))
            .env("DELULU_HOME", d.join("home"))
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .args(args)
            .output()
            .expect("the binary runs")
    };
    std::fs::create_dir_all(d.join("s").join("audit")).unwrap();
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n")
        .unwrap();
    std::fs::write(
        d.join("r.delulu"),
        "module r\n\nfn main(root: Root) ! {Read} {\n    let fs = root.fs_read(\"./elsewhere\")\n    let _ = fs.read_text(\"./elsewhere/x\")\n}\n",
    )
    .unwrap();
    let refused = run(&["run", "r.delulu", "--sandbox", "--grant", "fs.read=./granted"]);
    assert_ne!(refused.status.code(), Some(0), "the ungranted read is refused: {}", text(&refused.stderr));
    let ok = run(&["run", "h.delulu", "--sandbox", "--grant", "console"]);
    assert_eq!(ok.status.code(), Some(0), "{}", text(&ok.stderr));

    let out = d.join("real.jsonl");
    let e = run(&["audit", "export", "--format", "ocsf", "--out", out.to_str().unwrap(), "--device-name", "lab"]);
    assert_eq!(e.status.code(), Some(0), "{}", text(&e.stderr));
    keep_corpus(&out, "real-run");
    let events = lines(&out);
    let of = |action: &str| -> Vec<&Value> {
        events.iter().filter(|e| e["unmapped"]["delulu"]["action"] == action).collect()
    };
    let (launches, deaths) = (of("sandbox-launch"), of("sandbox-death"));
    assert_eq!((launches.len(), deaths.len()), (2, 2), "two runs, each launched and ended: {events:?}");
    for (l, dth) in launches.iter().zip(&deaths) {
        assert_eq!((l["class_uid"].as_u64(), l["activity_id"].as_u64()), (Some(1007), Some(1)));
        assert_eq!((dth["class_uid"].as_u64(), dth["activity_id"].as_u64()), (Some(1007), Some(2)));
        let g = l["process"]["uid"].as_str().unwrap();
        assert_eq!(g.len(), 64, "the run's generation names the guest: {g}");
        assert_eq!(dth["process"]["uid"].as_str(), Some(g), "a launch and its death pair by it");
    }
    assert_eq!(deaths[0]["status_id"], 2, "the refused run ended in failure");
    assert_eq!(deaths[1]["status_id"], 1, "the clean one in success");
    let violations = of("channel-violation");
    assert!(!violations.is_empty(), "the refused read reached the chain: {events:?}");
    assert!(violations.iter().all(|v| v["class_uid"] == 2004), "a refusal on the channel is a finding");
    let v = verify_ocsf(&out, &["--expect-start", delulu_broker::GENESIS_HASH]);
    assert_eq!(v.status.code(), Some(0), "{}", text(&v.stderr));

    // With no `--device-name`, the machine names itself — per OS (`gethostname`, `COMPUTERNAME`).
    let named = d.join("named.jsonl");
    let e = run(&["audit", "export", "--format", "ocsf", "--out", named.to_str().unwrap()]);
    assert_eq!(e.status.code(), Some(0), "this machine's name is known: {}", text(&e.stderr));
    let host = lines(&named)[0]["device"]["hostname"].as_str().unwrap_or_default().to_string();
    assert!(!host.trim().is_empty(), "the device is named");
    assert!(lines(&named).iter().all(|e| e["device"]["hostname"] == host.as_str()), "the same name on every line");
    assert_eq!(verify_ocsf(&named, &[]).status.code(), Some(0));
}
