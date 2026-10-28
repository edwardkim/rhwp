---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7220_review.md
last_verified: 2026-09-17
---

# PR #7220 검토

## 최종 판정

**승인** — HWPX의 원래 HWP5 slot 축을 문단 근거로 판별해 중복 투영을 피하는 변경을 수용한다.

[원 PR #7220](https://github.com/edwardkim/rhwp/pull/7220): 수정: 저장 textpos 가 이미 HWP5 축인 HWPX 구역 첫 문단을 올려 읽지 않는다 (#7190)
관련 [이슈 #7190](https://github.com/edwardkim/rhwp/issues/7190).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `82225af0edf8204d3fa78aeccee7e648f816682e`, base `devel`. 검토자는 `jangster77`이다.
- source `06ceed2db48d56b53d63f1b64a7796a87cc1b9eb` → applied `b472bf515a5339068baac41e6d73c920646e74c3`
- source `82225af0edf8204d3fa78aeccee7e648f816682e` → applied `9d94231187d11d1ccd5b495d00bf8155eb11a26b`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35189529043/job/105098809560), [CI](https://github.com/edwardkim/rhwp/actions/runs/35189529237/job/105098810648), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35189527047/job/105098803860), [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/35189527151/job/105098803928), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35189529084/job/105098809780), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35189529212/job/105098810025), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35189528859/job/105098809367), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35190659574). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7220_review_impl.md).

## 코드 경로와 독립 실행 증거

문단 text_start 후보 → 원래/투영된 slot boundary와 끝 경계 증거 → 문단 단위 축 선택 → 레이아웃 줄 소속. 첫 줄과 정상 유효 축의 대조군을 유지한다.

3011411의 첨부파일 그림이 저장된 둘째 줄에 남고, 36473713은 선행 제어 이후 한컴과 같은 개행을 사용한다. 3011411 내용 픽셀 일치율 base 11.78→28.02%. Native/fresh WASM의 대상 PNG는 동일하다. 관련 10개 focused 검사 통과.

관련 실행: **issue_7190 2개 및 issue_5961 축 투영 대조군 8개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

36473713 상단의 깨진 그림 placeholder와 기존 글꼴 차이까지 해결한 것은 아니다. 원 이슈의 off-canvas 진단 사각지대는 이 변경과 별개이므로 자동 종료 문구는 실제 해결 범위에 맞춰야 한다.

최종 통합 CI 확인 및 #7190의 남은 진단 범위를 본문에 구분한 뒤 merge한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 충족 | 문단 text_start 후보 → 원래/투영된 slot boundary와 끝 경계 증거 → 문단 단위 축 선택 → 레이아웃 줄 소속. 첫 줄과 정상 유효 축의 대조군을 유지한다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/issue7190/3011411_tac_picture_second_line.hwpx](../../../samples/issue7190/3011411_tac_picture_second_line.hwpx) | [samples/issue7190/3011411_tac_picture_second_line-2020.pdf](../../../samples/issue7190/3011411_tac_picture_second_line-2020.pdf) | 1 |
| [samples/issue7190/36473713_leading_control_lines.hwpx](../../../samples/issue7190/36473713_leading_control_lines.hwpx) | [samples/issue7190/36473713_leading_control_lines-2020.pdf](../../../samples/issue7190/36473713_leading_control_lines-2020.pdf) | 1 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [axis_picture_wasm_compare_001.png](../assets/pr7220_review/axis_picture_wasm_compare_001.png)
- [axis_picture_wasm_overlay_001.png](../assets/pr7220_review/axis_picture_wasm_overlay_001.png)
- [axis_picture_wasm_review_001.png](../assets/pr7220_review/axis_picture_wasm_review_001.png)
- [axis_controls_wasm_compare_001.png](../assets/pr7220_review/axis_controls_wasm_compare_001.png)
- [axis_controls_wasm_overlay_001.png](../assets/pr7220_review/axis_controls_wasm_overlay_001.png)
- [axis_controls_wasm_review_001.png](../assets/pr7220_review/axis_controls_wasm_review_001.png)
- [axis_picture_base_review_001.png](../assets/pr7220_review/axis_picture_base_review_001.png)
- [axis_picture_base_overlay_001.png](../assets/pr7220_review/axis_picture_base_overlay_001.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
