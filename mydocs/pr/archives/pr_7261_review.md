---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7261 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P2] 수용 조건은 anchor가 위여백 뒤라고 가정하지만 최종 표 윗변은 여전히 anchor=674.80px이다. 독립 PDF 윗변 671.27px와 3.53px 다르며 테스트는 바깥여백 한 개를 허용해 통과한다. 측정 조건과 최종 원점의 같은 계약이 완결되지 않았다.

anchor와 바깥여백의 좌표 계약을 실제 배치까지 연결하고, 대칭·비대칭 여백 대조군 및 PDF 경계를 검증한다. 여백 크기를 허용 오차로 삼는 검사만으로 종료하지 않는다.

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
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
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

## 실행한 검증과 한계

- 통합 제품의 `issue_7203_float_table_top_uses_stored_anchor`: **1 test run: 1 passed, 205 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

27~29쪽 비교에서 28쪽 표가 앞 문단 글자를 가르는 현상은 사라졌다. 표 윗변은 PDF보다 아래이며 기존 다른 anchor 경로의 편차도 해결된 것은 아니다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| float_anchor | 27, 28, 29 | 0 / 17.22% | 0 / 17.22% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| float_anchor p27 | [compare](../assets/pr7261_review/native_float_anchor_compare_027.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_027.png) · [review](../assets/pr7261_review/native_float_anchor_review_027.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_027.png) · [overlay](../assets/pr7261_review/wasm_float_anchor_overlay_027.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_027.png) |
| float_anchor p28 | [compare](../assets/pr7261_review/native_float_anchor_compare_028.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_028.png) · [review](../assets/pr7261_review/native_float_anchor_review_028.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_028.png) · [overlay](../assets/pr7261_review/wasm_float_anchor_overlay_028.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_028.png) |
| float_anchor p29 | [compare](../assets/pr7261_review/native_float_anchor_compare_029.png) · [overlay](../assets/pr7261_review/native_float_anchor_overlay_029.png) · [review](../assets/pr7261_review/native_float_anchor_review_029.png) | [compare](../assets/pr7261_review/wasm_float_anchor_compare_029.png) · [overlay](../assets/pr7261_review/wasm_float_anchor_overlay_029.png) · [review](../assets/pr7261_review/wasm_float_anchor_review_029.png) |

기존 devel 제품 대조: [base float_anchor p28](../assets/pr7261_review/base_float_anchor_review_028.png).

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- float_anchor: 위 p27, p28, p29의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7261_review/wasm_float_anchor_overlay_027.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#7203 OPEN 유지. 나머지 +10px 무리와 다른 줄 누락 축은 해결하지 않았다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
