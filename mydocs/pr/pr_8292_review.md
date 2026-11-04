---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-04
---

# PR #8292 검토 — 중복 표 ID의 HWPX 격자 저장

## 최종 판정

**메인터너 보정 후 수용 가능 — 검증한 누적 후보에 한정.**
검증 head `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`, 제품 source `39b4f81f6a`, 원 기여 적용 merge `8edd7f0a5f`다. 원격 원 head의 충돌이나 이전 API를 그대로 승인한 것이 아니다. 작업지시자가 통합 PR의 새 head CI·self-review·merge commit·원 PR별 후속 처리까지 승인했다. 초기 source 검증과 아래 최신 devel 재검증을 구분한다.

## 원본과 적용

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#8292](https://github.com/edwardkim/rhwp/pull/8292) / semanticist21 |
| 원 head | `1f26297e9cae86955dc36f2f100585cae4b06784` |
| 원 branch | `semanticist21/rhwp:fix/hwpx-table-grid-duplicate-ids` |
| 고정 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 원 head CI | success 32, neutral 1, skipped 6; skip·neutral은 실행 성공으로 세지 않음 |
| 단독 merge-tree | 1파일 충돌 |
| Reviewer 확인 | edwardkim / 2026-11-03T19:19:42.122439+09:00 |
| 누적 branch | `review/integration-20261103-semanticist21` |

원 head는 누적 merge history에 보존했다. [선정·충돌·기여 보존](assets/semanticist21-20261103/intake.md)에서 열 건의 실제 적용 순서와 제외한 다섯 reviewer 대상 head를 확인한다. 원 PR의 과거 devel merge 자체는 기여 기능으로 세지 않았다.

## 최신 devel에서 남은 문제와 기여 효과

행 격자 metadata가 필요한 표의 중복 ID를 저장 사본에서 분리해 빈 논리 행을 보존한다. 원본의 비중복 ID와 격자 metadata가 필요 없는 중복 ID는 유지한다.

## 메인터너 보정과 실제 소비 경로

현재 project_with_storage 경로의 clone·격자 rebuild·fingerprint·record push와 취소/예산 실패 전달을 보존했다. 앞 표에 발급할 ID가 뒤에 나오는 실제 원 ID와 충돌하지 않게 예산이 연결된 사전 ID 풀을 사용한다. UINT32_MAX·master page·중첩도 대조한다.

table_row_grid project_with_storage → 문서 원 ID 사전 순회 → 실제 metadata 소유에 한정한 ID 발급 → HWPX writer metadata → 재열기 grid. 원본 live IR은 바꾸지 않는다.

정상 대조·경계: 0 ID·같은 ID·미래 MAX 원 ID·중첩·비중복 보존, metadata 불필요 중복 보존, 예산 거부·취소와 새 요청 복구. 원본이 같은 시각 출력인 data-only 계약과 구분한다.

## 실제 검증

| 시각 대조 입력 | 경로 | 전쪽 수 | 최저 raw |
| --- | --- | ---: | ---: |
| grow-before | native | 1 | 100.00% |
| grow-after | native | 1 | 100.00% |
| grow-before | wasm | 1 | 100.00% |
| grow-after | wasm | 1 | 100.00% |

위 grow 대조는 정상 저장·paint 보존 확인이다. 중복 ID 재할당 자체는 row metadata·빈 논리 행·ID 보존의 구조 계약으로 검사하며 시각 점수를 그 데이터 계약의 증명으로 대체하지 않는다.

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

승인된 통합 PR [#8548](https://github.com/edwardkim/rhwp/pull/8548)의 exact-head CI·self-review·merge와 옵션 2 기록 PR 뒤 원 기여의 수용·감사·필요한 보정 이유를 한국어 존댓말로 게시한다. 실제 merge SHA·새 head CI·devel 포함을 확인하며 [댓글 초안](assets/semanticist21-20261103/comments/pr-8292.md)의 수치·문서/쪽 대응과 같은 증거를 사용한다.

이 기여는 원 PR이 제시한 합성 직렬화 또는 순수 composer 계약으로 검증한다. 실물 문서·해당 쪽의 시각 주장이 없으므로 관련 없는 다른 원본의 이미지를 독립 증거로 제시하지 않는다. 통합 전체의 시각 검증은 개별 계약과 구분해 연결한다.

## 최신 devel 통합 후 제출 기준
현재 후보 `696240b004707a6686864f94130befe3dd7f213e`, 통합 base `1924d371d6757392d063b8dc2e9bbe9a298be817`다. [최신 전쪽·실제 API·저장·필수 편람 결과](assets/semanticist21-20261103/rebase-validation.md)와 [직접 판독](assets/semanticist21-20261103/direct-review.md)의 최신 devel 절이 아래 제출 기준이다. 과거 c6ad/e68/H4의 결과를 이 후보의 새 실행으로 표시하지 않는다. 제품 렌더링 source가 같은 Native producer와 새 fresh WASM 실행을 명시적으로 구분한다.

stock nextest 12,475 PASS / 45 skip, Native 필수 3종·출처 계약·Rust lint·workspace·doc test·최적화 WASM·Studio 2,717건·운영 계약 275건을 통과했다. Native/WASM 각각 32개 입력·72쪽의 전쪽 raw 90% 미만과 누락·쪽수 불일치는 없다. 최신 base의 Native 편람 383쪽 gate도 최저 99.96465%·기준 대비 하락 0쪽으로 통과했다.
