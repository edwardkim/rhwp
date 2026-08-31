# Task M100 #4969 W10-Q3-E4 — 최신 devel 병합 재자격화 결과

- **결과 상태**: `qualified-merge-reconfirmed`, 메인테이너 승인 완료
- **checkpoint 상태**: 생성 승인·이 문서와 기계 증적을 같은 commit으로 고정
- **실행일**: 2026-08-30 KST
- **Q3-E4 checkpoint**: `880862177b4b8b430f3aab3ffb56702e90881310`
- **병합 upstream**: `965c050f0b691fabd385926e3574101a8eade158`
- **merge commit**: `a457f289e`
- **현재 동기화**: `31 ahead / 0 behind`
- **기계 판독 증적**:
  [`w10_q3_e4_merge_requalification.json`](../tech/investigations/issue-4969/w10_q3_e4_merge_requalification.json)

## 판정

최신 devel 병합 뒤에도 Q3-E4는 **qualified-merge-reconfirmed**다. 원격 6 commit의 실제 변경은 회전 저장 비트,
CLI 진단, MCP 가이드와 review 문서 17개 파일이다. E4 제품·테스트 파일과 교집합은 0이었고 양측 공통 변경은
`mydocs/orders/20260829.md` 한 파일뿐이었다. `ort`가 양쪽 추가 기록을 보존해 충돌 없이 merge했다.

atomic `GlyphRun`·`GlyphOutline` 게시, CanvasKit outline 선택, 미지원 backend 및 malformed/incomplete outline의
`TextRun` fallback, explicit default의 no-request 출력 동일성은 병합 head에서 모두 유지된다.

## 검증 결과

- Q3-E4 composition handoff: **5 passed / 0 failed**
- Q3-E4 glyph lowering·backend fallback: **17 passed / 0 failed**
- Q3-E4 atomic product activation: **7 passed / 0 failed**
- Q3-E red filter: **4 passed / 0 failed**
- 원격 #6373 회전 저장 교차 영향: **2 passed / 0 failed**
- `cargo check --tests`: 통과
- Rust unit-tier: 4,221 tests / 299 modules, 통과
- integration 파생 상태: 1,032 sources / 4,573 static test attrs / 32 suites + 16 exceptions, prepare·check 통과
- Studio 전체: **1,247 passed / 0 failed / 1 skipped**
- Studio production build: 239 modules, 통과
- all-target Clippy `-D warnings`: 통과
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과
- Docker WASM release + `wasm-opt`: 397.77초 wall time, 통과

## 결정성·크기 영수증

default 출력은 병합 전과 정확히 같다.

- layer JSON: 619,562 bytes,
  BLAKE3 `0b5212cc076c34dce706039d7c4da85936c0a6769e83f08baa7f158cbd9029de`
- CanvasKit plan: 1,755 bytes,
  BLAKE3 `a98e6933d76901cfa55a43205c758675a8864527f87f8bed7a689d16267cbb56`
- Title instance: `21.0px -> 19.840000000000003px`, GlyphRun 1, GlyphOutline 1, CanvasKit outline 선택

병합 후 `pkg/rhwp_bg.wasm`은 9,796,088 bytes,
SHA-256 `f149f84321ae6473daa321adeb4ed2a1f89b258ab71ff49b77ce6efe9b293de5`다. 병합 전 E4보다 25 bytes
(-0.000255%) 작다. 최종 hard gate 판정은 Q3-E5에서 수행한다.

## 실행 절차 정정

원격 신규 `tests/cases/issue_6373_picture_rotation_storage_bits.rs`를 처음에는 독립 Cargo target으로 호출해
`no test target`을 받았다. 공식 `run-rust-test.mjs`로 전환했지만 merge 전 generated suite가 stale여서 첫 라우터
호출은 0건이었다. 메인테이너 worktree 절차대로 manifest `--prepare`·`--check`를 실행한 뒤 같은 라우터가
`regression_suite_026`에서 2건을 실행해 모두 통과했다. generated suite·manifest는 검증 증적일 뿐 tracked diff나
stage에 남기지 않았다. 두 실패는 제품·회귀 실패가 아니라 호출·파생 상태 오류로 분리한다.

## 다음 승인 경계

재자격화 결과와 증적 checkpoint 생성을 승인받았다. 다음 경계는 Q3-E5 성능·크기·전체 회귀·실제 CanvasKit E2E
착수 승인이다. remote push, PR, GitHub comment는 자동 승인되지 않는다.
