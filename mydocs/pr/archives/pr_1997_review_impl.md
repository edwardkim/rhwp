# PR #1997 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1997
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1993
- 작성자: planet6897
- 문서 작성 시점 head SHA: `b78e6facdf8dc171023f2203863ed8a195626c10`
- 실제 변경 commit: `4e33872c7aad5bb2df7101efcefd657d53d74797`
- 체리픽 commit: `7ebb22152`

## Stage 1. 메타 확인 및 체리픽

완료.

- PR base가 `devel`이고 draft가 아님을 확인했다.
- reviewer `jangster77`를 assign했다.
- PR head 기준 mergeable 참고값은 `MERGEABLE`이다.
- 기능 변경 commit `4e33872c7aad5bb2df7101efcefd657d53d74797`를 `upstream/devel` 기준 로컬 브랜치 `pr1997-review`에 체리픽했다.
- PR head의 `b78e6facdf8dc171023f2203863ed8a195626c10`는 `devel` merge commit이라 체리픽 검토에서는 제외했다.

## Stage 2. 변경 내용 검토

완료.

- `line_run_coverage`는 `TextLine` 하위 `TextRun`들의 x 구간 합집합을 만들어 run 재분할 노이즈를 제거한다.
- `flatten_pos_set`은 구조 노드의 타입과 bbox, `TextLine` coverage를 정렬된 멀티셋으로 만들어 순서 차이에 둔감한 비교 기준을 제공한다.
- `diff_page`는 이 멀티셋이 같을 때 zero-diff로 단락한다.
- 신규 테스트는 재분할, 형제 줄 순서 스왑, 실제 coverage 변화 3가지 경계를 확인한다.
- geometry 진단의 범위상 텍스트 내용/스타일/픽셀 동일성은 이 PR의 판정 대상이 아니며, 이 점을 리뷰 문서에 잔여 해석 주의로 남겼다.

## Stage 3. 시각 검증 판정

완료.

- 변경 파일은 `src/diagnostics/render_geom_diff.rs` 단일 파일이다.
- 렌더러 출력, WASM canvas, PDF/SVG 생성 경로를 바꾸지 않는다.
- PR 목적은 render tree 비교 진단의 허위 변위 제거이므로 visual sweep 대신 diagnostics 테스트와 baseline/통합 테스트를 검증 근거로 삼았다.

## Stage 4. 로컬 검증

완료.

- `/Users/tsjang/rhwp/target` 하위 항목 삭제 후 진행했다.
- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib diagnostics::render_geom_diff`: 16 passed
- `env CARGO_INCREMENTAL=0 cargo test --test hwpx_roundtrip_baseline`: 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --test visual_roundtrip_baseline`: 3 passed
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과

## Stage 5. GitHub CI 확인

완료.

- `CI preflight`: pass
- `Build & Test`: pass
- `Build default-feature tests`: pass
- `Native Skia tests`: pass
- `WASM Build`: skipped
- `CodeQL preflight`: pass
- `Analyze rust`: pass
- `Analyze javascript-typescript`: pass
- `Analyze python`: pass
- `CodeQL`: pass

## Stage 6. 결론

차단 이슈는 발견하지 못했다. PR #1997은 merge 후보로 판단한다.

merge 후에는 PR 본문에 closing keyword가 없으므로 #1993 상태를 확인하고, 필요하면 후속 코멘트와 close 처리를 수행해야 한다.
