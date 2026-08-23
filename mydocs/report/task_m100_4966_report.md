---
kind: report
status: active
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 최종 보고서 — W7 canonical font registry

## 1. 현재 판정

W7의 구현 목표는 충족됐다. 서로 다른 backend가 같은 face를 선택하도록 강제하지 않고, Rust layout,
Canvas2D paint, webfont supply와 CanvasKit SFNT의 유한 규칙을 하나의 canonical registry에서 각 결정면에
맞는 정적 projection으로 생성한다. 전환 전후 선택 결과·순서·metric·renderer output은 동일하다.

최초 로컬 결과와 메인테이너 승인은 있었지만, PR #5950의 CI가 PR-base unit-tier 정책 위반을 발견해
**완료 판정을 철회하고 Stage W7-R 영향도 재감사로 되돌렸다**. 최신 CI, self-review, merge와 issue
close는 근본 정정과 전체 재검증 뒤의 후속 승인 게이트다.

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
- release library 4,074 pass·13 ignore
- release-test nextest 8,201/8,201, 정책 skip 41
- native-skia library, placeholder 2/2, direct PDF 4/4
- Clippy `-D warnings`, rustdoc, fmt 통과
- Studio TypeScript, 1,070 pass·1 skip, production build 통과
- Canvas2D·CanvasKit focused 38/38
- Docker optimized WASM과 fresh WASM trace 3/3
- 공개 HWP 7문서 167쪽 + 대표 HWP/HWPX 6문서 page 0, SVG byte mismatch 0

첫 nextest에서 새 generated Rust 파일의 schema version 리터럴 두 개가 중앙 스키마 계약에 걸렸다.
generator가 `src/schema_registry.rs`의 단일 상수를 참조하도록 정정하고 전체 8,174건을 다시 실행했다.
그 뒤 `upstream/devel@343ed2c013606319b6418dd8c637c5e04047e304`을 병합하고 늘어난 integration
source를 포함한 8,201건을 다시 전건 통과했다. 이 보정은 semantic projection hash를 바꾸지 않았다.

PR #5950의 최초 head `4a7c0f431`에서 CI lint가 기존 함수에 새로 붙인 `#[cfg(test)]`를 신규 test
support 6개로 판정했다. 로컬 사전 검사가 `--base-ref upstream/devel` 없이 현재 inventory만 확인해 이를
놓쳤다. 수기 함수를 runtime scope로 되돌려 통과시키는 최초 대응은 가드레일 우회이므로 커밋하지 않고
철회했다.

재감사에서 W1 `sourceCommit`의 Rust 파일 2개는 snapshot SHA-256과 일치했고, W1 Rust 후보 238개와
canonical registry 사이의 candidate ID·boundary·조건·source·target·order 불일치는 0건이었다. 반면
W3 계약은 W6의 metric table selector가 고정 배열에서 composed view로 이동한 sourceLocation 변화만으로
600개 후보를 의미 변경으로 오판했고, W7의 77개 검증 묶음은 이 W3 계약을 포함하지 않았다. 따라서 현재
증거는 font 선택 회귀보다 **역사 계측과 현재 authority의 수명주기 혼합 및 교차 단계 검증 누락**을
원인으로 가리킨다.

기존 전체 검증의 명령·환경·hash는 [Stage W7-6 보고서](../working/task_m100_4966_w7_stage6.md), 재감사와
근본 정정 절차는 [Stage W7-R1 기록](../working/task_m100_4966_w7_rework_stage1.md)에 있다.

W7-R2·R3 정정에서는 W1의 30개 boundary·1,352개 candidate를 현재 checkout이 아니라 기록된 Git blob에서
검증하도록 바꿨고, 600개 metric source 이동을 의미 회귀와 분리했다. 제품 source의 전환 전 수기 mapping과
oracle helper는 제거하고 `tests/cases/issue_4966_font_rule_projection.rs`에서 public trace 171개
(직접 관측 137·우선순위 shadow 34)와 metric alias 67개를 검증한다. focused 결과는 W1·W2·W3·W6·W7
87/87, source Rust 35/35, integration 2/2, PR-base unit-tier와 Clippy 통과다. 세부 내용은
[Stage W7-R2·R3 기록](../working/task_m100_4966_w7_rework_stage2.md)에 있다.

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
