# #8539 4단계 — 커밋 head 검증

- 검증 head: `1543ae62fe07137da39bbaa88aa82884cc084bf2`
- PR base: `upstream/devel` `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`
- 환경: macOS (Darwin 25.6, arm64), Rust 1.93.1, cargo-nextest 0.9.143, Node 22.23.4, Chrome 156
- 위치: 별도 review worktree `../rhwp-8198-rust-review`(detached, head와 같은 SHA), 공용 target `target/pr-review`
- 로그: `output/pr-review/8539/validate/` (ignored)

## 1. Rust 게이트 (CONTRIBUTING 「Rust 검증 worktree 준비와 실행」)

| 단계 | 명령 | 결과 |
|---|---|---|
| 준비 | `node scripts/rust-test-suite-manifest.mjs --prepare` | 통과 |
| 포맷 | `cargo fmt --all -- --check` | 통과 |
| Clippy native | `cargo clippy --locked -- -D warnings` | 통과 |
| Clippy WASM | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| workspace 빌드 | `cargo build --locked --workspace` | 통과 |
| Clippy all-targets | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| manifest | `--check --base-ref 7d06d5c70…` | 통과 |
| manifest 계약 | `node --test scripts/tests/rust-test-suite-manifest.test.mjs` | 통과 |
| focused | `run-rust-test.mjs issue_8539_rowbreak_stored_restart_cut` (release-test) | 3 passed |
| 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` | **12,418 passed**, 0 failed, 45 skipped |
| Native Skia lib | `cargo test --profile release-test --features native-skia --lib` | 통과 |
| Native Skia | `issue_2225_missing_picture_placeholder` | 2 passed |
| Native Skia | `render_p37_direct_pdf_export` | 22 passed |

`src/**`의 `#[cfg(test)]`는 바꾸지 않아 `rust-unit-test-tiers --check`는 비해당이다.
생성 suite·manifest는 review worktree에만 만들었고 커밋하지 않았다.

## 2. 커밋 head의 시각 검증

같은 review worktree·head에서 새로 빌드했다(`output/pr-review/8539/head/`).

- Native: `cargo build --locked --profile release-test --features native-skia --bin rhwp`,
  바이너리 SHA-256 `d259a2b73d00eac26dc42eaaadf79692b648a4d6e4ce168a1979648dea679789`
- fresh WASM: `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`(wasm-pack 0.15.0),
  `rhwp_bg.wasm` `bf751178ac0abfe8247587f43e2f68fa4c056bbb279abcaf40c2e5aec644ad8c`,
  `rhwp.js` `219cdf77a61e995569350fc81c8258f104bd177301a80bfe3a59670ff3804a06`(pkg·Studio public 동일), Chrome 156
- 입력: `samples/issue8539/nhis-dependent-requirements.hwp`, 기준 `nhis-dependent-requirements-2020.pdf`(한/글 2020 Print)
- `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 실루엣 `threshold_boundary_color_support_v1`,
  반경 2px, 색 허용 32. 글꼴은 시스템 NanumMyeongjo.ttc만 담은 폴더를 `RHWP_FONT_PATH`로 공급했다.

| 출력 | 1쪽 | 2쪽 | 3쪽 | raw 미대응 px | 경계 보정 px | 쪽수 |
|---|---|---|---|---|---|---|
| Native | 99.90 | 100.00 | 100.00 | 66 / 1 / 0 | 0 | 3 = 3 |
| fresh WASM | 99.90 | 100.00 | 100.00 | 66 / 1 / 0 | 0 | 3 = 3 |

두 TSV 원문은 같다(SHA-256 `4d1f624eec3811c0c4f00a5718ae87386ee7e1edbdb393ae7eac0b6a0e60ef2f`).
`--silhouette-only` 판정은 `not_evaluated`(90% 미만 없음, 쪽수 일치)이고, 일반 모드 1~3쪽 판정은 Native·WASM 모두 `passed`다.

직접 판독(WASM review 1~3쪽, Native review 2쪽):

- 1쪽: 5번 행 가운데·오른쪽 칸 셋째 줄 「수 또는 소득이 없는 경우 / 경우 부양 인정. 다만, 이」가 PDF와 같은 자리다.
- 2쪽: 첫머리 「부양 인정 / 혼ㆍ사별한 경우 자녀인」, 6번 행은 그 아래에 겹침 없이 있다. 끝 줄 「라. 「국가유공자 / 있어도 보수
  또는 소득이」가 PDF와 같다.
- 3쪽: 「등 예우 및 지원 / 없는 경우 부양 인정」으로 시작해 「받은 사람」으로 끝나고, 표 아래 괘선이 PDF와 같은 높이다.

대표 PNG(1·2쪽 × Native·WASM × review·overlay)를 `mydocs/pr/assets/issue_8539_rowbreak_stored_restart/`에 복사했다.

## 3. 판정

| 항목 | 판정 | 근거 |
|---|---|---|
| 9행·16행 컷 = 한/글 2020 | 충족 | dump-pages `[2,3,3]`·`[10,8,10]`, 회귀 테스트, 직접 판독 |
| 대상 문서 Native·WASM 최저 90% 이상 | 충족 | 최저 99.90 |
| 코퍼스 무회귀 | 충족(컷·쪽수 기준) | 1,424개 중 바뀐 문서는 대상 1개, 쪽수 변화 0 |
| 반례(선언 상자가 넘치는 행·상자 밖 재시작) 유지 | 충족 | 16418295·148776468·issue1937·issue1853·37787·byeolpyo4가 devel과 같은 컷 |
| 블록(rowspan) 컷 `advance_row_block_cut` | 비해당 | 이번 표의 분할 행에 병합 칸 없음. 코퍼스에서 같은 차이는 나오지 않음 |
| HWPX 저장본에 같은 상황 | 미검증 | 같은 저장 형상의 HWPX 실물·한/글 출력이 없다 |
| 한/글 2024 Print | 미검증 | 기준 PDF는 2020 Print뿐이다 |
