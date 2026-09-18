---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7261 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

저장 host를 기준으로 한 `(표 윗변 offset, 아래 점유 끝)`을 `stored_topbottom_object_span`으로
한 곳에서 계산한다. 예약 조건은 점유 끝을 소비하고, 최종 `saved_top`은 같은 윗변 offset을
소비한다. 빈 host·비인라인 TopAndBottom·저장 사다리 수용 범위는 유지한다.

- 기준 PDF p28 표 괘선 y=671.27px. 실제 표 윗변은 674.84→671.07px로 이동해
  차이가 약 3.57→0.20px로 줄었다. export-render-tree의 소수 첫째 자리 출력은 671.1px다.
- 기존 허용치 4.2px를 **0.5px**로 강화했다. 바깥여백 한 개를 오차로 허용하지 않는다.
- 실제 HWP IR에서 위·아래 여백을 283/0, 0/0, 141/0HU로 바꾼 독립 대조를 추가했다.
  입력을 중복 파일로 저장하지 않고 메모리에서 변경한다.
- 수정 전 **2/2 실패**, 수정 후 **2/2 통과**. 위여백은 실제 원점, 아래여백은 예약만 바꾼다.
- 별도 분할 표 경로 `issue_7203_split_float_anchors_to_paragraph_top` **3/3 통과**.
- Native Skia 포함 CLI build 통과. fresh WASM build 및 p27~29 Visual Sweep도 완료했다. Native p27·29 PNG는 수정 전과 bytes가 동일하다.

#7203의 다른 anchor 경로는 이번 수정 범위가 아니므로 이슈 전체 종료 근거로 사용하지 않는다.
아래 최초 검토 수치와 이미지는 수정 전 기록이다.

## 전체 검증에서 발견한 추가 회귀

통합 `227a31dfd`의 전체 nextest는 **10,071 PASS / 1 FAIL / 50 skipped**였다.
`body_overflow_baseline`이 기존 `samples/issue6111/56345_regulatory_impact_analysis.hwp`의
본문 하단 넘침 0→1건을 검출했다. #7261 위여백 보정 전후 CLI로 원인을 좁혔다.
1×1 빈 표의 저장 사다리가 선언 높이+위·아래 여백 전체를 증언하는 경우, host는 흐름 원점이다.
여기서 위여백을 빼면 최종 paint의 문단 원점과 lane 예약 원점이 달라져 후속 표가 밀린다.
실제로 마지막 표는 y=721.2→740.7, bottom=1062.48로 본문 하단1046.91을 15.57px 넘었다.

`stored_topbottom_object_span`이 저장 advance의 정확한 등식으로 흐름 원점과 위여백 뒤의
anchor를 구분하도록 보정한다. 임의 문서명·픽셀 허용치·baseline 완화는 추가하지 않는다.
최종 표의 본문 수용 반례는 수정 전 실패했다. 단순히 선언 높이 대신 측정 높이로 fit을
검사한 첫 시도는 실패를 해소하지 못해 폐기했다. 수정 후 앵커 3/3, 별도 분할 표 3/3, 본문 넘침 16/16 검사가 통과했다.
최종 표 y는 740.7→721.2px로 복원됐고 본문 하단 넘침은 1→0건이다.
Native/fresh WASM 각19·20쪽을 직접 비교했으며 rhwp PNG는 2/2 동일하다.
19쪽 대조는 유지되고 20쪽 후속 표 위치가 회귀 전으로 복원됐다. 마지막 행의 글꼴·줄바꿈과
외곽선 차이 및 rhwp20쪽/한컴21쪽은 기존 차이로 남는다. 이 전체 쪽수 차이를 해소했다고
보고하지 않는다. 본문 넘침 래칫 허용치는 변경하지 않았다.

기준은 기존 HWP에서 engine2020으로 `start → status(succeeded,17초) → download`했다.
MCP job `3f2da355-b139-427a-a67e-4c773a6a195d`, 21쪽·369,330bytes.
[원본 HWP](../../../samples/issue6111/56345_regulatory_impact_analysis.hwp) SHA-256
`58013017c3a3dc7e2d278b99c5b4fa1c61de0aa861f913a2c41a49145baadafc`,
[기준 PDF](../../../pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf) SHA-256
`c66ea20c3b8d6b31af73752e170372bc7af839c38812991125bdde4dfba35abb`.
PDF 추가에 따라 oracle_page_count 원장에 `21 / 20` 행을 추가했다.
독립 devel CLI와 보정 CLI가 모두20쪽이며 기존 행의 허용치를 완화하지 않았다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 19 | [compare](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_compare_019.png) · [overlay](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_overlay_019.png) · [review](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_review_019.png) | [compare](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_compare_019.png) · [overlay](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_overlay_019.png) · [review](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_review_019.png) |
| 20 | [compare](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_compare_020.png) · [overlay](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_overlay_020.png) · [review](../assets/pr7261_review/maintainer_anchor_flow_20260918/native_anchor_fit_review_020.png) | [compare](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_compare_020.png) · [overlay](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_overlay_020.png) · [review](../assets/pr7261_review/maintainer_anchor_flow_20260918/wasm_anchor_fit_review_020.png) |

[추가 보정 전20쪽 review](../assets/pr7261_review/maintainer_anchor_flow_20260918/before_native_anchor_fit_review_020.png) · [추가 보정 전 overlay](../assets/pr7261_review/maintainer_anchor_flow_20260918/before_native_anchor_fit_overlay_020.png).

## 보정 후 Visual Sweep 증적

p28의 목표 표 윗변이 PDF 괘선과 겹치는 것을 확인했다. 기존 가로 위치·글꼴 차이는 남아 있으며
다른 anchor 경로까지 완전히 일치했다고 주장하지 않는다. Native/WASM 각각 105쪽을 유지한다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 27 | [review](../assets/pr7261_review/maintainer_20260918/native_review_027.png) · [overlay](../assets/pr7261_review/maintainer_20260918/native_overlay_027.png) | [review](../assets/pr7261_review/maintainer_20260918/wasm_review_027.png) · [overlay](../assets/pr7261_review/maintainer_20260918/wasm_overlay_027.png) |
| 28 | [review](../assets/pr7261_review/maintainer_20260918/native_review_028.png) · [overlay](../assets/pr7261_review/maintainer_20260918/native_overlay_028.png) | [review](../assets/pr7261_review/maintainer_20260918/wasm_review_028.png) · [overlay](../assets/pr7261_review/maintainer_20260918/wasm_overlay_028.png) |
| 29 | [review](../assets/pr7261_review/maintainer_20260918/native_review_029.png) · [overlay](../assets/pr7261_review/maintainer_20260918/native_overlay_029.png) | [review](../assets/pr7261_review/maintainer_20260918/wasm_review_029.png) · [overlay](../assets/pr7261_review/maintainer_20260918/wasm_overlay_029.png) |

| 제품 | SHA-256 |
| --- | --- |
| Native | `be9d03277e5c2acd243a66b4df471eac1a6f522a2d37f2a3ebac35226c585674` |
| WASM | `804fc6cfbb66dac4ad9bf604db1090bb12883f06bed72b68e197ab5fc8a0fb39` |

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다.
[통합 PR #7264](https://github.com/edwardkim/rhwp/pull/7264)의 code candidate
`8228249fcfdf04cb7c47af9b3f5d5447a645d3b9` CI를 모두 확인했다.
[원격 CI 증적](pr_7244_review_impl.md#통합-pr-7264-code-candidate-ci)의 동일 PR 실행이며,
같은 PR의 trailing review·오늘할일 head aggregate와 mergeability를 확인한 뒤 merge한다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7261](https://github.com/edwardkim/rhwp/pull/7261) — fix: 자리차지 표의 윗변을 앵커 저장 자리에 둔다 (#7203 −13px 무리) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `c242af317d074a9923c7971e2e004120b4088574` |
| 규모 | 2 files, +93 / -3 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `c242af317d074a9923c7971e2e004120b4088574` | `a0859738be968207ccda57e7840fc66c8e02d14e` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35303610599) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35303610601) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35303610431) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35303610595) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35303610618) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35303610451)

## 범위·조판 계약 검토

관련 이슈: [#7203](https://github.com/edwardkim/rhwp/issues/7203). 자리차지 표의 저장 anchor 수용 조건에서 필요 높이를 height + bottom - top으로 계산한다.

native_empty_single_topbottom_table_saved_top의 stored_ladder_leaves_object_room → saved_top 수용 → para_y_for_table → 실제 표 윗변을 추적했다. gate만 바꾸고 마지막 원점은 그대로 두므로 AGENTS의 최종 덮어쓰기·원점 확인 조건이 남는다.

주요 소비 경로: [src/renderer/layout.rs](../../../src/renderer/layout.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_7203_float_table_top_uses_stored_anchor`: **1 test run: 1 passed, 205 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

27~29쪽 비교에서 28쪽 표가 앞 문단 글자를 가르는 현상은 사라졌다. 표 윗변은 PDF보다 아래이며 기존 다른 anchor 경로의 편차도 해결된 것은 아니다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| float_anchor | 27, 28, 29 | 0 / 17.22% | 0 / 17.22% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| float_anchor p27 | [compare](../assets/pr7261_review/native_float_anchor_compare_027.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_027.png) · [review](../assets/pr7261_review/native_float_anchor_review_027.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_027.png) · [overlay](../assets/pr7261_review/maintainer_20260918/wasm_overlay_027.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_027.png) |
| float_anchor p28 | [compare](../assets/pr7261_review/native_float_anchor_compare_028.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_028.png) · [review](../assets/pr7261_review/native_float_anchor_review_028.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_028.png) · [overlay](../assets/pr7261_review/wasm_float_anchor_overlay_028.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_028.png) |
| float_anchor p29 | [compare](../assets/pr7261_review/native_float_anchor_compare_029.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_029.png) · [review](../assets/pr7261_review/native_float_anchor_review_029.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_029.png) · [overlay](../assets/pr7261_review/wasm_float_anchor_overlay_029.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_029.png) |

기존 devel 제품 대조: [base float_anchor p28](../assets/pr7261_review/base_float_anchor_review_028.png).

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7261_review/maintainer_20260918/`: `native_review_027.png`, `native_overlay_027.png`, `wasm_review_027.png`, `wasm_overlay_027.png`, `native_review_028.png`, `native_overlay_028.png`, `wasm_review_028.png`, `wasm_overlay_028.png`, `native_review_029.png`, `native_overlay_029.png`, `wasm_review_029.png`, `wasm_overlay_029.png`를 실제 이미지로 포함한다.
- `mydocs/pr/assets/pr7261_review/maintainer_anchor_flow_20260918/`: `native_anchor_fit_review_019.png`, `native_anchor_fit_overlay_019.png`, `wasm_anchor_fit_review_019.png`, `wasm_anchor_fit_overlay_019.png`, `native_anchor_fit_review_020.png`, `native_anchor_fit_overlay_020.png`, `wasm_anchor_fit_review_020.png`, `wasm_anchor_fit_overlay_020.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#7203 OPEN 유지. 나머지 +10px 무리와 다른 줄 누락 축은 해결하지 않았다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
