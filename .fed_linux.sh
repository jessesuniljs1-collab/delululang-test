set -u
. "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
echo "=== FEDERATION LINUX VERIFY START $(date) ==="
uname -sr; cargo --version || exit 127
SRC=/mnt/d/nelan/DeluluLang; DST=/home/$USER/delulu-fed
rm -rf "$DST"; mkdir -p "$DST"
tar -C "$SRC" --exclude=target --exclude=.git --exclude=.claude -cf - . | tar -C "$DST" -xf -
cd "$DST" || exit 1
echo "-- witnesses present --"
ls crates/delulu-broker/src/cert.rs crates/delulu/src/cert_crypto.rs crates/delulu/tests/federation_cli.rs
grep -c "effective_state_inherited" crates/delulu-broker/src/tree.rs
export CARGO_TARGET_DIR="$DST/target-linux"
echo "=== [L1] cargo test --workspace ==="
cargo test --workspace > /tmp/fl1.txt 2>&1; echo "EXIT=$?"
echo "ok-suites: $(grep -c 'test result: ok' /tmp/fl1.txt)  FAILED: $(grep -c 'test result: FAILED' /tmp/fl1.txt)"
grep -E "^failures:" -A 6 /tmp/fl1.txt | head -25
echo "--- federation suites ---"
grep -E "Running tests/(federation_cli|device_delegation_cli|actuate_cli|estop_cli|satellite_demo)" -A 3 /tmp/fl1.txt | grep -E "Running|test result" | head -12
echo "=== [L2] fmt examples ==="; cargo run -q -p delulu -- fmt --check examples 2>&1 | tail -1
echo "=== [L2b] fmt book ==="; cargo run -q -p delulu -- fmt --check docs/book/samples 2>&1 | tail -1
echo "=== [L3] check-reference ==="; cargo run -q -p delulu-conform -- --check-reference 2>&1 | tail -1
echo "=== [L3b] coverage ==="; cargo run -q -p delulu-conform -- --coverage 2>&1 | tail -1
echo "=== [L4] python-less ==="; cargo check -q -p delulu --no-default-features > /tmp/fl4.txt 2>&1; echo "EXIT=$?"
echo "=== [L5] clippy ==="; cargo clippy --workspace --all-targets > /tmp/fl5.txt 2>&1; echo "EXIT=$?"
echo "clippy: $(grep -c '^warning' /tmp/fl5.txt) warnings / $(grep -c '^error' /tmp/fl5.txt) errors"
echo "=== FEDERATION LINUX VERIFY DONE $(date) ==="
