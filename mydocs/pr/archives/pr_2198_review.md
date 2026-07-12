# PR #2198 검토 - Task #2070 80168 페이지네이션 157쪽 정합 세트

## 메타

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/2198 |
| 작성자 | planet6897 |
| base | devel |
| head | task2070-pagination-set |
| 문서 작성 시점 head | 5ca966bf4c30b13b4c277a3f57f0e50c3326e888 |
| merge commit | 1cb01f1ace2945ea68c04a2e3b69f62684806d56 |
| 규모 | 41 files, +3075/-149 |
| CI | 문서 작성 시점 GitHub Actions all pass |
| #2070 상태 | merge 후 자동 close 확인, maintainer reopen 완료 |
| #2070 reopen 코멘트 | https://github.com/edwardkim/rhwp/issues/2070#issuecomment-4944141781 |

## 변경 범위 요약

PR은 80168 규제영향분석서의 HWP/HWPX 페이지 수를 한글 2022 PDF 기준 157쪽으로 맞추기 위해 다음 축을 한 번에 반영한다.

- 0값 LINE_SEG 정규화
- 비-TAC 표 선언높이 플로어
- NO_LS 빈 문단 em 줄박스
- 셀 재래핑 규칙 보강
- U+318D 아래아 폭/스크립트 분류
- RowBreak/CellBreak 분할 재시도
- 관련 샘플, 기준 PDF, 실측 도구, 보고서 추가

렌더러, 레이아웃, 텍스트 측정, parser normalize 경로가 바뀌므로 visual sweep 대상이다.

## 로컬 검증

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 수행했다.

| 검증 | 결과 |
|---|---|
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1842 --test issue_1891 --test issue_1939 --test svg_snapshot` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check upstream/devel...codex/pr-2198-review` | PASS |
| `pdfinfo pdf/80168_regulatory_analysis-2022.pdf` | 157 pages, Hwp 2022 / Hancom PDF |
| `target/release-test/rhwp dump-pages samples/80168_regulatory_analysis.hwp` | 157 pages |
| `target/release-test/rhwp dump-pages samples/issue1891/80168_regulatory_analysis.hwpx` | 157 pages |
| `target/release-test/rhwp dump-pages samples/issue2063_huge_cellbreak_table.hwp` | 159 pages |
| `target/release-test/rhwp dump-pages samples/76076_regulatory_analysis.hwp` | 83 pages |
| `target/release-test/rhwp dump-pages samples/86712_regulatory_analysis.hwp` | 64 pages |

## Issue #2070 해소 여부

#2070은 원문 기준으로 `21914299` 화성시 별표2 초대형 CellBreak 표의 `rhwp 213p vs 한글 162p`
과분할을 추적한다. 댓글에서는 같은 행높이 과대측정 계열의 RowBreak 변종도 #2070 scope로
확장되었다. 따라서 PR #2198의 `80168=157` 성공만으로 #2070 전체 해소를 판단하면 안 된다.

| #2070 항목 | 기준 | 현재 확인 | 판단 |
|---|---:|---:|---|
| 원문 CellBreak 타깃 `samples/issue2063_huge_cellbreak_table.hwp` | 한컴 PDF 162쪽 | rhwp 159쪽 | 큰 개선이나 아직 -3쪽 |
| 기준 PDF `pdf/issue2063_huge_cellbreak_table-2020.pdf` | 162쪽 | 162쪽 확인 | 기준 자료 존재 |
| 댓글 확장 RowBreak 변종 `D0150004-1-002` | 한컴 315쪽 | 최신 #2070 댓글상 rhwp 717쪽(+402 잔존) | 미해소 |
| PR #2198 핵심 타깃 `80168_regulatory_analysis` | 한컴 2022 PDF 157쪽 | HWP/HWPX 모두 157쪽 | 이 축은 해소 |
| 잔여 핀 `76076`, `86712` | 82쪽 / 65쪽 | 83쪽 / 64쪽 | 기준 PDF와 불일치 |

즉, 이 PR은 #2070과 연결된 상쇄망 중 `80168` 축은 해결하지만, #2070 원문 재현과 댓글로 확장된
RowBreak 잔여까지 모두 닫지는 못한다. #2070을 close하려면 최소한 원문 CellBreak 기준을
"162쪽 정확"이 아니라 "159쪽까지 개선 후 잔여 -3쪽은 별도 저우선"으로 메인테이너가 명시적으로
재정의하거나, 잔여 -3쪽 및 RowBreak 변종을 별도 이슈로 분리해야 한다.

## Visual sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2198_80168 \
  --hwp samples/80168_regulatory_analysis.hwp \
  --pdf pdf/80168_regulatory_analysis-2022.pdf \
  --page 1 --page 108 --page 157 \
  --rhwp-bin target/release-test/rhwp \
  --out output/pr2198_visual
```

결과:

- SVG/render-tree/PDF 모두 157 pages.
- 선택 페이지 1, 108, 157 중 108쪽 1개가 flagged.
- 108쪽 플래그: `render_tree_frame_tail_overflow`, `line_band_drift`, `column_line_band_drift`.
- 산출 요약: `output/pr2198_visual/summary.json`.
- 대표 증적: `output/pr2198_visual/pr2198_80168/review/review_108.png`, `output/pr2198_visual/pr2198_80168/analysis/annotated_108.png`.

해석: 80168 페이지 수 정합은 확인되지만, PR이 마지막 축으로 설명한 p108 조문대비표 분할 지점에는 visual sweep 후보가 남는다. 이 후보만으로 PR 목적 달성 자체를 부정하지는 않지만, 잔여로 기록해야 한다.

## 후속 보완 요청

### [Follow-up] PR이 `closes #2070`을 선언하지만 #2070 본문 타깃은 아직 완전히 해결되지 않았다

#2070 본문은 `21914299` 화성시 별표2 초대형 CellBreak 표의 `rhwp 213p vs 한글 162p` 과분할이다. PR #2198의 본문과 보고서는 주로 `80168_regulatory_analysis` 157쪽 정합을 설명하며, 로컬 확인 결과 #2070 원문 샘플인 `samples/issue2063_huge_cellbreak_table.hwp`는 159쪽이다.

159쪽은 기존 213쪽 과분할 대비 큰 개선이고 `tests/issue_1842.rs`의 넓은 회귀 범위(150..=175)도 통과한다. PR 본문이 주장한 80168 개선 요지는 맞으므로 merge 자체는 수용할 수 있다. 다만 기준 162쪽과는 여전히 -3쪽이고, 이슈 댓글의 RowBreak 변종 잔여도 PR 범위 안에서 닫히지 않는다. 따라서 merge 후에도 #2070 전체 close는 보류하거나, PR 본문의 `closes #2070`을 제거하고 잔여 CellBreak -3쪽 및 RowBreak 변종을 후속 이슈로 분리하는 것이 맞다.

### [Follow-up] 기준 PDF와 어긋나는 페이지 수를 테스트 기대값으로 승격하고 있다

`tests/issue_1891.rs`와 `tests/issue_1939.rs`는 `76076`의 기준 PDF가 82쪽임을 주석으로 인정하면서 기대값을 83으로 바꾼다. `86712`도 기준 65쪽에서 HWP 64, HWPX 63으로 낮춘다. 주석에 후속 복귀 조건이 있기는 하지만, 테스트 자체는 known-wrong 값을 pass 조건으로 고정한다.

이 상태의 CI green은 "공식 PDF 핀 유지"를 보장하지 못한다. merge 수용과 별개로, `76076/86712`의 기준 PDF 불일치가 의도된 임시 후퇴임을 PR 본문 또는 후속 이슈에 명확히 남기고, #2195/#2197 계열 보완에서 기준 PDF 기대값을 되돌리는 추적이 필요하다.

### [Follow-up] `tests/golden_svg/form-002/page-0.actual.svg`는 로컬 실패 산출물이라 커밋 대상이 아니다

현재 저장소에는 `tests/golden_svg/**/*.actual.svg` 추적 파일이 없고, `tests/svg_snapshot.rs`도 mismatch 시 `.actual.svg`를 "local inspection" 용도로 쓰도록 되어 있다. PR이 `tests/golden_svg/form-002/page-0.actual.svg`를 새로 추가한 것은 snapshot 실패 산출물을 함께 커밋한 형태다. 의도한 변경은 `page-0.svg` 갱신만 남기고 `.actual.svg`는 제거해야 한다.

### [Follow-up] PR 범위와 무관한 task 문서가 섞여 있다

PR #2198에는 `mydocs/plans/task_m100_2136.md`, `task_m100_2136_impl.md`, `task_m100_2137.md`, `mydocs/working/task_m100_2110_stage4.md`, `task_m100_2136_stage1.md`, `task_m100_2138_stage1.md`, `mydocs/report/task_m100_2136_report.md` 등이 포함되어 있다. PR 제목과 본문은 #2070/80168 상쇄망인데, 이 문서들은 #2110/#2136/#2137/#2138 범위다.

이 문서들은 리뷰 가능한 변경 범위를 흐리고, merge 후 히스토리 추적도 어렵게 만든다. #2070에 직접 필요한 보고서와 도구만 남기고 다른 task 문서는 후속 정리 PR에서 분리하거나 제거하는 것이 좋다.

## 결론

**merge 수용**으로 판단한다. PR 본문의 기본 요지인 `80168_regulatory_analysis` HWP/HWPX 157쪽 정합은 로컬에서 재현됐고, PR에 포함된 기준 PDF도 157쪽이다. 선택 테스트, fmt, diff-check, GitHub CI도 통과했다.

다만 #2070 전체 close 근거로는 부족하다. `closes #2070` 때문에 merge 시 이슈가 자동 close되면
메인테이너가 다시 open하고, 아래 코멘트 방향으로 후속 보완을 요청한다.

```markdown
핵심 방향과 80168 개선은 확인했습니다. 로컬에서도 `80168_regulatory_analysis.hwp` / HWPX 모두 157쪽이고, 첨부 PDF도 157쪽으로 확인했습니다. CI도 통과했습니다.

다만 #2070 전체 close 근거로는 아직 부족해 보입니다.

- #2070 원문 타깃인 `issue2063_huge_cellbreak_table.hwp`는 기준 PDF 162쪽 대비 현재 rhwp 159쪽입니다. 213쪽 과분할에서 크게 개선된 것은 맞지만, 이슈 원문 기준은 아직 완전히 닫히지 않았습니다.
- #2070 댓글에서 확장된 RowBreak 변종(`D0150004-1-002`)도 최신 재측정 기준으로 잔여가 남아 있습니다.
- `76076/86712`는 기준 PDF 82/65쪽과 다른 83/64쪽을 테스트 기대값으로 승격하고 있어, CI green이 기준 PDF 핀 유지를 의미하지 않게 됩니다.
- `tests/golden_svg/form-002/page-0.actual.svg`는 snapshot 실패 시 생성되는 로컬 확인용 산출물이므로 제거가 필요합니다.
- #2070과 무관한 #2110/#2136/#2137/#2138 문서가 PR에 섞여 있어 범위 정리가 필요합니다.
- visual sweep상 80168 p108에는 `line_band_drift` / `render_tree_frame_tail_overflow` 후보가 남아 있으므로 잔여로 기록해 주세요.

또한 #2070 close 판단에 필요한 검증 문서가 PR에 충분히 포함되어 있지 않습니다. #2070 전체 해소를 주장하려면 다음 자료도 PR 또는 추적 가능한 보존 위치에 포함해 주세요.

- #2070 원문 타깃: `21914299` 화성시 별표2 HWP
- 해당 기준 PDF: 한컴 2020/2022 기준 162쪽 PDF
- #2070 댓글에서 확장된 RowBreak 변종: `1130000-201900011_D0150004-1-002_2017년도 세출구조조정.hwp`
- 해당 기준 PDF: 한컴 기준 315쪽 PDF
- 각 문서에 대한 현재 rhwp 페이지 수, 기준 PDF 페이지 수, 남은 차이

특히 원문 CellBreak 타깃은 로컬 저장소에 `samples/issue2063_huge_cellbreak_table.hwp`와 `pdf/issue2063_huge_cellbreak_table-2020.pdf`가 있어 검증 가능했지만, PR #2198에는 이 검증 축이 본문/보고서/테스트 게이트로 충분히 연결되어 있지 않습니다. RowBreak 변종은 제 로컬에서는 원본/기준 PDF를 찾지 못해 직접 재검증하지 못했습니다.

따라서 이 PR은 “80168 상쇄망 축 해결”로는 의미가 크지만, `closes #2070`은 제거하거나, #2070의 남은 CellBreak -3쪽 및 RowBreak 변종을 별도 후속 이슈로 분리한 뒤 다시 판단하는 것이 안전해 보입니다.

만약 현재 PR 본문의 `closes #2070` 때문에 merge 후 #2070이 자동 close되면, maintainer 측에서 다시 reopen하고 위 잔여 항목을 이어서 추적하겠습니다.
```
