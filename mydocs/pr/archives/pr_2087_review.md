# PR #2087 리뷰 — DocumentCore Send 복원

- 작성 시각: 2026-07-09 18:15 KST
- PR: https://github.com/edwardkim/rhwp/pull/2087
- 작성자: `physwkim`
- 제목: `fix(layout): cell_units 캐시를 Rc→Arc 로 바꿔 DocumentCore: Send 복원`
- base / head: `devel` / `fix/layout-cell-units-cache-send`
- reviewer assign: `jangster77` 지정 완료
- merge commit: `67925215d235eb55a132d8a1a7932151dfc8c512`
- 처리 경로: 원 코드 PR merge 후 검증 결과만 별도 docs-only PR 로 보존

## 변경 범위

- `src/renderer/layout.rs`
  - `LayoutEngine::cell_units_cache` 를 `Rc<Vec<CellUnit>>` 에서 `Arc<Vec<CellUnit>>` 로 변경.
- `src/renderer/layout/table_layout.rs`
  - `cell_units()` 반환 타입과 cache clone/new 경로를 `Arc` 기준으로 변경.
- `src/document_core/mod.rs`
  - non-wasm target 에서 `assert_send::<DocumentCore>()` 컴파일 타임 단언 추가.

이 PR 은 Task #1949 의 `LayoutEngine` cell unit memoization 캐시가 `Rc` 를 사용하면서 공개 타입
`DocumentCore` 의 `Send` 성질이 깨진 회귀를 복구한다. `CellUnit` 은 numeric/bool/option 계열 값만 담는
구조라 `Arc<Vec<CellUnit>>` 로 공유해도 의미상 출력 변화는 없다.

## PR 주장 검증

- 외부 임시 crate 에서 `assert_send::<rhwp::DocumentCore>()` 를 확인했다.
- `v0.7.15`: 통과.
- `v0.7.16`: 통과.
- `v0.7.17` tag: 통과.
- PR 직전 `devel` (`0098cab6a1d0c517fe94250bf1521fd031506ccf`): 실패.
  - 실패 원인: `Rc<Vec<CellUnit>> cannot be sent between threads safely`.
- PR 기능 커밋 `9f5b4844b24f04b93d80518ff6025cf3d90e0eb9` 체리픽 후: 통과.

따라서 "v0.7.15 까지는 `DocumentCore: Send` 였고, 현재 `devel` 에서 깨졌으며, 이 PR 이 복원한다"는
주장은 맞다.

## 로컬 검증

검토 기준: `devel` 위에 PR 기능 커밋을 체리픽한 임시 브랜치.

- `git diff --check devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `cargo clippy --all-targets -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture`: 통과.
- 외부 path dependency crate 의 `assert_send::<rhwp::DocumentCore>()`: 통과.

## 배포면 영향 검증

PR 변경은 Rust 내부 캐시의 참조 카운터 구현체만 바꾸며, JS/TS 공개 API 와 npm package surface 는 변경하지
않는다. 그래도 `Arc` 가 wasm 및 각 배포면 빌드를 깨지 않는지 별도로 확인했다.

- `wasm-pack build --target web --out-dir pkg`: 통과.
- `rhwp-studio` `npm run build`: 통과.
- `rhwp-chrome` `npm run build`: 통과.
- `rhwp-firefox` `npm run build`: 통과.
- `rhwp-vscode` `npm run compile`: 통과.
- `npm/editor` `npm pack --dry-run`: 통과.
- `scripts/prepare-npm.sh`: 통과.
- `pkg` (`@rhwp/core`) `npm pack --dry-run`: 통과.

빌드 중 Vite chunk size 경고와 canvaskit `fs`/`path` externalized 경고가 보였지만, 기존 계열 경고이며 이번
`Rc` -> `Arc` 변경으로 인한 실패는 확인되지 않았다.

## GitHub CI

최신 PR head 기준 GitHub Actions 최종 상태:

- `Build & Test`: 통과.
- `Build default-feature tests`: 통과.
- `Native Skia tests`: 통과.
- `CodeQL`: 통과.
- `Analyze (rust)`: 통과.
- `Analyze (python)`: 통과.
- `Analyze (javascript-typescript)`: 통과.
- `Canvas visual diff`: 통과.
- `CI preflight`, `CodeQL preflight`, `Render Diff preflight`: 통과.
- `WASM Build`: skipped.

## 렌더 영향 및 시각 검증

변경 파일은 `src/renderer/layout/**` 를 포함하지만, 실제 변경은 `Rc` 를 `Arc` 로 바꾸는 cache ownership 타입
교체다. Cell unit 계산식, pagination 조건, paint/render tree 생성 로직은 바뀌지 않는다.

별도 HWP/HWPX 기준 PDF 또는 MCP 변환을 요구하는 사용자-visible 렌더 변경은 아니며, GitHub `Canvas visual diff`
도 통과했다. 따라서 이 PR 에서는 추가 visual sweep 을 수행하지 않았다.

## 결론

Blocking finding 없음.

PR #2087 은 `DocumentCore: Send` 회귀를 좁은 범위에서 복구하고, 재발 방지용 compile-time assertion 을 추가한다.
WASM, VSCode extension, Chrome/Firefox extension, npm 패키징 경로 모두 로컬 빌드/패키징 기준으로 영향 없음이
확인됐다.

원 코드 PR #2087 은 GitHub Actions 통과 후 merge 완료했다.
