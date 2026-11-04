---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-04
---

# PR #8293 검토 — 머리말·꼬리말 마지막 넘침 문단

## 최종 판정

**메인터너 보정 후 수용 가능 — 검증한 누적 후보에 한정.**
검증 head `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`, 제품 source `39b4f81f6a`, 원 기여 적용 merge `71692728f8`다. 원격 원 head의 충돌이나 이전 API를 그대로 승인한 것이 아니다. 작업지시자가 통합 PR의 새 head CI·self-review·merge commit·원 PR별 후속 처리까지 승인했다. 초기 source 검증과 아래 최신 devel 재검증을 구분한다.

## 원본과 적용

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#8293](https://github.com/edwardkim/rhwp/pull/8293) / semanticist21 |
| 원 head | `4447d39d2fa09155b1162203f4d1ed9f4e6fd1e6` |
| 원 branch | `semanticist21/rhwp:fix/header-overflow-paragraphs` |
| 고정 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 원 head CI | success 33, skipped 4; skip·neutral은 실행 성공으로 세지 않음 |
| 단독 merge-tree | 0파일 충돌 |
| Reviewer 확인 | edwardkim / 2026-11-03T19:19:46.547039+09:00 |
| 누적 branch | `review/integration-20261103-semanticist21` |

원 head는 누적 merge history에 보존했다. [선정·충돌·기여 보존](assets/semanticist21-20261103/intake.md)에서 열 건의 실제 적용 순서와 제외한 다섯 reviewer 대상 head를 확인한다. 원 PR의 과거 devel merge 자체는 기여 기능으로 세지 않았다.

## 최신 devel에서 남은 문제와 기여 효과

머리말 프레임을 넘는 마지막 문단도 한컴 출력처럼 표시한다. 빈 문단과 마지막 텍스트를 생략하지 않으며 본문과 의도적으로 겹치는 원본 배치를 clamp로 숨기지 않는다.

## 메인터너 보정과 실제 소비 경로

기존 HF 배치·정렬 규칙에서 마지막 문단을 소비하게 했다. HF caret/hit-test의 소유와 본문 클릭 소유를 함께 확인했다. 새 테스트의 JSON 키는 실제 공개 반환값 paragraphIndex를 사용한다.

HF 문단 순서·저장 줄 → HF frame layout의 마지막 문단 배치 → paint·getCursorRectInHeaderFooter·HF hit-test. 본문 첫 문단의 원점은 별도 계약으로 보존한다.

정상 대조·경계: 한컴 HWP/HWPX 두 형식, 70자 두 줄·빈 문단·마지막 문단, HF 클릭/캐럿과 일반 본문 클릭의 소유 분리.

## 실제 검증

| 시각 대조 입력 | 경로 | 전쪽 수 | 최저 raw |
| --- | --- | ---: | ---: |
| header-hwp | native | 1 | 100.00% |
| header-hwpx | native | 1 | 100.00% |
| header-hwp | wasm | 1 | 100.00% |
| header-hwpx | wasm | 1 | 100.00% |

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

승인된 통합 PR [#8548](https://github.com/edwardkim/rhwp/pull/8548)의 exact-head CI·self-review·merge와 옵션 2 기록 PR 뒤 원 기여의 수용·감사·필요한 보정 이유를 한국어 존댓말로 게시한다. 실제 merge SHA·새 head CI·devel 포함을 확인하며 [댓글 초안](assets/semanticist21-20261103/comments/pr-8293.md)의 수치·문서/쪽 대응과 같은 증거를 사용한다.

원 PR의 주장 입력/페이지와 직접 대응시킨 패널은 다음과 같다. first comment에서 다른 PR의 대표 그림으로 대신하지 않는다.

- header-hwp / native: [review](assets/semanticist21-20261103/rebase/png/header-hwp-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/header-hwp-native-overlay.png).
- header-hwp / wasm: [review](assets/semanticist21-20261103/rebase/png/header-hwp-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/header-hwp-wasm-overlay.png).
- header-hwpx / native: [review](assets/semanticist21-20261103/rebase/png/header-hwpx-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/header-hwpx-native-overlay.png).
- header-hwpx / wasm: [review](assets/semanticist21-20261103/rebase/png/header-hwpx-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/header-hwpx-wasm-overlay.png).

## 최신 devel 통합 후 제출 기준
현재 후보 `696240b004707a6686864f94130befe3dd7f213e`, 통합 base `1924d371d6757392d063b8dc2e9bbe9a298be817`다. [최신 전쪽·실제 API·저장·필수 편람 결과](assets/semanticist21-20261103/rebase-validation.md)와 [직접 판독](assets/semanticist21-20261103/direct-review.md)의 최신 devel 절이 아래 제출 기준이다. 과거 c6ad/e68/H4의 결과를 이 후보의 새 실행으로 표시하지 않는다. 제품 렌더링 source가 같은 Native producer와 새 fresh WASM 실행을 명시적으로 구분한다.

stock nextest 12,475 PASS / 45 skip, Native 필수 3종·출처 계약·Rust lint·workspace·doc test·최적화 WASM·Studio 2,717건·운영 계약 275건을 통과했다. Native/WASM 각각 32개 입력·72쪽의 전쪽 raw 90% 미만과 누락·쪽수 불일치는 없다. 최신 base의 Native 편람 383쪽 gate도 최저 99.96465%·기준 대비 하락 0쪽으로 통과했다.

이 PR의 대조 입력 2개 최저 raw는 Native 100.00000%, fresh WASM 100.00000%다. 점수와 별도로 원본·독립 Print·작은 문자·기호·간격·누락/중복·clip·후속 본문을 직접 대조했다.

## 223 head CI 후속 보정의 제출 기준

최신 제품 보정은 `5e374d01f641745d20bc9424ce233021a327ed16`다. Native CLI의 추가 계약 target을 빠뜨린 것을 보완해 실제 Native CI 목록 22개 전체를 통과했다. stock 12,475건·Native 전체 lib 4,040건·실제 출처/보호 CLI도 통과했다. 코드 보정·기존 해결 이력·정확한 실행과 조판 증거 재사용은 [CI 후속 검증](assets/semanticist21-20261103/ci-head223-followup.md)을 따른다. 앞선 223 head의 성공하지 못한 CI를 재사용하지 않고 새 제출 head CI·CodeQL 성공과 self-review 뒤 통합한다.
