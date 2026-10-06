---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7223_review.md
last_verified: 2026-09-17
---

# PR #7223 검토

## 최종 판정

**승인** — 저장 host 줄의 음수 line_spacing을 측정과 배치 양쪽에서 유지하는 변경을 수용한다.

[원 PR #7223](https://github.com/edwardkim/rhwp/pull/7223): 수정: 자리차지 표 뒤 host 줄 사다리를 음수 줄간격으로도 판정한다 (#7198)
관련 [이슈 #7198](https://github.com/edwardkim/rhwp/issues/7198).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `9744cfb806e04e77be82891b2535ef78636ae1a5`, base `devel`. 검토자는 `jangster77`이다.
- source `92810d4b5ebc3bb5191414339b710bb1d35bc4b2` → applied `dd8db6bbb453825e2197cdc09970ea856190e2e5`
- source `9744cfb806e04e77be82891b2535ef78636ae1a5` → applied `f1cf9a4f5f3dc67705405222c018e3c678537500`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35181411795/job/105074152763), [CI](https://github.com/edwardkim/rhwp/actions/runs/35181411813/job/105074153227), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35181411588/job/105074152214), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35181411811/job/105074152656), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35181411827/job/105074152912), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35181411667/job/105074152301), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35182346686). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7223_review_impl.md).

## 코드 경로와 독립 실행 증거

저장 LineSeg line_height + signed line_spacing → typeset과 layout의 host ladder 전진. 양수 line_spacing과 비정상 비양수 전진 guard는 유지한다.

156403546 1쪽 제목이 상단 회색 띠와 겹치는 문제가 개선됐고 해당 focused 검사 통과. 내용 픽셀 일치율 base 20.46→28.88%. 양수 간격 156451317 1쪽은 base와 통합 Native PNG가 byte-identical이어서 정상 대조군임을 직접 확인했다. Native/WASM도 일치했다.

관련 실행: **issue_7198_negative_spacing_host_after_float_table 2개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

보도자료 로고 위치/누락·본문 글꼴·아래 표의 기존 차이는 남는다. 제목의 host 전진 개선을 전체 문서 일치로 보고하지 않는다.

최종 통합 CI 및 다른 보류 사유 해소 후 merge한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 충족 | 저장 LineSeg line_height + signed line_spacing → typeset과 layout의 host ladder 전진. 양수 line_spacing과 비정상 비양수 전진 guard는 유지한다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/issue7198/156403546_negative_spacing_host_after_table.hwp](../../../samples/issue7198/156403546_negative_spacing_host_after_table.hwp) | [samples/issue7198/156403546_negative_spacing_host_after_table-2020.pdf](../../../samples/issue7198/156403546_negative_spacing_host_after_table-2020.pdf) | 1 |
| [samples/issue7198/156451317_positive_spacing_host_after_table.hwp](../../../samples/issue7198/156451317_positive_spacing_host_after_table.hwp) | [samples/issue7198/156451317_positive_spacing_host_after_table-2020.pdf](../../../samples/issue7198/156451317_positive_spacing_host_after_table-2020.pdf) | 1 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [negative_host_wasm_compare_001.png](../assets/pr7223_review/negative_host_wasm_compare_001.png)
- [negative_host_wasm_overlay_001.png](../assets/pr7223_review/negative_host_wasm_overlay_001.png)
- [negative_host_wasm_review_001.png](../assets/pr7223_review/negative_host_wasm_review_001.png)
- [positive_host_wasm_compare_001.png](../assets/pr7223_review/positive_host_wasm_compare_001.png)
- [positive_host_wasm_overlay_001.png](../assets/pr7223_review/positive_host_wasm_overlay_001.png)
- [positive_host_wasm_review_001.png](../assets/pr7223_review/positive_host_wasm_review_001.png)
- [negative_host_base_review_001.png](../assets/pr7223_review/negative_host_base_review_001.png)
- [negative_host_base_overlay_001.png](../assets/pr7223_review/negative_host_base_overlay_001.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
