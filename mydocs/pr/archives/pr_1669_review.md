# PR #1669 리뷰 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1669 |
| 제목 | task 1634: 선택 셀 전치 기능 구현 |
| 작성자 | jangster77 |
| base | devel |
| head | task/m100-1634-transpose |
| 관련 이슈 | #1634 |
| 규모 | 문서 작성 시점 참고값: 22 files, +1188 / -4 |
| head SHA | 문서 작성 시점 참고값: `ce3a385681dd14574397921bd94f4a7e0ba997de` |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE`, `BEHIND` |
| CI 상태 | PR 생성 직후 실행 대기/진행 상태. merge 전 최신 PR head 기준 GitHub Actions 통과 필요 |
| 작성 시각 | 2026-06-29 20:48 KST |

## 이슈 요약

#1634는 표에서 선택한 셀 범위를 행/열 전치해 붙여넣는 기능 요청이다. 사용자가 제시한 기준 동작은
스프레드시트의 `TRANSPOSE(B2:C5)`처럼 4x2 범위를 2x4 형태로 전환하는 것이다.

진행 중 사용자 검증에서 다음 세부 동작이 확정됐다.

- 전체 표를 선택한 경우에는 새 표를 만들지 않고 기존 표 자체를 전치한다.
- 표 일부만 선택한 경우에는 선택 범위 좌상단을 대상 시작 셀로 삼아 기존 표 안에 전치 붙여넣기를 먼저 시도한다.
- 전치 결과가 기존 표 범위를 벗어나는 경우에만 새 표 생성으로 폴백한다.

## 변경 범위

- 표 모델에 `TableTransposeData`와 셀 범위 전치 유틸리티를 추가했다.
- `DocumentCore`에 전치 복사 버퍼, 전치 붙여넣기, 전체 표 전치 API를 추가했다.
- native/WASM API에서 표 내부 전치 붙여넣기와 표 밖 새 표 생성 경로를 노출했다.
- Studio 명령 체계에 `table:transpose-copy`, `table:transpose-paste`를 추가했다.
- 표 메뉴와 표/기본 컨텍스트 메뉴에 `셀 전치 복사`, `셀 전치 붙여넣기`를 연결했다.
- 부분 선택/전체 선택/표 밖 붙여넣기 경로에 대한 Rust 및 Studio 정적 테스트를 보강했다.
- `mydocs/plans`, `mydocs/working`에 이슈 진행 계획과 stage 기록을 추가했다.

## 로컬 검증

| 항목 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel..HEAD` | 통과 |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 1995 passed / 7 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 1 ignored |
| `cargo test --test svg_snapshot` | 통과, 8 passed |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm test` | 통과, 153 passed |
| `npm run build` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

## 리스크

- 병합 셀, 비직사각형 선택, 중첩 표 전치는 이번 1차 범위에서 제외했다.
- 부분 선택 전치 붙여넣기는 기존 표 안에 들어갈 수 있는 경우만 내부 덮어쓰기를 수행한다. 범위를 초과하면 새 표 생성으로 폴백하므로, 사용자가 기대한 대상 위치와 새 표 생성 위치가 다르게 느껴질 수 있다.
- PR 생성 직후 base `devel`이 한 차례 앞서 있어 GitHub 상태가 `BEHIND`로 표시됐다. merge 전 최신 PR head 기준 CI와 merge state를 재확인해야 한다.

## 최종 권고

작업지시자 승인 후 self-merge 후보로 진행 가능하다. merge 전 최종 조건은 다음과 같다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- review 문서와 처리 계획서가 PR diff에 포함됨
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 최종 승인
