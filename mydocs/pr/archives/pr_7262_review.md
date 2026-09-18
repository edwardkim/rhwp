---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7262 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P1] 중복 소유는 개선됐지만 33쪽 첫 조각의 마지막 '위한 교육 1' 줄이 clip 하단을 약 3.2px 넘는다. 직접 overlay와 SVG glyph-band 원장에서 확인했다. 한컴 16/1줄 분할 대신 14/3줄을 유지해 남은 밴드가 실제 내용 높이를 담지 못한다.

같은 유닛 창의 실제 요구 높이와 예약 밴드를 맞추고 32→33→34쪽의 누락·중복·마지막 글자·다음 행을 확인한다. clip 완화만으로 숨기지 않는다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7262](https://github.com/edwardkim/rhwp/pull/7262) — 수정: 걸침 전용 행을 이어받는 조각이 소비한 유닛부터 잇는다 (#7226) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `2afaa71a1cf8bb1a7ffe0f46f481ac8df89d4afe` |
| 규모 | 6 files, +266 / -4 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `cfbab066c68c03285365c628685b0e02019717b4` | `f1da7e4e334c6ce73fbf4e312b8d18081f911c9c` |
| `2afaa71a1cf8bb1a7ffe0f46f481ac8df89d4afe` | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35308105628) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35308105588) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35308105385) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35308105890) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35308105611) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35308105429)

## 범위·조판 계약 검토

관련 이슈: [#7226](https://github.com/edwardkim/rhwp/issues/7226). rowspan 셀만 있는 행의 빈 RowCut 이어받기에서 이미 소비한 유닛을 재출력하지 않는다.

start_row_height_override → resumes_inside_own_start_row → rowbreak_straddle_cut_units의 su → cell_line_ranges_from_cut → 실제 paint. 내용 시작은 공유하지만 typeset의 컷·요구 높이·예약 예산은 바꾸지 않았다. 따라서 소유 수정만으로 종료 조건(내용·물리 점유 보존)을 충족하지 않는다.

주요 소비 경로: [src/renderer/layout/table_layout.rs](../../../src/renderer/layout/table_layout.rs), [src/renderer/layout/table_partial.rs](../../../src/renderer/layout/table_partial.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `issue_7226_rowspan_only_row_cut`: **3 tests run: 3 passed, 204 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

32~34쪽을 대조했다. 33쪽 상단 중복은 제거됐으나 마지막 줄 일부와 표/후속 행 원점 차이가 남는다. 전체는 rhwp 413쪽·기준 PDF 415쪽이며 대상 구간의 쪽 오프셋은 0이다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| rowspan | 32, 33, 34 | 0 / 11.12% | 0 / 11.12% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| rowspan p32 | [compare](../assets/pr7262_review/native_rowspan_compare_032.png) · [overlay](../assets/pr7262_review/native_rowspan_overlay_032.png) · [review](../assets/pr7262_review/native_rowspan_review_032.png) | [compare](../assets/pr7262_review/wasm_rowspan_compare_032.png) · [overlay](../assets/pr7262_review/wasm_rowspan_overlay_032.png) · [review](../assets/pr7262_review/wasm_rowspan_review_032.png) |
| rowspan p33 | [compare](../assets/pr7262_review/native_rowspan_compare_033.png) · [overlay](../assets/pr7262_review/native_rowspan_overlay_033.png) · [review](../assets/pr7262_review/native_rowspan_review_033.png) | [compare](../assets/pr7262_review/wasm_rowspan_compare_033.png) · [overlay](../assets/pr7262_review/wasm_rowspan_overlay_033.png) · [review](../assets/pr7262_review/wasm_rowspan_review_033.png) |
| rowspan p34 | [compare](../assets/pr7262_review/native_rowspan_compare_034.png) · [overlay](../assets/pr7262_review/native_rowspan_overlay_034.png) · [review](../assets/pr7262_review/native_rowspan_review_034.png) | [compare](../assets/pr7262_review/wasm_rowspan_compare_034.png) · [overlay](../assets/pr7262_review/wasm_rowspan_overlay_034.png) · [review](../assets/pr7262_review/wasm_rowspan_review_034.png) |

기존 devel 제품 대조: [base rowspan p33](../assets/pr7262_review/base_rowspan_review_033.png).

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- rowspan: 위 p32, p33, p34의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7262_review/wasm_rowspan_overlay_032.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#7226 및 #6981 전체 종료 금지. 중복 감소와 분할·clipping 미해결을 분리한다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
