---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 최종 보고서 — W7 canonical font registry

## 1. 완료 판정

W7의 구현 목표는 충족됐다. 서로 다른 backend가 같은 face를 선택하도록 강제하지 않고, Rust layout,
Canvas2D paint, webfont supply와 CanvasKit SFNT의 유한 규칙을 하나의 canonical registry에서 각 결정면에
맞는 정적 projection으로 생성한다. 전환 전후 선택 결과·순서·metric·renderer output은 동일하다.

현재 결과는 **Stage W7-6 완료, 메인테이너 최종 결과 승인 대기**다. GitHub push·PR·issue close는 아직
수행하지 않았다.

## 2. 단계별 산출

| Stage | 산출과 판정 |
| --- | --- |
| W7-1 | 30 boundary·1,352 candidate, 600 metric anchor와 runtime projection 동결 |
| W7-2 | 830-rule schema·canonical registry·one-time migration manifest |
| W7-3 | Rust 2개·TypeScript 3개 generated projection과 paired manifest |
| W7-4 | Rust layout-name 171·layout-metric 67 runtime 전환, 전건 동등성 |
| W7-5 | Canvas2D 281·webfont 153·CanvasKit 158 runtime 전환, Studio 동등성 |
| W7-6 | full Rust·Studio·native·Docker WASM·SVG parity, 운영 문서와 최종 감사 |

W6의 600개 metric 값과 순서는 registry에 복제하지 않고 안정 `entryId` 97개 참조로 연결했다. Studio의
substitution 265행, 정부상징 successor 10행과 webfont catalog 153행 literal은 generated projection
소비로 바뀌었다.

## 3. 보호 불변식

| 불변식 | 결과 |
| --- | --- |
| relation·decision plane을 합치지 않음 | 충족 |
| active unknown 43개를 layout-metric legacy-preservation으로 유지 | 충족 |
| metric data 600개와 lookup 순서·width hash 불변 | 충족 |
| document 상태·local probe·glyph/capability를 hand-written owner에 유지 | 충족 |
| Canvas2D CSS supply와 CanvasKit SFNT capability 분리 | 충족 |
| runtime JSON parse·ledger search·추가 I/O 없음 | 충족 |
| generated `ruleId`와 W2 evidence join | 충족 |
| native/WASM SVG 0-delta | 173쪽 mismatch 0 |
| private corpus·host path·font bytes 비공개 | 충족 |

CanvasKit 153개 font plan 중 declared SFNT capability가 없는 125개에도 현행 online URL plan이 존재한다.
W7은 이 plan을 load 성공으로 승격하지 않고 capability와 계획을 계속 분리했다.

## 4. 최종 검증

- W1·W2·W6·W7 Node contract 77/77
- release library 4,075 pass·13 ignore
- release-test nextest 8,174/8,174, 정책 skip 39
- native-skia library, placeholder 2/2, direct PDF 4/4
- Clippy `-D warnings`, rustdoc, fmt 통과
- Studio TypeScript, 1,068 pass·1 skip, production build 통과
- Canvas2D·CanvasKit focused 38/38
- Docker optimized WASM과 fresh WASM trace 3/3
- 공개 HWP 7문서 167쪽 + 대표 HWP/HWPX 6문서 page 0, SVG byte mismatch 0

첫 nextest에서 새 generated Rust 파일의 schema version 리터럴 두 개가 중앙 스키마 계약에 걸렸다.
generator가 `src/schema_registry.rs`의 단일 상수를 참조하도록 정정하고 전체 8,174건을 다시 실행했다.
이 보정은 semantic projection hash를 바꾸지 않았다.

세부 명령·환경·hash는 [Stage W7-6 보고서](../working/task_m100_4966_w7_stage6.md)에 있다.

## 5. 운영 경계

schema 1.0은 W1/W6 일회 이행 결과 830개를 봉인한 read-only canonical 판이다. 현 판은 `active`만
허용하고 고정 수량·W1/W6 evidence를 강제하므로 JSON 직접 추가·수정·삭제를 지원하지 않는다. 생성기
결함 정정은 registry semantic bytes를 유지한 채 projection만 재생성한다.

실제 규칙 변경은 다음 schema 판을 별도 이슈·계획으로 승인한 뒤 수행한다. 추가 rule의 새 evidence,
identity가 바뀌는 수정, 삭제 대신 `retired` 상태·후속 rule·사유를 먼저 계약으로 만든다. 상세 절차는
[Issue #4966 조사 정본](../tech/investigations/issue-4966/README.md)에 있다.

## 6. W8 인계 조건

W8은 개별 mapping의 정확성을 보정하는 단계다. 시작 조건은 다음과 같다.

1. 변경 가능한 다음 registry schema와 evidence 계보를 승인한다.
2. 한 번에 한 decision plane과 한 backend projection만 의미상 바꾸는 change set을 만든다.
3. 변경 전후 `ruleId`, metric entry, selection tuple과 renderer output을 대사한다.
4. Canvas2D availability를 CanvasKit SFNT capability로, plan을 load 성공으로 승격하지 않는다.
5. public fixture와 필요한 경우 비식별 aggregate만 사용하고 private corpus 식별 정보는 내보내지 않는다.

이 조건 전에는 schema 1.0을 느슨하게 만들거나 generated source를 직접 수정하지 않는다.
