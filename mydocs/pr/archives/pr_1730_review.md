# PR #1730 리뷰 — #1725 흐름 텍스트 각주 tail over-pagination 완화

- PR: #1730 `Task #1725: 흐름 텍스트 각주 tail 문단 over-pagination 수정 (각주 안전마진 tail 완화)`
- 작성자: @planet6897
- 기준 브랜치: `devel`
- PR head: `e810ff44d9f1bfa7b11eff2626472286fff227ea` (문서 작성 시점 참고값)
- 로컬 검토 브랜치: `local/pr1730-visible` (`6040140a8`, PR head와 diff 없음)
- 규모: 4 files, +86/-1
- 관련 이슈: #1725
- 문서 작성 시점 상태: `MERGEABLE`, `CLEAN`, GitHub Actions 전부 성공
- 처리 결과: 2026-07-01 `6455746e78bd78d7a5c9c57f17524d50a8d35625` merge 완료
- 후속 처리: #1725 수동 close 완료, 잔여 +8쪽 후속 #1733 등록, PR 감사 코멘트 완료

## 변경 요약

흐름 텍스트 법령의 각주가 있는 페이지에서, 다음 문단이 `vpos-reset`으로 새 페이지를 시작할 때 직전
tail 문단이 각주 안전마진 40px 때문에 수 px 차이로 밀려 단독 near-empty 페이지가 되는 문제를 완화한다.

핵심 변경은 `src/renderer/typeset.rs`에 한정된다.

- `TypesetState::skip_footnote_margin_once` 추가
- Task #359 `next_will_vpos_reset` 일반 텍스트 분기에서 tail 문단에 대해 1회 플래그 설정
- `typeset_paragraph` fit 계산에서 실제 각주 높이는 유지하고, 보수 버퍼인 `footnote_safety_margin`만 1회 되돌림

실제 각주 높이를 제거하지 않고 보수 버퍼만 완화하므로, 겹침 위험을 크게 키우지 않는 좁은 보정이다.

## 변경 범위

- 코드: `src/renderer/typeset.rs`
- 재현 샘플: `samples/task1725/text_footnote_tail_overpagination.hwpx`
- 계획/보고 문서:
  - `mydocs/plans/task_m100_1725.md`
  - `mydocs/report/task_m100_1725_report.md`

## PDF 기준 페이지 수 재검증

로컬 기준 PDF:

- `pdf/text_footnote_tail_overpagination-2024.pdf`
- `pdfinfo`: 242 pages, A4, Creator `Hwp 2024 13.0.0.3622`
- 이 PDF/HWP 기준 파일은 후속 기록 PR에 함께 첨부한다.

PR 샘플 기준 전후 비교:

| 항목 | 결과 |
|------|------|
| `upstream/devel` + PR 샘플 | 258쪽 |
| PR #1730 head + PR 샘플 | 250쪽 |
| 기준 PDF | 242쪽 |
| 개선 폭 | +16쪽 격차에서 +8쪽 격차로 완화 |

near-empty 페이지 집계도 PR 본문과 일치했다.

| 항목 | 결과 |
|------|------|
| `upstream/devel` | 26개 |
| PR #1730 head | 18개 |

추가 확인:

- PR head에서 `export-pdf samples/task1725/text_footnote_tail_overpagination.hwpx` 결과도 250페이지로 확인했다.
- 작업지시자 요청에 따라 임시 산출물 `tmp/pdfs/pr1730_hwpx_rhwp.pdf`는 삭제했다.
- 검증 HWP `samples/task1725/text_footnote_tail_overpagination.hwp`는 PR head에서 249쪽으로 측정됐다.
- 검증 기준 파일:
  - `pdf/text_footnote_tail_overpagination-2024.pdf`
  - `samples/task1725/text_footnote_tail_overpagination.hwp`

## 회귀 게이트

PR 본문에 적힌 대표 회귀 샘플 값을 로컬에서 재확인했다.

| 샘플 | 결과 |
|------|------|
| `samples/byeolpyo1.hwp` | 4쪽 |
| `samples/byeolpyo4.hwp` | 26쪽 |
| `samples/task1718/table_giant_cell_overfill.hwp` | 42쪽 |

## 로컬 검증

- `git diff --check upstream/devel...HEAD`: 통과
- `CARGO_INCREMENTAL=0 cargo build --quiet --bin rhwp`: 통과
- `upstream/devel` 전환 후 동일 PR 샘플 `dump-pages`: 258쪽
- PR head 복귀 후 동일 PR 샘플 `dump-pages`: 250쪽
- PR head `dump-pages` near-empty 집계: 18개
- `CARGO_INCREMENTAL=0 cargo test --lib`: 2044 passed, 0 failed, 7 ignored, 146.11s

## GitHub Actions

문서 작성 시점 PR head `e810ff44d9f1bfa7b11eff2626472286fff227ea` 기준:

- CI preflight: success
- Build & Test: success
- CodeQL preflight: success
- Analyze (rust/javascript-typescript/python): success
- Render Diff preflight: success
- Canvas visual diff: success
- WASM Build: skipped
- CodeQL: success

## 리뷰 결과

Blocking finding 없음.

PR은 #1725의 전체 242쪽 정합을 완성하지는 않지만, PR 본문도 이를 명확히 "258 → 250" 부분 개선으로
기록하고 있다. 실제 로컬 검증에서도 수정 전 258쪽, 수정 후 250쪽, near-empty 26→18개가 재현됐다.
회귀 게이트로 제시된 byeolpyo1/byeolpyo4/#1718 대표 샘플도 PR 본문 값과 일치한다.

## 비차단 확인 사항

- `mydocs/report/task_m100_1725_report.md`의 산출물 절은 "대형이라 samples 미포함"이라고 되어 있지만,
  실제 PR diff에는 `samples/task1725/text_footnote_tail_overpagination.hwpx`가 포함되어 있다. 후속 기록 PR에서
  문구를 실제 산출물과 맞게 보정한다.
- PR 본문이 `closes #1725`를 사용했지만 `devel` merge라 auto-close 되지 않았다. #1725는 수동 close했고,
  잔여 +8쪽과 near-empty 18개는 후속 #1733으로 분리했다.
- 기준 PDF/HWP 파일은 후속 기록 PR에 첨부한다.

## 최종 판단

수용 및 merge 완료.

- PR merge: https://github.com/edwardkim/rhwp/pull/1730
- merge commit: `6455746e78bd78d7a5c9c57f17524d50a8d35625`
- #1725 close comment: https://github.com/edwardkim/rhwp/issues/1725#issuecomment-4853893512
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1730#issuecomment-4853887327
- 잔여 원인 후속 이슈: #1733
