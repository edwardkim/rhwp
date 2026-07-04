# PR #1730 처리 계획 — #1725 각주 tail over-pagination 완화

## 대상

- PR: #1730
- 작성자: @planet6897
- 관련 이슈: #1725
- 문서 작성 시점 PR head: `e810ff44d9f1bfa7b11eff2626472286fff227ea`
- 로컬 검토 브랜치: `local/pr1730-visible`
- 처리 결과: #1730 merge 완료 (`6455746e78bd78d7a5c9c57f17524d50a8d35625`)
- 후속 이슈: #1733

## 커밋

1. `f89b1b2518deec5cb184a02d768e86dccb80184d`
   - `Task #1725: 흐름 텍스트 각주 tail 문단 over-pagination 수정 (각주 안전마진 tail 완화)`
   - 실제 기능/문서/샘플 변경 커밋
2. `e810ff44d9f1bfa7b11eff2626472286fff227ea`
   - `Merge branch 'devel' into pr/devel-1725`
   - 최신 `devel` 동기화용 merge commit

## 검토 단계

### Stage 1. PR 메타 확인

- base branch: `devel`
- mergeable: `MERGEABLE`
- mergeStateStatus: `CLEAN`
- maintainerCanModify: `true`
- GitHub Actions: 전부 success, WASM Build skipped

### Stage 2. 변경 내용 검토

- `src/renderer/typeset.rs`만 기능 변경
- tail-before-vpos-reset 일반 텍스트 문단에서 각주 안전마진 40px만 1회 완화
- 실제 각주 높이는 유지하므로 겹침 위험은 제한적

### Stage 3. PDF 기준 페이지 수 검증

완료.

- 기준 PDF: 242쪽
- `upstream/devel` + PR 샘플: 258쪽
- PR head + PR 샘플: 250쪽
- near-empty: 26개 → 18개
- PR head export-pdf 산출물: 250쪽

### Stage 4. 회귀 게이트 검증

완료.

- `samples/byeolpyo1.hwp`: 4쪽
- `samples/byeolpyo4.hwp`: 26쪽
- `samples/task1718/table_giant_cell_overfill.hwp`: 42쪽
- `CARGO_INCREMENTAL=0 cargo test --lib`: 2044 passed, 0 failed, 7 ignored

### Stage 5. merge 전 결정 필요

결정 결과: B 경로로 처리했다.

- 기능 변경은 그대로 merge
- 문서 문구 불일치와 검증 기준 PDF/HWP 첨부는 후속 문서/기준자료 PR에서 정리
- merge 후 감사 코멘트에서 잔여 +8쪽은 후속 다중 원인으로 #1733에 분리한다고 명시

## merge 후 필수 후속 처리

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. `gh pr merge 1730 --repo edwardkim/rhwp --merge --admin`: 완료
2. #1725 이슈 close 여부 확인: auto-close 실패 확인
3. auto-close 실패 시 수동 close 및 감사 코멘트: 완료
4. PR 감사 코멘트 작성: 완료
5. `devel` sync: 완료
6. 리뷰 문서 archive 이동: 완료
7. 오늘할일 문서 갱신: 후속 기록 PR에 포함
8. 후속 문서/샘플 PR 필요 여부 확정: 필요, 본 후속 기록 PR에서 처리

## 후속 코멘트에 포함할 요지

- PDF 기준 242쪽 대비 PR head는 250쪽으로, 수정 전 258쪽에서 8쪽 개선 확인
- near-empty 페이지 26개에서 18개로 감소 확인
- byeolpyo1/byeolpyo4/#1718 대표 샘플 무회귀 확인
- 잔여 +8쪽은 PR 본문처럼 PartialParagraph/PartialTable/비각주 tail 등 다중 원인으로 후속 추적

## 후속 기록 PR 포함 파일

- `mydocs/pr/archives/pr_1730_review.md`
- `mydocs/pr/archives/pr_1730_review_impl.md`
- `mydocs/orders/20260701.md`
- `mydocs/report/task_m100_1725_report.md`
- `pdf/text_footnote_tail_overpagination-2024.pdf`
- `samples/task1725/text_footnote_tail_overpagination.hwp`
