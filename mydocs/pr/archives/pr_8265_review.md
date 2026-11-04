# PR #8265 리뷰 — AI 제공자와 SSE 청크 경계

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**통합 후보 최종 판정 보류 — AI 제공자와 SSE 청크 경계의 관련 결과는 아래에 기록했다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8265](https://github.com/edwardkim/rhwp/pull/8265) / planet6897 / devel |
| 체리픽 당시 원 head | `65196bb3f97293763b7736438feb6d47a5072922` |
| 2026-10-29 재조회 원 head | `3d99386bb15ccb30b39735594e5771a01653acb8`; [head·본문 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json), [리베이스 patch 검토](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md) |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 1개 적용 / 0개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8265_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

OpenAI·Gemini의 모델·동의·도구 왕복·오류·제공자 전환을 실제 UI로 검사했다. CR/LF가 별도 청크로 올 때 SSE event가 조기 종료되지 않도록 보정했다. 읽기 도구 전후 문서 바이트 보존과 no-AI 빌드 제외도 확인했다.

원 PR 제목: feat(studio): AI 도우미에 OpenAI·Gemini 제공자 추가 (P3)

주요 구현 경로: [chat-controller.ts](<../../../rhwp-studio/src/ai/agent/chat-controller.ts>), [gemini.ts](<../../../rhwp-studio/src/ai/provider/gemini.ts>), [http.ts](<../../../rhwp-studio/src/ai/provider/http.ts>), [models.ts](<../../../rhwp-studio/src/ai/provider/models.ts>), [openai.ts](<../../../rhwp-studio/src/ai/provider/openai.ts>), [registry.ts](<../../../rhwp-studio/src/ai/provider/registry.ts>), [ai-panel.ts](<../../../rhwp-studio/src/ai/ui/ai-panel.ts>).

보호 검사: [ai-providers.test.mjs](<../../../rhwp-studio/e2e/ai-providers.test.mjs>), [ai-gemini-provider.test.ts](<../../../rhwp-studio/tests/ai-gemini-provider.test.ts>), [ai-openai-provider.test.ts](<../../../rhwp-studio/tests/ai-openai-provider.test.ts>), [ai-provider-switch.test.ts](<../../../rhwp-studio/tests/ai-provider-switch.test.ts>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

추가 보정: `0aa7a8c63ab560dc10884158f5ae2b512a492718`. 기여자의 원 변경과 구분하며 일반성·독립 근거·수정 전후는 통합 계획에 기록했다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

제공자·도구 왕복은 실제 Studio UI와 모의 API 응답, 문서 바이트 보존으로 검증한다. 다른 PR의 HWP 저장 검사나 쪽별 시각 수치를 이 PR의 직접 증거로 사용하지 않는다.

[SSE·제공자 focused 검사](<../../../mydocs/working/assets/review_integration_20261026/validation/non-rust-validation.json>), [fresh WASM 실제 제공자 E2E](<../../../mydocs/working/assets/review_integration_20261026/validation/browser-validation.json>), [extension no-AI 빌드](<../../../mydocs/working/assets/review_integration_20261026/validation/extension-no-ai-builds.json>)를 연결한다. 실제 키·외부 API는 사용하지 않았으며 extension의 기존 WASM 빌드는 AI 제외 검증만 입증한다.

## 조판 원칙 준수 판정

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬 검증 | 완료된 공통 검증은 [선택 회귀 110개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json), [후속 회귀 24개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json), [Rust lint·build](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr-local-rust-gates-20261029.json)과 [시각 도구·CI 구문](../../working/assets/review_integration_20261026/validation/post-rebase-8328/prepr-visual-ci-checks.json)에 기록한다. 전체 Rust 회귀는 작업지시자 지시에 따라 재실행하지 않았고 원격 CI는 별도다. |
| 기존 해결의 선행 분석 | 충족 — [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>); 원 이슈·원 해결·보정 SHA를 실패별로 추적 |
| 구현 근거와 일반성 | 충족 — 위 실제 소비 경로와 독립 기대값. 문서 ID 보정·출력 은폐 없이 공통 결과 사용 |
| 측정·배치 일관성 | 비해당 — 글꼴 순수 판정·digest 또는 저장 payload·순서·AI 전송 계약이며 조판 계산을 바꾸지 않음 |
| 리베이스 기능 보존 | 충족 — #8116 새 base의 15개 계약·4개 문서와 기존 17개 계약·전쪽을 함께 검증. 한 충돌에서 두 수용 조건을 보존했으며 커밋 대조와 실행 원장에 기록 |
| 최종 출력 소비 | 충족 — 글꼴·digest의 독립 입력 반례, payload 재열기·교체 반례 또는 실제 Studio UI |
| 분할·이어받기 | 비해당 — 표 조각 컷·이어받기 규칙은 변경하지 않음 |
| 줄 소속과 점유 높이 | 비해당 — 새 줄 배치 규칙 없음 |
| 독립성·반례 | 충족 — 실제 원본·독립 Print 또는 원본 payload/외부 경로·편집 대조군. 임시 스크립트를 정식 회귀로 대신하지 않음 |
| 기준값 변경 | 비해당 — 이 PR의 baseline·golden·허용치를 완화하지 않음 |
| 회귀 추가 선행 조건 | 비해당 — 새 물리 배치/pixel 기대값 없음 |
| 수식·간격 확대 판독 | 비해당 — 물리 배치 변경 없음 |
| 구현 주장과 검증 범위 | 충족 — 고정 source와 실제 결과를 연결. 원 head 단독 CI·정량 성능·미실행 경로로 범위 확장하지 않음 |

## 시각 증적과 남은 조건

원 fork head는 변경하지 않았다. 실제 편람 수정 #8268·충돌 #8161은 제외하며 추가 PR을 체리픽하지 않는다.

## Merge 후 contributor comment 계획

현재는 게시·close·merge를 실행하지 않는다. 추후 승인된 통합 머지 뒤 최종 merge SHA·CI URL과
Visual Sweep 정본·SHA 고정 raw 이미지·원 기여와 보정 범위·남은 차이를 정중한 한국어로 안내한다.
작업지시자가 승인한 후속 처리로 본문 파일을 게시하고 API에서 실제 내용과 링크를 재확인한다.
