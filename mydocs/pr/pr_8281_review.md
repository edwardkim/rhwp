---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8281 검토 — 저장 줄 없는 개체 뒤 캐럿

## 최종 판정

**로컬 보정과 집중·시각 검증 진행 — 최종 전체 검증이 끝나기 전까지 머지 보류.**
원 head를 보존한 누적 후보에 보정과 실제 API·시각 대조를 적용했다. [현재 검증 명세](assets/semanticist21-20261103/validation.md)의 중간 제품 head와 최종 정규 검증을 구분한다. 모든 필수 검증이 끝난 뒤 최종 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8281](https://github.com/edwardkim/rhwp/pull/8281) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:fix/no-lineseg-inline-object-caret` |
| 원 head | `fa6c4b7203408b36d405ba165348731ecb44001d` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:30.031075+09:00 API 확인 |
| 원 head CI | commit status success; neutral 1, skipped 6, success 32 |
| 실제 merge-tree | 충돌 1파일 |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

문자 논리 주소와 그림으로 끝나는 문단의 끝 캐럿 상자를 보정한다.

문자 논리 주소는 실제 devel 조상 3a71190ec382ee94c991ae4a25f0794e2727f6e4의 text_logical_starts로 반영됐다. 끝 런·끝 글꼴 높이는 별도 실행 검증 대상이다.

text_logical_starts를 유지하고 남은 문단 끝 런·높이만 공통 inline flow 계획과 실제 배치에서 함께 소비한다. 기존 Storage push 경로와 표로 끝나는 정상 대조군을 보존한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

충돌 파일:
- `src/renderer/layout/paragraph_layout.rs`

## 검증 범위

no_lineseg_inline_object_caret_anchor; 그림 앞뒤·그림만 있는 문단·끝 글자 모양·HWPX 저장 재열기·표 끝 대조

제품 경로:
- `src/renderer/inline_flow.rs`
- `src/renderer/layout/paragraph_layout.rs`

정식 focused 대상:
- `tests/cases/no_lineseg_inline_object_caret_anchor.rs`

원본·독립 Print·편집 상태의 대응을 확인하고 현재 base/후보의 Native/fresh WASM 전쪽 TSV와 확대 판독을
확보한다. 입력이 합성 계약인 범위와 실물 한컴 출력은 구분한다. 현재 단계에 새 측정 수치를 적지 않는다.

## 조판 원칙 준수 검토

| 항목 | 접수 판정과 후속 근거 |
| --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기존 이슈·해결 PR의 선행 분석 | 미검증 — 위 현재 devel 관측과 #8093/#8187/#8517의 선행 연결을 시작했으며 적용 경계의 실행을 남긴다. |
| 구현 근거와 일반성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 측정·배치 일관성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 리베이스 반영 PR의 기능 보존 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 최종 출력 소비 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 분할·이어받기 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 줄 소속과 점유 높이 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 사례와 증거의 독립성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기준값 변경 | 비해당 — 접수 단계에서 코드·기대값·baseline·golden을 변경하지 않았다. 이후 검사 변경은 독립 출력 gate 뒤에 수행한다. |
| 회귀 추가·변경의 선행 조건 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 90% 통과 후 확대 판독 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 주장과 검증 범위 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |

## Merge 후 contributor PR comment 계획

현재는 maintainer 시각 검증 미실행이다. 검증 완료 뒤 이 원 PR의 실제 결론·source/merge SHA·CI URL,
[Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 실제 측정한 전쪽 최저값·직접 판독을
연결한다. 수용 시 원 기여와 메인터너 보정 이유를 존댓말로 구분하고 정식 대표 PNG가 devel에 반영된 뒤
승인된 원격 범위에서 게시·API 재확인한다. 아직 없는 수치나 이미지 URL은 만들지 않는다.

[누적 검토 계획](semanticist21_20261103_review_plan.md) · [원 head·CI·충돌 증거](assets/semanticist21-20261103/intake.json).

## 로컬 충돌 보정

현행 text_logical_starts를 유지했다. 끝 캐럿도 같은 주소 결과의 마지막 값을 소비한다.
#8517의 빈 그림 호스트 메트릭을 유지하고 남은 끝 글자 상자는 Storage push로 연결했다.
원 PR 회귀의 `나` 시작 주소 1은 이미 해결된 원문 축 기대값이므로, 그림 슬롯을 포함한
현행 논리 주소 2로 대조한다. 실행·독립 출력 검증은 누적 후보에서 이어 간다.

## 실제 미해결 경계의 추가 확인

원본 두 검사는 고정 devel에서도 모두 통과했다. 그림뿐인 문단은 기존 empty-host text box가 있고,
그림 뒤에 글자가 있는 문단은 그 글자가 끝 앵커를 제공하므로, 두 성공을 미해결의 증거로 쓰지 않는다.
실제 남은 경계는 본문 뒤의 넓은 그림이 마지막 물리 줄을 단독 소유하는 경우다.

공개 WASM API에서 여섯 글자 뒤에 40000×400 HU 그림을 넣었다. devel은 끝 TextRun이 없고 캐럿이
기본 12px로 바뀌었다. 후보는 논리 끝 7의 빈 TextRun과 본문과 같은 13.3px을 보존했다.
수정 전 Native 2 PASS/1 FAIL, 수정 후 Native 3 PASS를 최소 글꼴 환경에서 확인했다.
독립 한컴 2024 Print와 Native/fresh WASM은 같은 그림의 160..164px 밴드를 보이며 raw는
92.90%/96.56%다. 인쇄의 계단형 이미지와 두 backend의 부드러운 보간을 구분해 위치와 글자 순서를 확대 판독했다.
이 시각 gate 이후 같은 상태의 관계형 회귀를 기존 파일에 추가했다. 표로 끝나는 문단은 그림의 끝 글자 상자 대상에서 제외하며 기존 issue_6776 계약을 유지한다.
