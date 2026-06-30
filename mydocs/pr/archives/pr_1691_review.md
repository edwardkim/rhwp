# PR #1691 검토 보고서 — 행정업무 편람 페이지 수 정합

- PR: https://github.com/edwardkim/rhwp/pull/1691
- 제목: `task 1672: 행정업무 편람 페이지 수를 PDF 기준과 맞춤`
- 작성자: `jangster77` (collaborator self-merge 후보)
- 연결 이슈: #1672 `2025 행정업무운영 편람 전체 페이지 수가 한컴 PDF와 불일치`
- 작성일: 2026-06-30
- 처리 경로: collaborator self-merge 후보 예외 경로. review 문서를 PR head 에 포함해 merge 후 별도 문서 PR 을 만들지 않는다.
- base/head: `edwardkim/rhwp:devel` <- `edwardkim/rhwp:task_m100_1672`
- 작성 시점 참고값: `MERGEABLE`, Open PR, draft 아님
- 작성 시점 참고 규모: 8 files, +667 / -38 (review 문서 추가 전 코드·샘플 변경 기준)

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 merge 판단 전 최신 상태를 다시 확인한다.

## 1. 요약 판단

PR #1691은 #1672의 행정업무 편람 샘플 페이지 수 불일치를 PDF oracle 기준으로 맞추는 내부 타스크 PR이다.
대상 샘플의 기준 PDF 페이지 수는 383쪽이며, 수정 후 HWP/HWPX 양쪽 `rhwp info` 결과가 모두 383쪽으로 맞았다.

권고는 **GitHub Actions 최종 green 확인 + 작업지시자 승인 후 merge 후보**다. 단, 렌더/페이지네이션 경험칙을 건드리므로
열린 PR 과의 파일 겹침, 대형 샘플 추가, 향후 다른 RowBreak 문서 회귀 가능성은 merge 전 마지막으로 확인한다.

## 2. 관련 이슈 요약

#1672의 현상은 `samples/2025 행정업무운영 편람(최종).hwp`와 대응 HWPX에서 전체 페이지 수가 한컴 PDF 기준과 맞지 않는 문제다.

- PDF oracle: `pdf/2025 행정업무운영 편람(최종)-2024.pdf`
- 기대값: 383쪽
- 기존 rhwp 계산: HWP 416쪽, HWPX 416쪽
- 문제 범위: HWP/HWPX가 동일하게 과대 분할되므로 포맷별 파서보다 공통 레이아웃·페이지네이션 경로 우선

## 3. 변경 범위

| 영역 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | RowBreak 표 계열의 layout drift 안전마진을 문서 특성에 따라 조정하고, 저장 LINE_SEG vpos 기반 fit 상한과 spacing_before 흐름 보존 조건을 보정 |
| `src/renderer/layout/table_layout.rs` | RowBreak hard-break 완화 조건을 큰 표 또는 2열 이하 표로 제한해 소형 표 회귀를 방지 |
| `samples/2025 행정업무운영 편람(최종).hwp` | 페이지 수 회귀 확인용 HWP 샘플 추가 |
| `samples/2025 행정업무운영 편람(최종).hwpx` | 동일 샘플의 HWPX 경로 확인용 데이터 추가 |
| `pdf/2025 행정업무운영 편람(최종)-2024.pdf` | Hancom PDF 기준 oracle 추가 |
| `mydocs/plans`, `mydocs/working` | #1672 수행·구현 계획과 stage1 완료 기록 |

## 4. 로컬 검증 결과

아래 검증은 PR 생성 전 로컬 head 기준으로 수행했다.

```text
cargo fmt --check
cargo test --release --lib
cargo test --profile release-test --tests
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check upstream/devel..HEAD
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
```

검증 결과 요약:

| 항목 | 결과 |
|---|---|
| release unit | `2006 passed; 0 failed; 7 ignored` |
| release-test integration | 통과 |
| doctest | `0 passed; 0 failed; 1 ignored` |
| clippy | 통과, 0 warning |
| studio test | `153 passed; 0 failed` |
| WASM build | 통과 |
| diff check | 통과 |

페이지 수 확인:

| 대상 | 결과 |
|---|---|
| PDF oracle | 383 |
| `samples/2025 행정업무운영 편람(최종).hwp` | 383 |
| `samples/2025 행정업무운영 편람(최종).hwpx` | 383 |
| `samples/k-water-rfp.hwp` 회귀 확인 | 27 |
| `samples/hwpspec.hwp` 회귀 확인 | 178 |

## 5. GitHub Actions 작성 시점 참고값

PR 생성 직후 조회 기준:

- `CI preflight`: success
- `CodeQL preflight`: success
- `Render Diff preflight`: success
- `WASM Build`: skipped
- `Build & Test`: in progress
- `Canvas visual diff`: in progress
- `Analyze (rust/javascript-typescript/python)`: in progress

review 문서 커밋이 PR head 에 추가되면 Actions 는 새 head 기준으로 다시 판단해야 한다. 최종 merge 조건은
**PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인**이다.

## 6. 리스크와 확인 포인트

### 6.1 열린 PR 파일 겹침

작성 시점 열린 PR 중 다음 파일 겹침이 있다.

| PR | 겹치는 파일 |
|---|---|
| #1690 | `src/renderer/layout/table_layout.rs` |
| #1688 | `src/renderer/layout/table_layout.rs` |
| #1683 | `src/renderer/typeset.rs` |
| #1170 | `src/renderer/layout/table_layout.rs` |

merge 순서가 바뀌면 conflict 또는 경험칙 상호작용 가능성이 있으므로 merge 직전 최신 `devel` 기준으로 재확인한다.

### 6.2 대형 샘플 추가

실제 행정업무 편람 HWP/HWPX/PDF를 회귀 oracle 로 포함한다. 페이지 수 정합 확인에는 유효하지만 저장소 크기가 늘어난다.
사용자가 "샘플데이터도 git 에 포함"을 명시했으므로 이번 PR 범위에 포함했다.

### 6.3 경험적 페이지네이션 보정

RowBreak 표가 많은 문서의 누적 over-pagination을 줄이기 위한 보정이다. `k-water-rfp`, `hwpspec`, 기존 통합 테스트로
대표 회귀는 확인했지만, RowBreak 표가 많은 다른 실문서에서는 추가 관찰이 필요할 수 있다.

## 7. 최종 권고

- 권고: **GitHub Actions 최신 head green 확인 후 작업지시자 승인 기반 merge 후보**
- merge 전 필수 확인:
  - PR head 최신 커밋 기준 GitHub Actions 통과
  - #1690/#1688/#1683/#1170 등 겹침 PR 과의 merge 순서 확인
  - `Closes #1672` 자동 close 동작 확인 계획 유지
- merge 후 확인:
  - #1672가 자동 close 되지 않으면 수동 close/comment 수행
  - PR 브랜치와 로컬 작업 브랜치 정리
