---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7221_review.md
last_verified: 2026-09-17
---

# PR #7221 검토

## 최종 판정

**머지 보류** — PDF에 가까워진 paint top과 별개로 typeset의 원점·쪽 예산은 다른 계산을 사용한다.

[원 PR #7221](https://github.com/edwardkim/rhwp/pull/7221): 수정: 쪼개진 자리차지 표 조각을 문단 상자 상단에 건다 (#7203 분할 갈래)
관련 [이슈 #7203](https://github.com/edwardkim/rhwp/issues/7203).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `3ac1356dc46fab66994f55bba598f599da9c5e2e`, base `devel`. 검토자는 `jangster77`이다.
- source `3ac1356dc46fab66994f55bba598f599da9c5e2e` → applied `502dfe7d9e8b951e53e01d7322741a100e2e06ec`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35176870264/job/105060391038), [CI](https://github.com/edwardkim/rhwp/actions/runs/35176870265/job/105060391374), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35176869991/job/105060390602), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35176870328/job/105060391113), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35176870281/job/105060391182), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35176870150/job/105060390794), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35177857222). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7221_review_impl.md).

## 코드 경로와 독립 실행 증거

layout.rs:1645 helper가 prev line bottom으로 leading gap을 계산하고 :12750 실제 paint origin에 적용한다. 이 값은 typeset의 fragment 예약 원점으로 공유되지 않는다. 빈 host RowBreak 첫 조각의 컷/예약/paint가 공통 원점이라는 계약은 아직 성립하지 않는다.

hwpctl p26 pi528 y=717.3→710.6(PDF 709.94), p52 pi1274 y=948.4→941.7(PDF 940.73). 105쪽 유지와 간격 없는 대조군은 통과했다. p52/53의 compare와 overlay에서 윗변·이어받기·뒤 표를 직접 확인했다.

관련 실행: **issue_7203_split_float_anchors_to_paragraph_top 3개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

코드 검토상 규칙 위반: 변경된 원점이 paint에만 적용된다. 작성자도 typeset 원점 분리를 명시했다. 부분 시각 개선은 인정하지만 현재 공통 조판 원칙의 충족 증거로 볼 수 없다. #7203을 닫지 않는다.

같은 조각의 실제 top/점유 끝점을 공통 결과로 만들고 측정·예약·paint에서 소비한다. 원래 높이만 fit하는 경계 및 다음 표의 위치를 독립 PDF/정식 테스트로 확인한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 미충족 | layout.rs:1645 helper가 prev line bottom으로 leading gap을 계산하고 :12750 실제 paint origin에 적용한다. 이 값은 typeset의 fragment 예약 원점으로 공유되지 않는다. 빈 host RowBreak 첫 조각의 컷/예약/paint가 공통 원점이라는 계약은 아직 성립하지 않는다. |
| 분할·이어받기 계약 | 미충족 | 컷/예약/paint의 동일성 또는 새 페이지 경계 회귀가 해소되지 않았다. |
| 줄 소속과 점유 높이 | 미충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/hwpctl_API_v2.4.hwp](../../../samples/hwpctl_API_v2.4.hwp) | [pdf/hwpctl_API_v2.4-hwp-2020.pdf](../../../pdf/hwpctl_API_v2.4-hwp-2020.pdf) | 12,26,52,53,57 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

공유 hwpctl 증적은 [#7228 review](pr_7228_review.md#보존한-비교-png)의 p12/26/52/53/57 compare·overlay·review를 참조한다. 동일 PNG를 이름만 바꿔 중복 커밋하지 않는다.

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
