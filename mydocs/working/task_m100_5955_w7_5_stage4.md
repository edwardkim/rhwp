---
kind: report
status: completed
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-4 lifecycle audit와 W2 trace join

## 1. 판정

Stage W7.5-4 구현과 검증을 완료했다. v2 registry를 한 번 검증해 immutable lookup index를 만드는 lifecycle
resolver와, W2 Font Decision Trace의 rule reference를 별도 JSON으로 설명하는 offline API·CLI를 추가했다.
runtime trace envelope, trace hash, renderer 선택과 document state는 변경하지 않았다.

resolver는 carried-forward active, 후속 change set이 도입한 active, retired, successor가 있는 replaced와
dangling을 구조화한다. 실제 W2 계보를 조사하면서 v2 밖의 정상 규칙을 dangling과 구분하기 위해
`historical-reference-only`와 `trace-declared-source-drift`를 추가했다. 두 판정은 registry rule이라고 승격하지
않고 어떤 역사·trace 근거로 lifecycle 밖에 있는지만 설명한다.

## 2. RED 계약과 API 경계

먼저 output schema와 focused contract를 만들었고 audit module이 없는 상태에서
`ERR_MODULE_NOT_FOUND`로 RED를 확인했다. 계약은 다음을 고정한다.

- Rust `records[].provenance[].ruleId`와 Studio `records[].paint.*.ruleIds[]`를 원래 순서로 수집
- 같은 trace·registry 입력은 byte-identical canonical audit 생성
- input trace 객체를 clone하거나 수정하지 않음
- 830 current rule 전부 carried-forward active로 resolve
- synthetic add, retire, retire-and-replace와 근거 없는 dangling 판정
- successor cycle, cross-plane replacement와 evidence dangling을 분류 전에 거부
- caller-selected output, symlink, 민감 경로형 record ID와 상한 위반 거부

`createRuleLifecycleResolver`는 registry와 역사 ledger를 한 번만 검증하고 여러 reference를 O(1) map lookup으로
처리한다. 단일 조회용 `resolveRuleLifecycle`도 같은 검증 경계를 사용한다.

## 3. W1 reference-only와 W2 source drift 정정

초기 구현은 v2 registry에 없는 모든 ID를 dangling으로 분류했다. 그러나 실제 공개 missing-face HWP trace의
1,784개 reference 중 metric-entry·measurement 등 W7 reference-only rule이 1,211개였다.

첫 정정은 봉인 v1 registry가 기록한 W1 ledger path와 SHA-256을 검증하고, ledger 1,507개 중 v2 lifecycle에
들어오지 않은 677개 ID를 `historical-reference-only`로 분류한다. W1 전건 synthetic trace는 830
carried-forward + 677 historical reference-only, dangling 0으로 닫혔다.

남은 3개 고유 metric-entry ID는 W2가 이미 `reason: ledgerSourceDrift`로 명시한 Stage 2 이후 identity였다.
이를 registry rule이나 W1 historical rule로 추정하지 않고 `trace-declared-source-drift`로 분리했다. 선언 없는
미등록 ID는 계속 dangling이다.

## 4. 실제 공개 trace 판정

repository-tracked missing-face HWP의 page 0을 기존 `rhwp-q-font-trace`로 읽고 audit API에 직접 전달했다.
입력 파일이나 trace JSON을 artifact로 저장하지 않았다.

| 항목 | 결과 |
| --- | ---: |
| trace status | complete |
| records | 607 |
| rule reference | 1,784 |
| unique rule ID | 19 |
| carried-forward active | 573 |
| historical reference-only | 802 |
| trace-declared source drift | 409 |
| introduced / retired / replaced | 0 / 0 / 0 |
| 근거 없는 dangling | 0 |

수량은 unique rule 수가 아니라 trace 위치별 reference occurrence다. 동일 metric decision이 여러 문자에서
반복되면 각각 원래 JSON pointer와 record ID를 유지한다.

## 5. schema·보안 계약

audit schema는 registry·historical ledger digest, trace record/reference 수, 분류 summary,
`referencesSha256`과 각 reference의 lifecycle reason을 고정한다.

- CLI 입력 파일 최대 16 MiB, symlink·directory 거부
- trace record 최대 4,096
- record당 provenance 최대 64, backend당 rule ID 최대 4,096
- 전체 rule reference 최대 262,144
- record/rule ID 최대 2,048자 stable identifier
- host input path와 trace source 내용은 output에 포함하지 않음
- unknown lifecycle graph나 evidence 손상은 partial audit 없이 fail-closed

JSON Schema Draft 2020-12 meta-schema와 실제 generated audit validation을 통과했다.

## 6. 검증 결과

| 검증 | 결과 |
| --- | --- |
| lifecycle audit focused contract | 8/8 통과 |
| W2 + lifecycle + v2 registry + projection Node contract | 55/55 통과 |
| Studio font decision trace focused test | 5/5 통과 |
| canonical v2 registry·projection check | 통과 |
| actual public trace lifecycle join | dangling 0 |
| input trace mutation | 0 |

Stage W7.5-3에서 기록한 metric-lineage evidence digest의 사전 존재 drift는 이번 단계에서도 수정하지 않았다.
audit는 봉인 W1 ledger digest만 사용하며 W6 lineage manifest나 제품 metric을 재생성하지 않는다.

## 7. 보호 불변식 self-review

- runtime trace schema·Rust query·WASM/Studio public API를 변경하지 않았다.
- audit는 font 선택, metric lookup, paint·supply projection을 호출하거나 바꾸지 않는다.
- retired rule을 삭제하지 않고 successor·predecessor를 그대로 설명한다.
- historical reference-only와 trace source drift를 active lifecycle로 승격하지 않는다.
- 근거 없는 ID를 추정하지 않고 dangling으로 남긴다.
- private corpus, font bytes와 host path를 사용하거나 기록하지 않았다.
- actual mapping·projection generated source와 registry artifact를 변경하지 않았다.

## 8. 다음 경계

결과 승인을 받으면 Stage W7.5-4 변경과 보고서를 한 경계 커밋으로 고정한다. Stage W7.5-5는 canonical 제품
registry를 바꾸지 않는 synthetic fixture에서 evidence-only, add, retire, retire-and-replace의 pre/post
projection delta와 rollback을 rehearsal한다. remote push는 별도 승인 대상이다.
