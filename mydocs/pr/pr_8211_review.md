---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-04
---

# PR #8211 검토 — 표 삽입·본문 흐름

## 최종 판정

**메인터너 보정 후 수용 가능 — 검증한 누적 후보에 한정.**
검증 head `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`, 제품 source `39b4f81f6a`, 원 기여 적용 merge `53128ad702`다. 원격 원 head의 충돌이나 이전 API를 그대로 승인한 것이 아니다. 작업지시자가 통합 PR의 새 head CI·self-review·merge commit·원 PR별 후속 처리까지 승인했다. 초기 source 검증과 아래 최신 devel 재검증을 구분한다.

## 원본과 적용

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#8211](https://github.com/edwardkim/rhwp/pull/8211) / semanticist21 |
| 원 head | `82b959af244dfdbfeaffb4186910c371f2afdc29` |
| 원 branch | `semanticist21/rhwp:codex/table-measurement-ownership` |
| 고정 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 원 head CI | success 31, skipped 4; skip·neutral은 실행 성공으로 세지 않음 |
| 단독 merge-tree | 5파일 충돌 |
| Reviewer 확인 | edwardkim / 2026-11-03T19:19:22.646529+09:00 |
| 누적 branch | `review/integration-20261103-semanticist21` |

원 head는 누적 merge history에 보존했다. [선정·충돌·기여 보존](assets/semanticist21-20261103/intake.md)에서 열 건의 실제 적용 순서와 제외한 다섯 reviewer 대상 head를 확인한다. 원 PR의 과거 devel merge 자체는 기여 기능으로 세지 않았다.

## 최신 devel에서 남은 문제와 기여 효과

실제 53개 표 삽입·셀 편집 호출에서 devel의 live 28쪽과 저장 뒤 24쪽 불일치를 해소했다. 후보는 live·저장 재열기 모두 독립 Print와 같은 24쪽이며 기존 뒤 본문·중첩 표·꼬리 내용을 보존한다.

## 메인터너 보정과 실제 소비 경로

삽입 시 측정 문단 주소를 이동하고 원본 제어 스트림의 완전성을 인증한다. 첫 문단 원점을 initialize_first_paragraph_origin에서 생산해 일반/분할 진입과 vpos가 같은 하한을 소비한다. None 혼합 host의 현행 unrestricted_float와 split handoff를 보존한다.

src/document_core/commands/object_ops/table.rs의 shift_for_insert → 원본 줄·개체 소유 측정 → typeset/paragraph의 initialize_first_paragraph_origin → typeset/table/block/entry와 typeset/section/vpos → 실제 표·뒤 본문 배치. 기존 table_fragment_source_range의 컷·이월·종료 보호를 유지한다.

정상 대조·경계: fresh/loaded/empty-host/after-table 네 소유 상태, TAC·Square·None·CellBreak 및 실제 notice의 중간/이어받기/꼬리.

## 실제 검증

| 시각 대조 입력 | 경로 | 전쪽 수 | 최저 raw |
| --- | --- | ---: | ---: |
| notice-reopened | native | 24 | 97.74% |
| ownership-fresh-reference | native | 1 | 100.00% |
| ownership-loaded-before | native | 1 | 100.00% |
| ownership-empty-host-after | native | 1 | 100.00% |
| ownership-after-table-after | native | 1 | 100.00% |
| notice-reopened | wasm | 24 | 97.46% |
| ownership-fresh-reference | wasm | 1 | 100.00% |
| ownership-loaded-before | wasm | 1 | 100.00% |
| ownership-empty-host-after | wasm | 1 | 100.00% |
| ownership-after-table-after | wasm | 1 | 100.00% |

전체 31입력·71쪽 Native/fresh WASM, 실제 notice 53호출·양식·다각형·간격·clip·끝 캐럿·중첩 저장의 browser API, 실제 두 backend 저장본의 교차 재열기를 [최종 명세](assets/semanticist21-20261103/validation.md)에서 구분한다. 전체 nextest 12,465 PASS / 45 skip, 관련 19개 모듈 78 PASS, Native Skia 필수 3종, 세 Clippy·workspace build·release build/lib·최적화 WASM·Studio가 통과했다. 승인된 통합 PR #8548의 exact-head CI를 확인하며, 최초 CI의 보안 보정과 그 새 검증은 [후속 기록](assets/semanticist21-20261103/ci-security-followup.md)에서 구분한다.

기준 devel의 같은 공개 계약 검사 36건은 13 PASS/23 FAIL이었다. 이후 추가한 wrap의 고유 경계 1건도 base FAIL/후보 PASS였다. #8281의 기존 두 성공, #8290 뒤 간격, #8308의 기존 분할 가시 배치는 이미 해결된 부분으로 분리했다. 구현 source 문자열이나 이전 CI를 현재 실행 증거로 세지 않았다.

## 조판 원칙 준수 검토

| 항목 | 근거와 판정 |
| --- | --- |
| 사양·원본·기존 해결 이력 | 위 원인·원본 저장값과 독립 Print/기존 해결 commit을 대조했다. 기존 API·Storage·schema 35·#8517을 되돌리지 않았다. |
| 공통 생산과 실제 소비 | 위 실제 경로에서 동일 줄/프레임/메타데이터 결과를 소비한다. 문서 ID·수치 조합이나 출력 clamp로 맞추지 않았다. |
| 저장 정보의 유효성 | 실물 원본과 모델로 만든 작은 대조군을 구분했다. 수동 메타데이터를 정상 저장본으로 가장하지 않았다. |
| 줄 소속·점유·보존 | 해당 실제 API와 기존 관계 검사의 내용 순서·한 번 표시·앞뒤 본문·거부 후 원자성을 보호했다. |
| 분할·이어받기 | 첫 원점의 일반/분할 진입·vpos 소비, CellBreak·표 뒤 본문과 실제 24쪽의 중간/꼬리를 대조했다. source 컷·예약·종료의 기존 table_fragment_source_range 보호도 통과했다. |
| 회귀 선행 조건·기준값 | 관련 Native/fresh WASM 전쪽 90% 이상과 직접 판독 뒤 필요한 관계 검사를 추가/보정했다. 독립 기준과 맞는 기존 기대값을 유지했다. |
| px 좌표 | 양쪽 98% 이상·직접 판독을 마친 완료 쪽만 독립 좌표 오차를 유지했다. 그 외는 소유·순서·포함·겹침·내용 보존 관계로 검사한다. |
| 실제 backend·글꼴 | Native Skia와 새 최적화 WASM Canvas를 직접 실행했다. 최소 글꼴 회귀와 독립 Print용 명시 공급은 별도다. 글꼴은 배포하지 않는다. |
| 전체 페이지와 확대 판독 | 모든 대상 전쪽 90% 이상·쪽수 일치. 작은 글자·문장부호·표 경계·간격과 각 PR의 변경 영역을 직접 대조했다. |
| 현재 원격 상태 | 원 head는 변경하지 않았다. 승인된 메인터너 통합 PR #8548을 열었고 새 head CI·self-review·merge를 확인한다. |

## Merge 후 contributor PR comment 계획

승인된 통합 PR [#8548](https://github.com/edwardkim/rhwp/pull/8548)의 exact-head CI·self-review·merge와 옵션 2 기록 PR 뒤 원 기여의 수용·감사·필요한 보정 이유를 한국어 존댓말로 게시한다. 실제 merge SHA·새 head CI·devel 포함을 확인하며 [댓글 초안](assets/semanticist21-20261103/comments/pr-8211.md)의 수치·문서/쪽 대응과 같은 증거를 사용한다.

원 PR의 주장 입력/페이지와 직접 대응시킨 패널은 다음과 같다. first comment에서 다른 PR의 대표 그림으로 대신하지 않는다.

- notice-live-p21 / native: [review](assets/semanticist21-20261103/rebase/png/notice-live-p21-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/notice-live-p21-native-overlay.png).
- notice-live-p21 / wasm: [review](assets/semanticist21-20261103/rebase/png/notice-live-p21-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/notice-live-p21-wasm-overlay.png).
- ownership-fresh-reference / native: [review](assets/semanticist21-20261103/rebase/png/ownership-fresh-reference-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-fresh-reference-native-overlay.png).
- ownership-fresh-reference / wasm: [review](assets/semanticist21-20261103/rebase/png/ownership-fresh-reference-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-fresh-reference-wasm-overlay.png).
- ownership-loaded-before / native: [review](assets/semanticist21-20261103/rebase/png/ownership-loaded-before-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-loaded-before-native-overlay.png).
- ownership-loaded-before / wasm: [review](assets/semanticist21-20261103/rebase/png/ownership-loaded-before-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-loaded-before-wasm-overlay.png).
- ownership-empty-host-after / native: [review](assets/semanticist21-20261103/rebase/png/ownership-empty-host-after-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-empty-host-after-native-overlay.png).
- ownership-empty-host-after / wasm: [review](assets/semanticist21-20261103/rebase/png/ownership-empty-host-after-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-empty-host-after-wasm-overlay.png).
- ownership-after-table-after / native: [review](assets/semanticist21-20261103/rebase/png/ownership-after-table-after-native-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-after-table-after-native-overlay.png).
- ownership-after-table-after / wasm: [review](assets/semanticist21-20261103/rebase/png/ownership-after-table-after-wasm-review.png) · [overlay](assets/semanticist21-20261103/rebase/png/ownership-after-table-after-wasm-overlay.png).

## 최신 devel 통합 후 제출 기준
현재 후보 `696240b004707a6686864f94130befe3dd7f213e`, 통합 base `1924d371d6757392d063b8dc2e9bbe9a298be817`다. [최신 전쪽·실제 API·저장·필수 편람 결과](assets/semanticist21-20261103/rebase-validation.md)와 [직접 판독](assets/semanticist21-20261103/direct-review.md)의 최신 devel 절이 아래 제출 기준이다. 과거 c6ad/e68/H4의 결과를 이 후보의 새 실행으로 표시하지 않는다. 제품 렌더링 source가 같은 Native producer와 새 fresh WASM 실행을 명시적으로 구분한다.

stock nextest 12,475 PASS / 45 skip, Native 필수 3종·출처 계약·Rust lint·workspace·doc test·최적화 WASM·Studio 2,717건·운영 계약 275건을 통과했다. Native/WASM 각각 32개 입력·72쪽의 전쪽 raw 90% 미만과 누락·쪽수 불일치는 없다. 최신 base의 Native 편람 383쪽 gate도 최저 99.96465%·기준 대비 하락 0쪽으로 통과했다.

이 PR의 대조 입력 5개 최저 raw는 Native 97.69193%, fresh WASM 97.45945%다. 점수와 별도로 원본·독립 Print·작은 문자·기호·간격·누락/중복·clip·후속 본문을 직접 대조했다.
