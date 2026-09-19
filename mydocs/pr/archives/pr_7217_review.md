---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7217_review.md
last_verified: 2026-09-17
---

# PR #7217 검토

## 최종 판정

**승인** — 그림 프레임의 흐름 geometry를 보존하면서 내부 여백을 paint 영역에 반영하는 변경을 수용한다.

[원 PR #7217](https://github.com/edwardkim/rhwp/pull/7217): 수정: 그림 안쪽 여백을 뺀 자리에 그림을 그린다 (#7193)
관련 [이슈 #7193](https://github.com/edwardkim/rhwp/issues/7193).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `0ce544f913dfd8671a7167389ac3b5ac77aadfd2`, base `devel`. 검토자는 `jangster77`이다.
- source `0ce544f913dfd8671a7167389ac3b5ac77aadfd2` → applied `09204fd03700d4d5190dd7168bc53581f628e550`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35173088561/job/105048747180), [CI](https://github.com/edwardkim/rhwp/actions/runs/35173088683/job/105048747472), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35173088265/job/105048746146), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35173088557/job/105048746752), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35173088502/job/105048746845), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35173088436/job/105048747083), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35174140795). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7217_review_impl.md).

## 코드 경로와 독립 실행 증거

picture_content_inset → ImageNode.content_inset → paint_bbox. SVG·Canvas2D/paint ops·Skia·HTML 소비 경로를 대조했으며 flow/hit-test frame은 유지한다. 회전·무효 여백 제외 조건은 명시된 적용 범위다.

2983289/3184393의 첨부파일 그림과 편람 130쪽의 네 방향 내부 여백을 Native/fresh WASM + 한컴 PDF로 직접 비교했다. 2983289 내용 픽셀 일치율은 base 26.43→33.39%, 편람은 30.67→35.22%였다. 이 수치는 보조값이며 실제 그림 영역·주변 줄을 함께 보았다. margin 없는 frame 대조군과 LayerTree paint op 검사도 통과했다.

관련 실행: **issue_7193_picture_inner_margin 4개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

편람 그림의 외곽선, 문서 글꼴 등의 기존 차이가 남아 전체 문서의 픽셀 일치를 승인한 것은 아니다. #7217에 포함되지 않은 다른 그림 변환/회전 문제까지 해결했다고 쓰지 않는다.

통합 최종 CI와 다른 보류 사유 해소 후 merge한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 충족 | picture_content_inset → ImageNode.content_inset → paint_bbox. SVG·Canvas2D/paint ops·Skia·HTML 소비 경로를 대조했으며 flow/hit-test frame은 유지한다. 회전·무효 여백 제외 조건은 명시된 적용 범위다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/issue7193/2983289_picture_inner_margin.hwpx](../../../samples/issue7193/2983289_picture_inner_margin.hwpx) | [samples/issue7193/2983289_picture_inner_margin-2020.pdf](../../../samples/issue7193/2983289_picture_inner_margin-2020.pdf) | 1 |
| [samples/issue7193/3184393_picture_inner_margin.hwp](../../../samples/issue7193/3184393_picture_inner_margin.hwp) | [samples/issue7193/3184393_picture_inner_margin-2020.pdf](../../../samples/issue7193/3184393_picture_inner_margin-2020.pdf) | 1 |
| [samples/2025 행정업무운영 편람(최종).hwpx](../../../samples/2025%20%ED%96%89%EC%A0%95%EC%97%85%EB%AC%B4%EC%9A%B4%EC%98%81%20%ED%8E%B8%EB%9E%8C%28%EC%B5%9C%EC%A2%85%29.hwpx) | [pdf/2025 행정업무운영 편람(최종)-hwpx-2020.pdf](../../../pdf/2025%20%ED%96%89%EC%A0%95%EC%97%85%EB%AC%B4%EC%9A%B4%EC%98%81%20%ED%8E%B8%EB%9E%8C%28%EC%B5%9C%EC%A2%85%29-hwpx-2020.pdf) | 130 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [picture_hwpx_wasm_compare_001.png](../assets/pr7217_review/picture_hwpx_wasm_compare_001.png)
- [picture_hwpx_wasm_overlay_001.png](../assets/pr7217_review/picture_hwpx_wasm_overlay_001.png)
- [picture_hwpx_wasm_review_001.png](../assets/pr7217_review/picture_hwpx_wasm_review_001.png)
- [picture_hwp_wasm_compare_001.png](../assets/pr7217_review/picture_hwp_wasm_compare_001.png)
- [picture_hwp_wasm_overlay_001.png](../assets/pr7217_review/picture_hwp_wasm_overlay_001.png)
- [picture_hwp_wasm_review_001.png](../assets/pr7217_review/picture_hwp_wasm_review_001.png)
- [picture_four_sides_wasm_compare_130.png](../assets/pr7217_review/picture_four_sides_wasm_compare_130.png)
- [picture_four_sides_wasm_overlay_130.png](../assets/pr7217_review/picture_four_sides_wasm_overlay_130.png)
- [picture_four_sides_wasm_review_130.png](../assets/pr7217_review/picture_four_sides_wasm_review_130.png)
- [picture_hwpx_base_review_001.png](../assets/pr7217_review/picture_hwpx_base_review_001.png)
- [picture_hwpx_base_overlay_001.png](../assets/pr7217_review/picture_hwpx_base_overlay_001.png)
- [picture_four_sides_base_review_130.png](../assets/pr7217_review/picture_four_sides_base_review_130.png)
- [picture_four_sides_base_overlay_130.png](../assets/pr7217_review/picture_four_sides_base_overlay_130.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
