# PR #1766 리뷰 — Task #1765 병합 셀 경로 가설 기각 조사

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1766 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1765` |
| 관련 이슈 | #1765, refs #1759 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE, mergeStateStatus=BLOCKED |
| head 참고값 | `b4218b6a5ff6a1d8088a55b655a58c659cd042ba` |
| reviewer assign | @jangster77 요청 완료 |
| 메인터너 보정 | 계획서/README 결론 상충 정리, 기준 PDF/HWPX 샘플 및 visual asset 추가 |

## 변경 범위

소스 변경 없이 #1765 가설을 조사하고, 병합 셀 trailing line-spacing 가설 기각이 맞음을 문서/샘플/기준
PDF 대조로 증명하는 PR이다.

- `mydocs/plans/task_m100_1765.md`
- `mydocs/plans/task_m100_1765_impl.md`
- `mydocs/working/task_m100_1765_stage1.md`
- `mydocs/working/task_m100_1765_stage2.md`
- `mydocs/report/task_m100_1765_report.md`
- `samples/task1765/README.md`
- `samples/task1765/merged_cell_trailing_ls.hwp`
- `samples/task1765/merged_cell_trailing_ls.hwpx`
- `samples/task1765/merged_cell_trailing_ls-2024.pdf`
- `mydocs/pr/assets/pr_1766_merged_cell_hwp_visual_review_p2.png`
- `mydocs/pr/assets/pr_1766_merged_cell_hwpx_visual_review_p2.png`

PR head에는 `Merge branch 'devel' into pr/devel-1765` 커밋이 포함되어 있으므로, 로컬 검토에서는 실제
조사 커밋 `a200cb0b833a82d32e9a921c6566a24b80178f2b`만 `upstream/devel` 기준으로 cherry-pick 했다.

## 로컬 검증

- `git diff --check upstream/devel..HEAD`
  - 통과
- `cargo fmt --check`
  - 통과

소스 변경이 없으므로 cargo build/test 전체는 아직 수행하지 않았다. PR #1766의 GitHub Actions는 문서 작성
시점에 Build & Test, Render Diff, CodeQL 일부가 진행 중이다.

## 검토 결과

현재 PR 의 최종 보고서와 Stage 2는 "가설 기각 / per-line 콘텐츠 측정 누적으로 재분류"라는 결론을 제시한다.
이 PR 의 merge 목적은 그 결론이 맞음을 증명하는 샘플과 검증 기록을 `devel` 에 보존하는 것이다.

초기 검토 시 다음 문서가 원래 가설을 사실처럼 유지하고 있어, PR만 읽는 사람에게 상충된 메시지를 줬다.

- `mydocs/plans/task_m100_1765.md`
  - 병합 셀 경로 trailing ls 가드 확장을 수정 방향으로 확정 서술한다.
- `mydocs/plans/task_m100_1765_impl.md`
  - Stage 2를 "가드 확장 + 테스트"로 남겨 실제 결론과 다르다.
- `samples/task1765/README.md`
  - 샘플 구조를 "rowspan 병합 셀"로 설명하고, 기대를 "trailing ls 억제"로 적는다.
  - Stage 2/최종 보고서의 "셀 전부 rs=1, 병합 경로 미경유, per-line 누적" 결론과 충돌한다.

메인터너 보정으로 위 문서를 최종 결론 기준으로 정리했다.

- 수행계획서: "가드 확장" 확정 서술을 "가설 검증 및 기각" 흐름으로 수정
- 구현계획서: Stage 2를 "가드 확장 + 테스트"에서 "가설 검증 + 기각"으로 수정
- Stage 1 보고서: 원인 단정을 "초기 가설"로 수정
- README: 샘플 목적을 #1759 per-line 콘텐츠 측정 누적 대표 사례로 정정

## 시각 검증 / 샘플 보강 상태

작업지시자가 기준 PDF/HWPX 샘플을 추가했다.

추가 샘플:

- `samples/task1765/merged_cell_trailing_ls-2024.pdf`
  - 한컴 2024 13.0.0.3622 / Hancom PDF 1.3.0.550 기준 PDF
  - PDF 페이지 수: 4쪽
- `samples/task1765/merged_cell_trailing_ls.hwpx`
  - HWPX 경로 visual sweep 검증용 샘플

`mydocs/manual/visual_sweep_guide.md` 기준으로 문제 페이지인 2쪽을 HWP/HWPX 양쪽에서 기준 PDF와 비교했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1766-merged-cell-hwp samples/task1765/merged_cell_trailing_ls.hwp samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --file-target pr1766-merged-cell-hwpx samples/task1765/merged_cell_trailing_ls.hwpx samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --page 2 \
  --out output/pr1766-visual-review
```

결과:

- HWP: SVG/PDF 페이지 수 4 / 4, 선택 페이지 2, 자동 후보 `0/1`
- HWPX: SVG/PDF 페이지 수 4 / 4, 선택 페이지 2, 자동 후보 `0/1`
- HWP review PNG:
  - `output/pr1766-visual-review/pr1766-merged-cell-hwp/review/review_002.png`
  - asset: `mydocs/pr/assets/pr_1766_merged_cell_hwp_visual_review_p2.png`
  - compare: `output/pr1766-visual-review/pr1766-merged-cell-hwp/compare/compare_002.png`
  - overlay: `output/pr1766-visual-review/pr1766-merged-cell-hwp/overlay/overlay_002.png`
- HWPX review PNG:
  - `output/pr1766-visual-review/pr1766-merged-cell-hwpx/review/review_002.png`
  - asset: `mydocs/pr/assets/pr_1766_merged_cell_hwpx_visual_review_p2.png`
  - compare: `output/pr1766-visual-review/pr1766-merged-cell-hwpx/compare/compare_002.png`
  - overlay: `output/pr1766-visual-review/pr1766-merged-cell-hwpx/overlay/overlay_002.png`
- visual_accuracy_proxy_percent:
  - HWP page 2: 15.17532
  - HWPX page 2: 15.17532

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 15.18%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

review PNG 기준으로는 2쪽 표의 전체 위치와 흐름이 HWP/HWPX 모두 기준 PDF와 같은 구조이며, 자동 후보도
없다.

## 결론

초기 문서 상충은 메인터너 보정으로 해소했다. 기준 PDF/HWPX 샘플과 visual asset도 옵션 1 방식으로 같은
PR에 포함한다.

최신 PR head 기준 GitHub Actions가 통과하면 "병합 셀 경로 가설 기각 증명" PR로 merge 후보로 판단한다.
