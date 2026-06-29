# PR #1647 리뷰 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1647 |
| 제목 | task 1633: 셀존 대각선 UI와 HWP 저장 호환성 보정 |
| 작성자 | jangster77 |
| base | devel |
| head | task_m100_1633 |
| 관련 이슈 | #1633 |
| 규모 | 문서 작성 시점 참고값: 41 files, +2741 / -96 |
| head SHA | merge 전 최종 확인값: `e77ff40bfd7a23aaa81caa03cc1ad5b8aec20e65` |
| mergeable | merge 전 최종 확인값: `MERGEABLE`, `CLEAN` |
| CI 상태 | merge 전 최종 확인값: Build & Test, CodeQL, Render Diff 통과 |
| merge commit | `f5d1b22074d383f847cd149eed32f261f2e80922` |
| 이슈 처리 | #1633 자동 close 실패 확인 후 수동 close/comment 완료 |
| 작성 시각 | 2026-06-29 13:40 KST |

## 이슈 요약

#1633은 `대각선샘플.hwp`에서 대각선이 적용된 셀을 선택한 뒤
`셀 테두리/배경 > 대각선` UI를 열었을 때, 한컴과 다르게 현재 셀 상태가
초기화되지 않는 문제로 시작했다. 진행 중 사용자 검증을 통해 UI 초기화뿐 아니라
`cellzone` 대각선/중심선 렌더링, `각 셀마다 적용`과 `하나의 셀처럼 적용`의
저장 구조, HWP 저장 후 한컴 표시 호환성 문제가 함께 확인됐다.

## 변경 범위

- `cellzone` overlay가 적용된 셀의 effective borderFill을 UI 조회에 반영한다.
- 대각선 탭의 선 속성, 방향 버튼, 해제 버튼, 미리 보기 초기화 동작을 한컴 기준으로 보정한다.
- 여러 셀 선택 시에만 `하나의 셀처럼 적용`을 활성화하고, 단일 셀에서는 해당 경로를 비활성화한다.
- 문서가 로드되지 않은 상태의 각주/미주 삽입 메뉴를 비활성화한다.
- `하나의 셀처럼 적용` 대각선은 선택 영역 `cellzone`에 저장하고, HWP TABLE cellzone 직렬화를 보강한다.
- `각 셀마다 적용` 대각선은 셀 고유 border fill과 필요 시 1x1 override zone으로 렌더링을 보존한다.
- 중심선은 한컴 호환을 위해 1x1 cellzone이 아니라 셀 border fill로 렌더링/저장한다.
- 신규/HWPX 출처 표 셀의 HWP LIST_HEADER를 한컴식 47바이트 구조로 보강한다.
- `대각선샘플`, `대각선샘플3`, `대각선샘플4`, `대각선샘플5` 및 focused 회귀 테스트를 추가했다.

## 로컬 검증

- `cargo fmt --check && git diff --check`: 통과
- `cargo test --test issue_1623_cellzone_diagonal`: 통과, 19 passed
- `cargo test --lib serializer::control`: 통과, 10 passed
- `wasm-pack build --target web --out-dir pkg`: 통과
  - Codex 실행 환경에서는 `wasm-bindgen` prebuilt 미제공으로 `cargo install` fallback 경고가 있었으나 빌드는 완료됐다.
- `npm --prefix rhwp-studio run build`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과

## 원격 검증

최종 head `e77ff40bfd7a23aaa81caa03cc1ad5b8aec20e65` 기준 GitHub Actions 결과:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- Build & Test: success
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- Canvas visual diff: success
- WASM Build: skipped
- CodeQL check: success

## 리스크

- 대각선과 중심선은 한컴 UI/저장 구조를 기준으로 분기했다. 후속 샘플에서 한컴의 세부 cellzone precedence가 더 발견되면 케이스별 가드가 추가로 필요할 수 있다.
- PR head에는 devel 추월 대응을 위한 merge commit이 여러 차례 포함됐다. 최종 merge 전 최신 head 기준 CI 통과와 `CLEAN` 상태를 확인했다.

## 최종 처리 결과

#1647은 최종 head `e77ff40bfd7a23aaa81caa03cc1ad5b8aec20e65` 기준 GitHub Actions 통과와
`MERGEABLE` / `CLEAN` 상태를 확인한 뒤 admin merge 했다. merge commit 은
`f5d1b22074d383f847cd149eed32f261f2e80922` 이다.

#1633은 PR merge 후 자동 close가 동작하지 않아 수동 close/comment 처리했다. 본 review 문서와
오늘할일 갱신은 작업지시자 지시에 따라 #1647 merge 후 별도 `mydocs/**` 문서 전용 PR로 처리한다.
