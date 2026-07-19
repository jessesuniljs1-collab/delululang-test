//! The DeluluLang measurement program (Stage 9, spec §3) — the published, reproducible evidence
//! for the identity claim.
//!
//! Three studies:
//! - **A** (`study_a`): whole-program authority verification at scale. The mechanism claim.
//! - **B** (`study_b`): agent task success and repair loops.
//! - **C** (`study_c`): the performance honesty baseline.
//!
//! Every study is deterministic and reproducible from a clean checkout. Where a study cannot be
//! run in full — Study B's live-model lane needs API keys and network — the harness ships complete
//! and the write-up labels that lane UNRUN rather than quietly reporting the half that did run as
//! though it were the whole.
//!
//! Constitution §9's honesty clauses bind every sentence these modules emit. In practice that
//! means: no number appears in prose that is not in `results.json`, an empty experiment is a
//! failure rather than a vacuous success, and the threats to validity are part of the deliverable
//! rather than an appendix nobody links to.

pub mod corpus;
pub mod study_a;
pub mod study_b;
pub mod study_c;
