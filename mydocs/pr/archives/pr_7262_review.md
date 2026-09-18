---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7262 검토

## 최종 판정

**메인터너 보정 후 수용 가능 — 같은 행 이어받기의 예약·clip 보류 사유 해소, 최종 통합 게이트 대기.**

`7215dac37` 이후 보정은 `resumes_inside_own_start_row`를 컷 생산·paint뿐 아니라
`straddle_continuation_demand`의 예약 대상 선택에도 공유한다. `start_row=0`도 같은
물리 밴드가 있는 경우를 배제하지 않는다. 선택된 `rowbreak_straddle_cut_units`의
`su/eu`를 `cell_cut_visible_height`에 전달하므로 이미 소비한 앞 유닛을 재예약하지 않는다.

소비 경로는 다음과 같다.

| 단계 | 실제 소비 지점 |
| --- | --- |
| 남은 행 밴드 → 컷 | `resumes_inside_own_start_row` → `rowbreak_straddle_cut_units` |
| 컷 → 요구 높이 | `straddle_continuation_demand` → `cell_cut_visible_height` |
| 누적 예약·예산 실패 | `typeset.rs`의 rowspan 행/일반 행 두 호출: `need - consumed - cs_before`, fit 실패 시 컷/이월 |
| 실제 조각 | `table_partial.rs`의 같은 demand 소비 → 행 높이·셀 clip·뒤 행 위치 |
| 종료 | typeset의 terminal cut 검사도 같은 demand 호출, 기존 다음 내용/빈 쪽 검사 유지 |

- 수정 전 마지막 줄 바닥 **184.04px > 셀 바닥 180.87px**: 4개 중 해당 1개 실패.
  수정 후 셀 바닥 약 **187.8px**로 실제 패딩까지 예약하여 전체 줄이 표시된다.
- 실제 문서의 중복 없음·내용 보존·쪽수·마지막 줄/뒤 행/본문 검사와 ±1px 본문 예산의
  대상 이어받기 반례를 포함하여 **5/5 통과**. 예산 반례는 원본 표 IR을 사용한 계약 검사이며
  새 한컴 생성본이나 전 문서 PDF 일치 증거가 아니다.
- #6981 **8/8**(41개 예산 변형·빈 물리 밴드·끝 컷·종료 포함),
  정상 86712 관련 #7243 **2/2 통과**. clip을 풀거나 본문 바닥을 늘리지 않았다.
- Native/fresh WASM 32~34쪽 및 정상 86712 26·28·29쪽 재캡처, **6/6 PNG 동일**.
  정상 86712는 최초 통합 제품의 세 쪽 PNG와도 동일하다.
- 32쪽은 변경 전과 동일하다. 추가 예약으로 뒤 행의 한 줄이 33→34쪽으로 이동했고,
  32~34쪽 TextLine 집합은 수정 전후 누락·추가·중복 없이 같다(39/46/20 → 39/45/21줄).
  마지막 `위한 교육 1`의 실제 표시와 후속 행을 직접 확인했다.
- 413쪽 유지. 한컴 PDF 415쪽 및 대상 행 16/1 대 rhwp 14/3의 분할 차이는 남는다.
  이 보정은 내용 소유·예약·clipping 해소이며 전체 페이지 분할·글꼴 일치가 아니다.
  #7226·#6981 전체 종료를 주장하지 않는다.

Native SHA-256 `9856eaa6c4fd156eb76634e2a9629e12d61f7e832bf04227a608ebf394af9b84`, fresh WASM `3e61f63a54c94212cbca6864dbbeb41e6c3690c4edcc3053ecf0dbd09ed3e52a`.
Native CLI는 focused nextest의 release-test/native-skia 산출물이고 WASM은 별도 `--no-opt`
빌드다. [공동 기록](pr_7244_review_impl.md)의 명령 형식과 전용 target을 사용했다.
전체 nextest·lint·Native Skia 최종 게이트는 아직 미실행이다.

## 보정 후 Visual Sweep

| 입력·쪽 | Native SVG/Chrome | fresh WASM SVG/Chrome |
| --- | --- | --- |
| rowspan p32 | [compare](../assets/pr7262_review/maintainer_20260918/native_rowspan_compare_032.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_rowspan_overlay_032.png) · [review](../assets/pr7262_review/maintainer_20260918/native_rowspan_review_032.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_compare_032.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_overlay_032.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_review_032.png) |
| rowspan p33 | [compare](../assets/pr7262_review/maintainer_20260918/native_rowspan_compare_033.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_rowspan_overlay_033.png) · [review](../assets/pr7262_review/maintainer_20260918/native_rowspan_review_033.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_compare_033.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_overlay_033.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_review_033.png) |
| rowspan p34 | [compare](../assets/pr7262_review/maintainer_20260918/native_rowspan_compare_034.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_rowspan_overlay_034.png) · [review](../assets/pr7262_review/maintainer_20260918/native_rowspan_review_034.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_compare_034.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_overlay_034.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_rowspan_review_034.png) |
| corrected_86712 p26 | [compare](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_compare_026.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_overlay_026.png) · [review](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_review_026.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_compare_026.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_overlay_026.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_review_026.png) |
| corrected_86712 p28 | [compare](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_compare_028.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_overlay_028.png) · [review](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_review_028.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_compare_028.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_overlay_028.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_review_028.png) |
| corrected_86712 p29 | [compare](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_compare_029.png) · [overlay](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_overlay_029.png) · [review](../assets/pr7262_review/maintainer_20260918/native_corrected_86712_review_029.png) | [compare](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_compare_029.png) · [overlay](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_overlay_029.png) · [review](../assets/pr7262_review/maintainer_20260918/wasm_corrected_86712_review_029.png) |

아래 Metadata 이후는 **수정 전 기록**이다. merge 후 contributor/issue comment에는
이 절의 최신 review와 **모든 standalone overlay**를 실제 이미지로 포함한다.

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
