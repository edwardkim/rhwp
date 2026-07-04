# PR #1606 검토 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1606 `task 1599: 표 중심선 렌더링 보강` |
| URL | https://github.com/edwardkim/rhwp/pull/1606 |
| 관련 이슈 | #1599 `표의 중심선이 표시 되지 않습니다.` |
| 작성자 | @jangster77 |
| base | `devel` |
| head | `task_m100_1599_table_center_line` |
| 규모 | 문서 작성 시점 참고값: +526 / -26, 21 files |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |
| head SHA | 문서 작성 시점 참고값: `bf331e0594e2abdeb3a08967b4cea9e4a3947645` |

## 변경 범위

- `BorderFill`에 HWPX `centerLine`을 보존하는 `CenterLine` 값을 추가했다.
- HWP5 `BORDER_FILL` attr bit 13 및 bit 8/10 보조 비트에서 중심선 방향을 해석한다.
- HWPX 파서/직렬화에서 `centerLine`, `slash/backSlash`의 `Crooked`, `isCounter` 값을 보존한다.
- 셀 대각선 렌더링 경로에서 중심선을 기존 `DiagonalLine` 색/굵기/종류로 렌더링한다.
- 한컴 2024 기준 PDF 대조 결과에 맞춰 HWPX `VERTICAL`을 셀 중앙 가로 진행 막대로 표시하도록 보정했다.
- `samples/추진일정.hwp`, `samples/추진일정.hwpx`, `pdf/추진일정-2024.pdf`를 회귀 샘플로 추가했다.

## 검증

### 로컬 검증

- `cargo test --lib center_line -- --nocapture`: 통과
- `cargo test --lib diagonal -- --nocapture`: 통과
- `cargo test --lib`: 통과, 1975 passed, 0 failed, 7 ignored
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `samples/추진일정.hwp` 및 `samples/추진일정.hwpx` SVG/PNG 출력 비교: HWP/HWPX 출력 동일
- 한컴 2024 PDF 기준 PNG와 rhwp 출력 PNG 시각 검증: 작업지시자 확인 완료

### GitHub Actions

문서 작성 시점 참고값이며, merge 전 최신 PR head 기준으로 다시 확인한다.

- `CI preflight`: success
- `Build & Test`: success
- `CodeQL preflight`: success
- `Analyze (javascript-typescript)`: success
- `Analyze (python)`: success
- `Analyze (rust)`: success
- `Render Diff preflight`: success
- `Canvas visual diff`: success
- `WASM Build`: skipped

## 리스크

- 중심선 방향 명칭은 HWPX literal 보존용 이름과 화면 표시 방향이 다르다. 코드 주석에 한컴 2024 기준 표시 방향을 명시했다.
- 렌더링 변경 PR이므로 merge 후에도 필요 시 golden/snapshot 회귀를 확인한다.
- `Closes #1599`가 있어도 `devel` base 특성상 자동 close가 실패할 수 있으므로 merge 후 이슈 상태를 수동 확인한다.

## 판단

로컬 검증, 원격 CI, 기준 PDF 시각 검증이 모두 통과했다. 최종 merge 조건은 최신 PR head 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 처리 결과

- #1606 merge 완료: `a6de4b66e7d91ba3e21395328008e8ab162a8548`
- merge 시각: 2026-06-28 02:30 KST
- #1599 상태: 자동 close 실패 확인 후 수동 close 완료
- merge 후 렌더 영향 체크: `cargo test --test svg_snapshot` 통과, 8 passed
- 원격/로컬 작업 브랜치 `task_m100_1599_table_center_line` 삭제 및 prune 확인 완료
