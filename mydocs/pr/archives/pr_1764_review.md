# PR #1764 리뷰 — Task #1763 셀 선언높이 권위 trailing line-spacing 보정

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1764 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1763` |
| 관련 이슈 | #1763 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE, mergeStateStatus=BEHIND |
| 누적 검토 순서 | #1761 → #1762 → #1764 |
| reviewer assign | @jangster77 요청 완료 |

## 변경 범위

다문단 셀 측정에서 마지막 줄 trailing line-spacing이 선언 셀높이를 초과 확장시키는 문제를 보정한다.

- `src/renderer/height_measurer.rs`
  - 마지막 줄 trailing line-spacing 산출과 선언높이 권위 가드 추가
- `src/renderer/typeset.rs`
  - #1753 prefill 상호작용 보정으로 fit 마진 제거
- `tests/issue_1763_cell_trailing_ls_expand.rs`
  - 재현 회귀 테스트 추가
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`
  - 의도된 golden 갱신
- `samples/task1763/**`, `mydocs/plans/report/working/task_m100_1763*`
  - 재현 샘플과 작업 문서

## 체리픽/충돌 확인

#1764는 원래 #1756 위 스택 PR이었고, #1756 merge 후 메인터너 권한으로 최신 `devel` 기준 clean head
`4e47d0fb1283cefcadd87bba08da0348c7b04230`로 정리했다.

최신 `upstream/devel` 기준 누적 브랜치에서 #1761, #1762 다음으로 #1764 clean head를 cherry-pick 했고
충돌은 없었다.

## 로컬 검증

- `cargo fmt --check`
  - 통과
- `git diff --check upstream/devel..HEAD`
  - 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1763_cell_trailing_ls_expand -- --nocapture`
  - 1 passed

PR #1764 최신 head 기준 GitHub Actions는 Build & Test, Render Diff, CodeQL 모두 통과 상태를 확인했다.

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 PR 재현 샘플을 한컴 2024 PDF와 비교했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1764-cell-trailing-hwp samples/task1763/cell_trailing_ls_expand.hwp samples/task1763/cell_trailing_ls_expand-2024.pdf \
  --file-target pr1764-cell-trailing-hwpx samples/task1763/cell_trailing_ls_expand.hwpx samples/task1763/cell_trailing_ls_expand-2024.pdf \
  --page 1 \
  --out output/pr1764-visual-review
```

결과:

- HWP: SVG/PDF 페이지 수 1 / 1, 자동 후보 `0/1`
- HWPX: SVG/PDF 페이지 수 1 / 1, 자동 후보 `0/1`
- HWP review PNG:
  - `output/pr1764-visual-review/pr1764-cell-trailing-hwp/review/review_001.png`
  - asset: `mydocs/pr/assets/pr_1764_cell_trailing_hwp_visual_review_p1.png`
  - compare: `output/pr1764-visual-review/pr1764-cell-trailing-hwp/compare/compare_001.png`
  - overlay: `output/pr1764-visual-review/pr1764-cell-trailing-hwp/overlay/overlay_001.png`
- HWPX review PNG:
  - `output/pr1764-visual-review/pr1764-cell-trailing-hwpx/review/review_001.png`
  - asset: `mydocs/pr/assets/pr_1764_cell_trailing_hwpx_visual_review_p1.png`
  - compare: `output/pr1764-visual-review/pr1764-cell-trailing-hwpx/compare/compare_001.png`
  - overlay: `output/pr1764-visual-review/pr1764-cell-trailing-hwpx/overlay/overlay_001.png`
- visual_accuracy_proxy_percent:
  - HWP page 1: 3.75233
  - HWPX page 1: 3.75233

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 3.75%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

보조 일치율은 폰트/선 굵기/잉크 형태 차이 때문에 낮다. review PNG 기준으로는 표 전체 위치와 행 높이
흐름이 기준 PDF와 같은 구조이며, 이 PR의 핵심인 row0 선언높이 유지 목적에는 맞는다.

## 샘플 처리 메모

메인터너/콜라보레이터가 검증 근거로 추가한 기준 PDF/HWPX 샘플은 옵션 1 처리에 따라 이 PR 에 함께
포함한다.

- `samples/task1763/cell_trailing_ls_expand-2024.pdf`
  - 한컴 2024 기준 PDF
- `samples/task1763/cell_trailing_ls_expand.hwpx`
  - HWPX 경로 visual sweep 검증용 샘플

기존 PR diff의 HWP 재현 샘플과 함께 HWP/HWPX 양쪽 경로를 기준 PDF에 대조했다.

## 결론

PR 내용과 최신 CI 기준으로 merge 후보다. #1761, #1762 조사 PR을 먼저 처리한 뒤 #1764를 이어서 처리하는
순서가 맞다.
