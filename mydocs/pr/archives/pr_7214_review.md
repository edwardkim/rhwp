---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7214_review.md
last_verified: 2026-09-17
---

# PR #7214 검토

## 최종 판정

**승인 — 요청한 머지 보류 사유 해소(검토 변경 범위).** 중첩 표 제한값 조회와 크기 변경이 같은 CellPath를 사용함을 확인했다. 전체 회귀·Skia·lint·fresh WASM·Visual Sweep을 완료했다. 원격 CI/merge 승인과 문서 전체 PDF 일치 판정은 별개다.

## 메인터너 보정 (2026-09-17)

`c1c9e2047`에서 중첩 셀 제한 조회를 같은 셀 경로로 통일했다. native `get_cell_properties_by_cell_path_native` → WASM `getCellPropertiesByPath` → bridge → `clampCompensatedResizeDelta(TableRef)`로 전달되며 depth>1에서는 평면 조회를 사용하지 않는다.

Rust 6개, 실제 clamp 호출 Studio 3개(수정 전 2 FAIL / 1 PASS), TypeScript, Studio 전체 1758 PASS / 2 skip을 확인했다. 실제 3026219 브라우저 drag는 flat 조회 0회 / path 조회 12회, 바깥 표 불변, Undo 원복, page error 0건이었다. 최종 누적 head의 fresh WASM에서도 같은 실제 drag 결과를 다시 확인했다.

### 보정 후 증적

렌더링/UI 검증 코드 head는 `54c24ebddb1a578786a6eb082c40c493dcde07f1`이다. 이후 `f94dece59`는 테스트의 동등한 역방향 탐색 보정이며 fmt·전체 target Clippy·해당 2개 테스트를 재검증했다. [최종 공통 검증](pr_7210_review.md#메인터너-보정-최종-검증)에 실행 범위와 결과를 모았다.

- [maintainer_inner_table_wasm_compare_001.png](../assets/pr7214_review/maintainer_inner_table_wasm_compare_001.png)
- [maintainer_inner_table_wasm_overlay_001.png](../assets/pr7214_review/maintainer_inner_table_wasm_overlay_001.png)
- [maintainer_inner_table_wasm_review_001.png](../assets/pr7214_review/maintainer_inner_table_wasm_review_001.png)
- [maintainer_studio_inner-hover.png](../assets/pr7214_review/maintainer_studio_inner-hover.png)
- [maintainer_studio_inner-drag.png](../assets/pr7214_review/maintainer_studio_inner-drag.png)
- [maintainer_studio_partial-border.png](../assets/pr7214_review/maintainer_studio_partial-border.png)

### 보정 전 판정과 증거

**머지 보류** — 안쪽 표 drag·Undo는 작동하지만 제한값 조회가 여전히 바깥 표 API를 사용한다.

**이 아래의 코드 위치·수치·보류 판정·미실행 설명과 기존 PNG는 초기 검토 `cd074a4da`의 이력이다. 현재 판정은 문서 상단과 보정 후 증적을 따른다.**

[원 PR #7214](https://github.com/edwardkim/rhwp/pull/7214): 수정: 중첩 표의 셀 크기를 셀 경로로 조절한다 (#7189)
관련 [이슈 #7189](https://github.com/edwardkim/rhwp/issues/7189).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `fc55c1a138500007cabeffff3d1146621ca45161`, base `devel`. 검토자는 `jangster77`이다.
- source `fc55c1a138500007cabeffff3d1146621ca45161` → applied `542f2442036a71b1033deb9f78b94fd001f572d1`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35171051099/job/105042591379), [CI](https://github.com/edwardkim/rhwp/actions/runs/35171051184/job/105042591847), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35171048974/job/105042584099), [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/35171048981/job/105042584003), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35171051148/job/105042591901), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35171051159/job/105042591935), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35171051012/job/105042591154), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35172005170). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7214_review_impl.md).

## 코드 경로와 독립 실행 증거

hitTest.cellPath → TableRef.path → getTableCellBboxesByPath → resizeTableCellsByPath → resolve_table_mut_by_cell_path. 반면 input-handler-table.ts:183–215의 clampCompensatedResizeDelta는 getCellProperties(sec,ppi,ci,cellIdx)를 호출해 같은 소유 경로를 소비하지 않는다.

3026219의 depth=2 내부 열을 25px drag: 첫 셀 width 296.2→320.3px, 바깥 표 bbox 불변, Undo 후 내부 bbox 배열 원복. 같은 실행 중 getCellProperties(0,2,0,0)는 바깥 표 width=47909 HU를 반환했고, cellIdx 1/2/4/6/8/10은 범위 초과 오류가 발생했다. 예외는 catch로 무시됐다. 일반 drag 성공이 최소폭·보상폭 경계의 정확성을 입증하지 않는다.

관련 실행: **issue_7189_nested_table_resize_by_path 5개 및 실제 Canvas2D drag/Undo**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

실행 검출: 잘못된 대상 속성 조회와 범위 초과. 최소폭에서 실제 최종 문서가 손상된다는 주장까지는 검증하지 않았다. 외곽 클릭의 바깥 표 선택 가능성은 코드 검토상 우려이며 이번 probe에서는 drag가 시작되지 않아 재현으로 세지 않는다.

속성 조회도 동일한 cellPath로 연결하고, 서로 다른 외곽/내부 셀 폭에서 최소폭·인접 열 보상·Undo를 실제 Studio 및 정식 회귀로 확인한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 미충족 | hitTest.cellPath → TableRef.path → getTableCellBboxesByPath → resizeTableCellsByPath → resolve_table_mut_by_cell_path. 반면 input-handler-table.ts:183–215의 clampCompensatedResizeDelta는 getCellProperties(sec,ppi,ci,cellIdx)를 호출해 같은 소유 경로를 소비하지 않는다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 미검증 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [tests/fixtures/planet_review_20260917/3026219_[별지 3] 기관표준심의위원회 의견서(국토지리정보원 공간정보 표준화지침).hwpx](../../../tests/fixtures/planet_review_20260917/3026219_%5B%EB%B3%84%EC%A7%80%203%5D%20%EA%B8%B0%EA%B4%80%ED%91%9C%EC%A4%80%EC%8B%AC%EC%9D%98%EC%9C%84%EC%9B%90%ED%9A%8C%20%EC%9D%98%EA%B2%AC%EC%84%9C%28%EA%B5%AD%ED%86%A0%EC%A7%80%EB%A6%AC%EC%A0%95%EB%B3%B4%EC%9B%90%20%EA%B3%B5%EA%B0%84%EC%A0%95%EB%B3%B4%20%ED%91%9C%EC%A4%80%ED%99%94%EC%A7%80%EC%B9%A8%29.hwpx) | [pdf/planet-review-20260917/3026219-2020.pdf](../../../pdf/planet-review-20260917/3026219-2020.pdf) | 1 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [inner_table_wasm_compare_001.png](../assets/pr7214_review/inner_table_wasm_compare_001.png)
- [inner_table_wasm_overlay_001.png](../assets/pr7214_review/inner_table_wasm_overlay_001.png)
- [inner_table_wasm_review_001.png](../assets/pr7214_review/inner_table_wasm_review_001.png)
- [studio-inner-hover.png](../assets/pr7214_review/studio-inner-hover.png)
- [studio-inner-drag.png](../assets/pr7214_review/studio-inner-drag.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
