---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-04
---

# PR #8281 검토 — 개체만 있는 마지막 줄의 끝 캐럿

## 최종 판정

**메인터너 보정 후 수용 가능 — 검증한 누적 후보에 한정.**
검증 head `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`, 제품 source `39b4f81f6a`, 원 기여 적용 merge `28dad1c94b`다. 원격 원 head의 충돌이나 이전 API를 그대로 승인한 것이 아니다. 10건의 로컬 보정·필수 검증 승인은 완료했고 push·공개 댓글·새 PR·merge는 별도 승인 범위다.

## 원본과 적용

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#8281](https://github.com/edwardkim/rhwp/pull/8281) / semanticist21 |
| 원 head | `fa6c4b7203408b36d405ba165348731ecb44001d` |
| 원 branch | `semanticist21/rhwp:fix/no-lineseg-inline-object-caret` |
| 고정 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 원 head CI | success 32, neutral 1, skipped 6; skip·neutral은 실행 성공으로 세지 않음 |
| 단독 merge-tree | 1파일 충돌 |
| Reviewer 확인 | edwardkim / 2026-11-03T19:19:30.031075+09:00 |
| 누적 branch | `review/integration-20261103-semanticist21` |

원 head는 누적 merge history에 보존했다. [선정·충돌·기여 보존](assets/semanticist21-20261103/intake.md)에서 열 건의 실제 적용 순서와 제외한 다섯 reviewer 대상 head를 확인한다. 원 PR의 과거 devel merge 자체는 기여 기능으로 세지 않았다.

## 최신 devel에서 남은 문제와 기여 효과

원본 두 검사는 devel에서도 통과했다. 실제 남은 경계인 여섯 글자 뒤 넓은 그림이 단독 마지막 줄로 넘어가면 devel은 끝 TextRun이 없어 캐럿 높이가 12px로 바뀐다. 후보는 논리 끝 7의 빈 런과 본문의 13.3px을 유지한다. 새 관계 검사는 base FAIL/후보 PASS다.

## 메인터너 보정과 실제 소비 경로

이미 반영된 text_logical_starts와 #8517 빈 그림 host 메트릭을 유지하고, 그림으로 끝나는 물리 줄의 끝 글자 상자를 공통 inline flow 계획에서 생산한다. 표로 끝나는 문단의 issue_6776 계약에는 적용하지 않는다.

inline_flow의 줄 나눔·paragraph-end style → paragraph_layout의 Storage TextRun push → getPageTextLayout/getCursorRect와 Canvas/Native. 동일 줄 결과가 마지막 이미지와 끝 앵커를 소유한다.

정상 대조·경계: 빈 그림 문단·그림 뒤 텍스트는 이미 정상인 대조군이다. 줄바꿈 뒤 그림 단독 줄을 새 음성 대조로 사용하고 live/HWPX 재열기를 확인했다.

## 실제 검증

| 시각 대조 입력 | 경로 | 전쪽 수 | 최저 raw |
| --- | --- | ---: | ---: |
| no-lineseg-empty | native | 1 | 100.00% |
| no-lineseg-text | native | 1 | 100.00% |
| no-lineseg-wrap | native | 1 | 92.90% |
| no-lineseg-empty | wasm | 1 | 100.00% |
| no-lineseg-text | wasm | 1 | 100.00% |
| no-lineseg-wrap | wasm | 1 | 96.56% |

전체 31입력·71쪽 Native/fresh WASM, 실제 notice 53호출·양식·다각형·간격·clip·끝 캐럿·중첩 저장의 browser API, 실제 두 backend 저장본의 교차 재열기를 [최종 명세](assets/semanticist21-20261103/validation.md)에서 구분한다. 전체 nextest 12,465 PASS / 45 skip, 관련 19개 모듈 78 PASS, Native Skia 필수 3종, 세 Clippy·workspace build·release build/lib·최적화 WASM·Studio가 통과했다. 승인된 통합 PR #8548의 exact-head CI를 확인하며, 최초 CI의 보안 보정과 그 새 검증은 [후속 기록](assets/semanticist21-20261103/ci-security-followup.md)에서 구분한다.

기준 devel의 같은 공개 계약 검사 36건은 13 PASS/23 FAIL이었다. 이후 추가한 wrap의 고유 경계 1건도 base FAIL/후보 PASS였다. #8281의 기존 두 성공, #8290 뒤 간격, #8308의 기존 분할 가시 배치는 이미 해결된 부분으로 분리했다. 구현 source 문자열이나 이전 CI를 현재 실행 증거로 세지 않았다.

## 조판 원칙 준수 검토

| 항목 | 근거와 판정 |
| --- | --- |
| 사양·원본·기존 해결 이력 | 위 원인·원본 저장값과 독립 Print/기존 해결 commit을 대조했다. 기존 API·Storage·schema 35·#8517을 되돌리지 않았다. |
| 공통 생산과 실제 소비 | 위 실제 경로에서 동일 줄/프레임/메타데이터 결과를 소비한다. 문서 ID·수치 조합이나 출력 clamp로 맞추지 않았다. |
| 저장 정보의 유효성 | 실물 원본과 모델로 만든 작은 대조군을 구분했다. 수동 메타데이터를 정상 저장본으로 가장하지 않았다. |
| 줄 소속·점유·보존 | 해당 실제 API와 기존 관계 검사의 내용 순서·한 번 표시·앞뒤 본문·거부 후 원자성을 보호했다. |
| 분할·이어받기 | 줄 컷/예약 알고리즘을 새로 만들지 않았다. 위 변경의 소유·저장·clip 소비와 정상 분할/후속 내용 회귀를 실제 실행했다. |
| 회귀 선행 조건·기준값 | 관련 Native/fresh WASM 전쪽 90% 이상과 직접 판독 뒤 필요한 관계 검사를 추가/보정했다. 독립 기준과 맞는 기존 기대값을 유지했다. |
| px 좌표 | 양쪽 98% 이상·직접 판독을 마친 완료 쪽만 독립 좌표 오차를 유지했다. 그 외는 소유·순서·포함·겹침·내용 보존 관계로 검사한다. |
| 실제 backend·글꼴 | Native Skia와 새 최적화 WASM Canvas를 직접 실행했다. 최소 글꼴 회귀와 독립 Print용 명시 공급은 별도다. 글꼴은 배포하지 않는다. |
| 전체 페이지와 확대 판독 | 모든 대상 전쪽 90% 이상·쪽수 일치. 작은 글자·문장부호·표 경계·간격과 각 PR의 변경 영역을 직접 대조했다. |
| 현재 원격 상태 | 원 head는 변경하지 않았다. 승인된 메인터너 통합 PR #8548을 열었고 새 head CI·self-review·merge를 확인한다. |

## Merge 후 contributor PR comment 계획

원 기여의 개체만 있는 마지막 줄의 끝 캐럿를 수용한 사실과 위 보정 이유를 존댓말로 안내한다. 실제 통합 PR·merge SHA·새 head CI URL과 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다. 원 PR별 원 head의 포함과 API 상태를 재확인한다.

직접 표시할 최종 패널은 [대표 비교](assets/semanticist21-20261103/zoom/no-lineseg-wrap.png)다. asset이 실제 devel에 포함된 것을 확인한 뒤 raw URL로 표시한다.

[원격 처리안과 댓글 초안](assets/semanticist21-20261103/remote-action-plan.md) · [누적 검토 계획](semanticist21_20261103_review_plan.md).
