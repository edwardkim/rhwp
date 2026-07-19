# PR #2036 리뷰 — #2020 렌더링 차이 재현 자산과 회귀 게이트 보강

- PR: #2036 `task 2020: 렌더링 차이 재현 자산과 회귀 게이트 보강`
- URL: https://github.com/edwardkim/rhwp/pull/2036
- 기준 브랜치: `devel`
- head branch: `task/m100-2020-rendering-diff-suite`
- 문서 작성 시점 참고 코드 커밋: `ecf2c8247`
- 관련 이슈: #2020
- 검토 경로: 내부 task PR + 옵션 1. review 문서, visual asset, 오늘할일을 PR head 에 함께 포함한다.
- 최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. 다만 이 PR 은 #2020 전체를 닫는 PR 이 아니다. FSC HWP/HWPX 흐름 일부, 국립국어원 3쪽
하단 이월, SVG 첨자 출력, 재현 자산 보존과 회귀 게이트를 먼저 고정한다. 여권신청서, 복학원서, 폰트/자간,
특정 기호 뒤 공백은 최신 검증에서도 잔여 차이가 있어 #2020 은 open 유지가 맞다.

PR 본문도 `Refs #2020` 으로 작성되어 있고 `Closes #2020` 는 넣지 않았다.

## 변경 요약

- `src/renderer/typeset.rs`: 빈 host 문단의 자리차지 RowBreak 표가 저장 LineSeg와 실제 객체 높이 기준으로
  현재 쪽 본문 하단에 들어가는 경우, 선언 높이의 근소 초과만으로 조기 이월하지 않도록 보정했다.
- `src/renderer/svg.rs`: SVG 백엔드의 위첨자/아래첨자 glyph 크기와 baseline 을 Canvas/HTML 출력과 맞췄다.
- `tests/issue_2020.rs`: #2020 문서군 페이지 수와 FSC HWP 2쪽 하단 14x15 표 위치를 회귀 테스트로 고정했다.
- `samples/issue2020/`: 이슈 본문 원본 HWP/HWPX, 본문 스크린샷 PNG 13개, 비교 보고서 PDF 를 보존했다.
- `pdf/issue2020/`: MCP/Hancom 기준 PDF 를 장기 검증 기준으로 보존했다.
- `.github/workflows/*`: 신규 `samples/**/*.png` 를 review reference fast-pass 허용 목록에 추가했다.
- `mydocs/manual/pr_review_workflow.md`: issue/PR 본문 첨부 파일은 `samples/` 아래 보존하도록 절차를 보강했다.

## 첨부 문서 반영

이슈 본문과 외부 링크에서 확인한 재현 자산은 다음 위치에 보존했다.

| 항목 | 보존 경로 |
|------|-----------|
| 금감원 보도자료 HWP/HWPX | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp`, `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx` |
| 여권발급신청서 HWP | `samples/issue2020/passport_application_lawgo.hwp` |
| 이슈 본문 스크린샷 | `samples/issue2020/issue2020_expected_*.png`, `samples/issue2020/issue2020_actual_*.png` |
| 이슈 본문 비교 PDF | `samples/issue2020/issue2020_image_rendering_parsing.pdf` |
| 기준 PDF 보존 | `pdf/issue2020/*.pdf` |

이슈 본문 비교 PDF 와 `pdf/issue2020/issue2020_comparison_report.pdf` 의 SHA-256 은
`7e9a50873fd255519dfb1ebb92cd146f3f6f4ffe8f4221e66a6edc5e5f8dc794` 로 동일하다.

## 로컬 검증

- `cargo fmt --check`: 통과.
- `git diff --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2020 -- --nocapture`: 통과, 2 passed.
- `CARGO_INCREMENTAL=0 cargo test renderer::svg::tests::test_svg_draw_text_superscript_adjusts_baseline_and_size -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- 기존 `http://localhost:7700/` Vite 서버에서 `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp` 로드 확인: 5쪽,
  Canvas 793x1122, 브라우저 console/page error 0건.

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 대표 페이지를 확인했다.

| 항목 | 기준 PDF | 결과 | 대표 asset |
|------|----------|------|------------|
| FSC HWP 1쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` | 5/5쪽, 플래그 0, pixel match 88.93448%, proxy 9.38671% | 참고: 임시 `output/issue2020/visual_fsc_hwp_p1_latest/.../review/review_001.png` |
| FSC HWP 2쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` | 5/5쪽, `render_tree_frame_tail_overflow` 1건. 후보는 페이지 번호 `-2-`. 핵심 표는 2쪽 유지 | `mydocs/pr/assets/pr_2036_issue2020_fsc_hwp_review_p002.png` |
| FSC HWPX 1쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` | 5/5쪽, 플래그 0, pixel match 88.93448%, proxy 9.38671% | 참고: 임시 `output/issue2020/visual_fsc_hwpx_p1_latest/.../review/review_001.png` |
| 국립국어원 3쪽 | `pdf/2022년 국립국어원 업무계획-2022.pdf` | 35/35쪽, 플래그 0, pixel match 90.20470%, proxy 35.29146% | `mydocs/pr/assets/pr_2036_issue2020_niklp_review_p003.png` |
| 여권신청서 1쪽 | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf` | 2/2쪽, `frame_overflow_pixels`, `render_tree_frame_tail_overflow`, `line_band_drift`, `column_line_band_drift`, `large_ink_region_drift` 남음 | `mydocs/pr/assets/pr_2036_issue2020_passport_remaining_p001.png` |
| 복학원서 1쪽 | `pdf/issue2020/복학원서-2022.pdf` | 1/1쪽, `line_order_overlap` 1건. 원형/도형 위치 차이 남음 | `mydocs/pr/assets/pr_2036_issue2020_bokhak_remaining_p001.png` |

## 이슈 증상별 판정

| 이슈 본문 증상 | 현재 판정 |
|----------------|-----------|
| 글자 위/아래 간격이 좁아짐 | 미해결. 여권 공식 PDF 기준 line/column band drift 가 남는다. |
| 특정 기호 뒤 불필요 공백 | 미확정/잔여. 단독 회귀 테스트가 아직 없다. |
| 이미지 위 텍스트 겹침 | 일부 해결. FSC page count 와 하단 표 이월은 해결했지만 전체 시각 일치는 낮다. |
| 윗첨자가 일반 글자로 렌더링 | 해결. SVG 첨자 크기/baseline 테스트 통과. |
| 글꼴이 원본과 다름 | 미해결. FSC 기준 PDF 대비 폰트/굵기/자간 차이가 남는다. |
| 도형 또는 원형 표시 위치 차이 | 미해결. 복학원서 차이 남음. |
| 일부 객체가 회전된 것처럼 보임 | 미해결. 복학원서 원형/도형 차이 남음. |
| 같은 페이지 하단 내용이 다음 페이지로 밀림 | 해결. FSC HWP 5쪽 유지, 2쪽 하단 14x15 표 유지, 국립국어원 3쪽 플래그 0. |

## 리스크와 후속

- 이 PR 에 workflow fast-pass 허용 목록 변경이 포함되므로 문서/asset-only fast-pass PR 이 아니다. 최신 PR head
  기준 GitHub Actions 통과를 확인해야 한다.
- #2020 은 merge 후에도 close 하지 않는다. 남은 시각 차이는 같은 이슈에서 계속 추적한다.
- PR comment 또는 issue comment 를 남길 때는 위 대표 asset 을 `devel` 기준 raw GitHub 링크로 사용한다.
