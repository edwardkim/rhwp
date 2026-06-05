# PR #1083 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1083>
- 제목: Task #1073: 중첩 표(셀 내부 표) 페이지 분할 — 본문 하단 overflow 해소
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1073>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `pr/task1073-nested-table-split` |
| head sha | `f2a85f86f0234f42f420c40c0b5a2c24ec7914f5` |
| mergeable | true |
| 변경 파일 | 12개 |
| 증감 | +667 / -5 |
| 리뷰/댓글 | 없음 |

CI 확인:

| check | status |
|---|---|
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 2. 변경 요약

PR #1083은 `samples/kps-ai.hwp`에서 관측된 중첩 표 overflow 문제를 해결한다.

문제 상황:

```text
셀 안에 페이지보다 큰 중첩 표가 있고,
외부 표 행 분할만으로는 페이지에 들어가지 않아 PartialTable 이 본문 하단을 약 758px 초과한다.
```

수정 방향:

```text
중첩 표를 하나의 atomic cell content 로 취급하지 않고,
텍스트 없는 단일 중첩 표 문단을 중첩 표의 행 단위 CellUnit 으로 분해한다.
그 뒤 start_cut/end_cut 을 NestedTableSplit(start_row, end_row) 으로 연결해 연속 페이지에서
중첩 표가 다시 0행부터 렌더링되지 않도록 한다.
```

주요 코드 변경:

| 파일 | 내용 |
|---|---|
| `src/renderer/height_measurer.rs` | `MeasuredCell.nested_split_row_count` 추가, 중첩 표 행 분할 가능 여부 판정 |
| `src/renderer/layout/table_layout.rs` | `CellUnit.nested_row` 추가, 단일 중첩 표 문단을 per-row 유닛으로 분해 |
| `src/renderer/layout/table_partial.rs` | cut unit 범위를 `NestedTableSplit` 범위로 매핑 |
| `tests/issue_1073_nested_table_split.rs` | `samples/kps-ai.hwp` 회귀 가드 3개 추가 |

## 3. 현재 코드 반영 여부

현재 `local/devel`에는 PR #1083의 핵심 구현이 아직 없다.

확인 결과:

```text
src/renderer/height_measurer.rs:
  MeasuredCell.nested_split_row_count 없음

src/renderer/layout/table_layout.rs:
  CellUnit.nested_row 없음
  중첩 표 per-row CellUnit 분해 없음

src/renderer/layout/table_partial.rs:
  cut unit -> NestedTableSplit(start_row, end_row) 직접 매핑 없음
```

샘플과 권위 비교 자료는 로컬에 존재한다.

```text
samples/kps-ai.hwp
pdf/kps-ai-2022.pdf
```

## 4. 검토 포인트

### 4.1 수용 가능성이 높은 점

- PR의 타깃 결함과 이슈 #1073의 증상이 일치한다.
- 변경이 표 분할 엔진의 실제 병목인 4개 지점에 걸쳐 있다.
  - 분할 가능성 판정
  - 측정 모델
  - 컷 모델
  - 부분 렌더 매핑
- 공개 샘플 `samples/kps-ai.hwp` 기반 회귀 테스트가 추가되어 있다.
- PR CI는 `Build & Test`, `Canvas visual diff`, `Analyze (rust)` 모두 통과했다.

### 4.2 주의해야 할 점

이 변경은 단순 버그 픽스라기보다 표 분할 엔진의 기능 확장이다.

특히 다음 공유 경로를 건드린다.

```text
cell_units
advance_row_cut
row_cut_content_height
cell_line_ranges_from_cut
layout_partial_table
```

따라서 PR 자체 CI가 통과했더라도, 현재 `devel` 기준으로 체리픽 후 다음 검증이 필요하다.

```text
cargo fmt --check
cargo check
cargo test --lib
cargo test --test issue_1073_nested_table_split
필요 시 kps-ai SVG/page dump 확인
```

### 4.3 잔여 한계

PR 본문과 보고서가 다음 한계를 명시한다.

```text
break row 가 한컴 대비 약 2 중첩행 늦음.
구조/콘텐츠 정합과 overflow 0은 달성했지만, break row 정확 일치는 잔여.
2단계 이상 중첩, 텍스트 동거 문단 중첩 표는 atom 폴백.
```

이 한계는 PR 수용 시 별도 후속 이슈 또는 known limitation 으로 남기는 것이 적절하다.

## 5. 권장 처리 방향

권장안:

```text
1. PR #1083 단일 커밋을 현재 local/devel 에 -x cherry-pick 한다.
2. 충돌 여부와 현재 devel 기준 테스트 결과를 확인한다.
3. 테스트가 통과하면 devel 에 반영하고 PR #1083 및 이슈 #1073을 체리픽 수용으로 close 한다.
4. break row 정밀도 잔여는 별도 이슈 또는 문서화 대상으로 남긴다.
```

바로 merge commit 으로 받기보다는 `-x` cherry-pick 이 적합하다.

이유:

```text
- PR base 가 현재 devel 보다 오래되었다.
- 변경 범위는 단일 커밋으로 정리되어 있다.
- 외부 기여자 출처를 커밋 메시지에 남길 수 있다.
- 현재 devel 기준 검증을 로컬에서 다시 수행하기 쉽다.
```

## 6. 승인 요청

다음 절차로 진행해도 되는지 승인 요청한다.

```text
git fetch origin pull/1083/head:pr/1083
git cherry-pick -x f2a85f86f0234f42f420c40c0b5a2c24ec7914f5
cargo fmt --check
cargo check
cargo test --lib
cargo test --test issue_1073_nested_table_split
```

