# PR #1736 리뷰 — #1733 tail-before-vpos-reset over-pagination 추가 완화

- PR: #1736 `Task #1733 (부분): 흐름 텍스트 tail-before-vpos-reset over-pagination 추가 완화 (250→245)`
- 작성자: @planet6897
- 기준 브랜치: `devel`
- PR head: `9f541df2b48b8c5899267f7237e835710c853d3e` (문서 작성 시점 참고값)
- 규모: 2 files, +88/-1
- 관련 이슈: #1733
- 문서 작성 시점 상태: `MERGEABLE`, `BLOCKED` (GitHub required check `Build & Test` 대기)
- 처리 결과: 2026-07-01 `342199af1be055f573f65aca3a48e3579feea726` merge 완료
- 후속 처리: #1733은 부분 해결 이슈이므로 open 유지, PR 감사 코멘트 완료

## 변경 요약

#1730 merge 후에도 국제고속선기준 HWPX 샘플이 기준 PDF 242쪽 대비 250쪽으로 남아 있던 문제 중,
tail-before-vpos-reset 계열 일부를 추가 완화한다.

핵심 변경은 `src/renderer/typeset.rs`에 한정된다.

- tail 문단과 vpos-reset 문단 사이에 컨트롤 없는 빈 문단 1개가 끼어도 tail-before-vpos-reset으로 인식
- tail-before-vpos-reset 일반 텍스트 문단에 한해 `TAIL_BREAK_OVERFLOW_TOLERANCE_PX=20.0`을 1회 적용
- 기존 각주 안전마진 1회 완화와 함께, 각주가 없는 page-full tail over-fill 케이스도 좁게 완화

문서 변경은 `mydocs/report/task_m100_1733_report.md` 추가다.

## 변경 범위

- 코드: `src/renderer/typeset.rs`
- 보고서: `mydocs/report/task_m100_1733_report.md`

PR에는 실제 기능 커밋 1개와 `devel` 동기화 merge commit 1개가 포함되어 있다.

## 로컬 merge 검증

`upstream/devel` 기준 merge 시뮬레이션 결과 충돌 없음.

```bash
git checkout -B local/pr1736-merge-test upstream/devel
git merge local/pr1736 --no-commit --no-ff
```

결과: automatic merge 성공.

## 페이지 수 검증

PR head merge-test 코드 기준 `target/debug/rhwp dump-pages ... | grep -c global_idx` 결과:

| 샘플 | 결과 |
|------|------|
| `samples/task1725/text_footnote_tail_overpagination.hwpx` | 245쪽 |
| `samples/task1725/text_footnote_tail_overpagination.hwp` | 244쪽 |
| `samples/byeolpyo1.hwp` | 4쪽 |
| `samples/byeolpyo4.hwp` | 26쪽 |
| `samples/task1718/table_giant_cell_overfill.hwp` | 42쪽 |

PR 본문 재현 명령의 기준인 HWPX 샘플은 245쪽으로 확인됐다. 같은 기준 자료의 HWP 파일은 244쪽으로
측정되어, 리뷰 문서에서는 HWPX와 HWP 결과를 구분해 기록한다.

## 로컬 검증

새 PR review 지침에 따라 cargo 검증 전 `target` 하위 항목을 삭제한 뒤 수행했다.

- `CARGO_INCREMENTAL=0 cargo test --lib`: 2044 passed, 0 failed, 7 ignored, real 171.90s
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과, real 18.30s
- 샘플 page count 검증: 위 표와 같음
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과, real 27.70s
- `CARGO_INCREMENTAL=0 cargo test --test svg_snapshot`: 8 passed, 0 failed, real 17.15s

## GitHub Actions

문서 작성 시점 PR head `9f541df2b48b8c5899267f7237e835710c853d3e` 기준 참고값 및 merge 직전 재확인 결과:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- Canvas visual diff: success
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- CodeQL: success
- WASM Build: skipped
- Build & Test: success, 18m43s

merge 직전 `MERGEABLE`, `CLEAN`, GitHub Actions 전부 성공을 확인했다.

## 리뷰 결과

Blocking finding 없음.

완화 조건은 "현재 페이지에 이미 항목이 있고, 현재 문단이 visible text이며, 다음 또는 그 다음 문단이
vpos-reset으로 새 페이지를 시작하는 tail"에 묶여 있다. `tail_overflow_tolerance_once`도 fit 계산에서
소비되며 즉시 0으로 리셋되므로 지속 상태 누수 위험은 낮다.

## 비차단 리스크

- 기능 변경에 대응하는 새 단위 테스트는 추가되지 않았다. 다만 PR 본문이 의도한 대표 샘플 page count,
  기존 `cargo test --lib`, `clippy --all-targets`, `svg_snapshot`은 로컬에서 통과했다.
- #1733 전체 목표인 PDF 기준 242쪽에는 아직 도달하지 않는다. 본 PR은 250쪽에서 245쪽으로 줄이는
  부분 완화이며, 잔여 +3쪽은 #1733 후속 범위로 남는다.

## 최종 판단

수용 및 merge 완료.

- PR merge: https://github.com/edwardkim/rhwp/pull/1736
- merge commit: `342199af1be055f573f65aca3a48e3579feea726`
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1736#issuecomment-4854743131
- #1733: 부분 해결 이슈로 open 유지
