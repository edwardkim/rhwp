#!/bin/bash
set -euo pipefail
rhwp_audit_dir=output/pr-review/pr7518-20261004/test-oracle-audit-20261005
rhwp_audit_base=731de9e1b4bb946d76f35108ed7e186ebe4ebecb
rhwp_audit_target=/home/edward/mygithub/rhwp/target/pr-review
step() {
  local name="$1"
  shift
  "$@" > "$rhwp_audit_dir/$name.log" 2>&1
  printf '%s PASS\n' "$name"
}
step prepare-lint node scripts/rust-test-suite-manifest.mjs --prepare
step fmt cargo fmt --all
step prepare-after-fmt node scripts/rust-test-suite-manifest.mjs --prepare
step fmt-check cargo fmt --all -- --check
step clippy-native cargo clippy --locked --target-dir "$rhwp_audit_target" -- -D warnings
step clippy-wasm cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir "$rhwp_audit_target" -- -D warnings
step build-workspace cargo build --locked --workspace --target-dir "$rhwp_audit_target"
step clippy-all-targets cargo clippy --locked --workspace --all-targets --target-dir "$rhwp_audit_target" -- -D warnings
step manifest-check node scripts/rust-test-suite-manifest.mjs --check --base-ref "$rhwp_audit_base"
step 2308-final node scripts/run-rust-test.mjs --cargo-test issue_2308_render_normalized_derived_state -- --target-dir "$rhwp_audit_target"
export CARGO_BIN_EXE_rhwp="$PWD/output/pr-review/pr7518-20261004/rhwp-p39-final-2869859ee"
set +e
node scripts/run-rust-test.mjs --cargo-test issue_7518_reflow_row_physical_frame -- --target-dir "$rhwp_audit_target" > "$rhwp_audit_dir/7518-final.log" 2>&1
rhwp_audit_focus_exit=$?
set -e
printf '7518-focused exit=%s (three known failures expected)\n' "$rhwp_audit_focus_exit"
python3 - <<'PY'
from pathlib import Path
p=Path('output/pr-review/pr7518-20261004/test-oracle-audit-20261005')
s=(p/'7518-final.log').read_text()
assert '18 passed; 3 failed;' in s, s[-3000:]
assert 'terminal_physical_tail_is_drawn_after_the_last_content_unit ... ok' in s
assert 'terminal_cell_ownership_rejects_replayed_or_missing_content ... ok' in s
assert '5 passed; 0 failed; 1 ignored;' in (p/'2308-final.log').read_text()
print('Final scoped results verified: 7518 18 PASS/3 known FAIL; 2308 5 PASS/1 existing ignored.')
PY
