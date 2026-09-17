---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7215_review.md
last_verified: 2026-09-17
---

# PR #7215 검토

## 최종 판정

**머지 보류** — HWP3 오인 방지 자체의 focused 검사는 통과했지만, 현재 head에 포함된 #7221·#7228의 조판 계약 미충족이 남는다.

[원 PR #7215](https://github.com/edwardkim/rhwp/pull/7215): 수정: 한글97 마지막 줄 허용치에 HWP3 시대 신호를 함께 요구한다 (#7035)
관련 [이슈 #7035](https://github.com/edwardkim/rhwp/issues/7035).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `7d052733c3818914c80819d9767733fc803ff39d`, base `devel`. 검토자는 `jangster77`이다.
- source `7d052733c3818914c80819d9767733fc803ff39d` → applied `cd074a4da5e5afc38c8894d9e88a5d39490b3537`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35189554041/job/105098885410), [CI](https://github.com/edwardkim/rhwp/actions/runs/35189553986/job/105098885864), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35189552389/job/105098884307), [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/35189552387/job/105098879477), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35189554045/job/105098885232), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35189554074/job/105098885119), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35189553852/job/105098884374), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35190730286). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7215_review_impl.md).

## 코드 경로와 독립 실행 증거

HwpSummaryInformation의 era 신호 + 기존 style 비율 → apply_hwp3_origin_fixup → HWP3-origin pagination tolerance. strict/lenient 진입에서 era 인자를 전달한다. PDF 포맷 버전을 engine이나 허용 기준으로 쓰지 않는다.

native_hwp5_low_style_ratio는 기존 비율 조건에 들지만 era 신호가 없어 허용치가 부여되지 않는다. 기존 HWP3 변환 입력 대조군을 포함한 3개 통과. 통합 hwpctl은 Native/WASM 105쪽이며 148733091의 20쪽 끝/다음 내용도 비교했다. 이 결과는 #7228과의 누적 tree 결과다.

관련 실행: **issue_7035_hwp3_tolerance_needs_era_signal 3개, issue_6368 대조군 1개**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

원 PR 본문의 106쪽·실패 설명은 과거 중간 상태여서 현재 stacked head와 맞지 않는다. 자체 작은 변경의 통과를 의존 변경 전체 승인으로 확대하지 않는다. #7035 전체 해결·종료는 주장하지 않는다.

#7221·#7228의 원점·높이 공통 계약 보정 후 105쪽과 HWP3 정상 대조군을 재확인하고 본문을 최종 head 기준으로 정리한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 미충족 | HwpSummaryInformation의 era 신호 + 기존 style 비율 → apply_hwp3_origin_fixup → HWP3-origin pagination tolerance. strict/lenient 진입에서 era 인자를 전달한다. PDF 포맷 버전을 engine이나 허용 기준으로 쓰지 않는다. |
| 분할·이어받기 계약 | 미충족 | 컷/예약/paint의 동일성 또는 새 페이지 경계 회귀가 해소되지 않았다. |
| 줄 소속과 점유 높이 | 미검증 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [samples/issue6921/148733091-briefing-stenographic-record.hwp](../../../samples/issue6921/148733091-briefing-stenographic-record.hwp) | [pdf/148733091-briefing-stenographic-record-2020.pdf](../../../pdf/148733091-briefing-stenographic-record-2020.pdf) | 20 |
| [samples/hwpctl_API_v2.4.hwp](../../../samples/hwpctl_API_v2.4.hwp) | [pdf/hwpctl_API_v2.4-hwp-2020.pdf](../../../pdf/hwpctl_API_v2.4-hwp-2020.pdf) | 12,26,52,53,57 |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [stenographic_wasm_compare_020.png](../assets/pr7215_review/stenographic_wasm_compare_020.png)
- [stenographic_wasm_overlay_020.png](../assets/pr7215_review/stenographic_wasm_overlay_020.png)
- [stenographic_wasm_review_020.png](../assets/pr7215_review/stenographic_wasm_review_020.png)

공유 hwpctl 증적은 [#7228 review](pr_7228_review.md#보존한-비교-png)의 p12/26/52/53/57 compare·overlay·review를 참조한다. 동일 PNG를 이름만 바꿔 중복 커밋하지 않는다.

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
