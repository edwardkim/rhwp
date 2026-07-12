# Task M100 #2124 - frontend SOLID evidence anchors

- 이슈: #2124
- 단계: Stage 4 - frontend SOLID evidence
- 작성일: 2026-07-10
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 기준 문서: `mydocs/manual/solid_scoring_guide.md`
- metrics snapshot: `mydocs/metrics/frontend/2026-07-11/metrics.json`
- 공식 SOLID 점수: 미채점

## 1. 목적과 한계

이 문서는 `solid_scoring_guide.md`의 5원칙 20점 앵커를 프론트 리뷰에 적용할 때 확인할 evidence를
고정한다. 함수 크기와 cognitive complexity는 SRP 검토 후보를 찾는 지표이지 SOLID 위반 또는 점수를
직접 증명하지 않는다. OCP/LSP/ISP/DIP는 계약, 대체 가능성, 소비자 범위, 의존 방향을 코드 리뷰로
확인해야 한다.

초기 초안의 `54/100`은 코드베이스 전체를 하나의 평가 단위로 삼고, 독립 reviewer calibration 없이
정량 hotspot에서 원칙별 점수를 추론했다. 이는 `solid_scoring_guide.md`에 따른 공식 기준선으로 재현하기
어려우므로 폐기한다. #2124에서는 evidence와 채점 절차만 고정하고 공식 점수는 남기지 않는다.

## 2. 정량 evidence

| 항목 | 값 |
|------|----:|
| 공식 snapshot 제품 파일 수 | 203 |
| 제품 함수 / CC가 보고된 함수 | 4,728 / 2,282 |
| Total CC / 전체 Top 20 합 | 11,805 / 2,581 |
| CC>25 함수 수 / 합 | 62 / 3,932 |
| CC>100 함수 수 / 합 | 6 / 1,732 |
| metrics parse/fatal diagnostics | 0 |
| Studio runtime code lines / Total CC | 54,007 / 9,505 |
| Studio CC>25 / Max CC | 46 / 453 |
| legacy `/web` code lines / Total CC | 5,795 / 828 |
| legacy `/web` CC>25 / Max CC | 4 / 86 |

모든 test/e2e/spec 파일은 제품 복잡도 모집단에서 제외했다. 이후 PR은 threshold 개수뿐 아니라 Total CC,
고복잡도 합, 안정적인 function id별 before/after diff를 함께 제시한다.

상위 hotspot:

| 파일 | 함수 | CC | LOC |
|------|------|---:|----:|
| `rhwp-studio/src/engine/input-handler-mouse.ts` | `onClick` | 453 | 995 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `onKeyDown` | 444 | 909 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | `handleOk` | 348 | 381 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | `populateFromProps` | 212 | 286 |
| `rhwp-studio/src/engine/input-handler-table.ts` | `finishResizeDrag` | 142 | 354 |

대형 파일 evidence:

| 파일 | 물리 줄 수 | 관찰할 책임 후보 |
|------|----------:|------------------|
| `rhwp-studio/src/ui/picture-props-dialog.ts` | 2,825 | UI state, DOM wiring, validation, apply logic |
| `rhwp-studio/src/core/wasm-bridge.ts` | 2,211 | WASM lifecycle, JSON adapter, public wrapper |
| `web/editor.js` | 2,216 | legacy editor endpoint와 DOM behavior |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 1,958 | mouse event, selection, resize/drag behavior |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | 1,942 | keyboard event, command routing, edit behavior |
| `web/font_substitution.js` | 715 | legacy font substitution |
| `rhwp-chrome/content-script.js` | 609 | extension detection, thumbnail, open flow |
| `rhwp-firefox/content-script.js` | 628 | Chrome과 유사한 browser surface |
| `rhwp-safari/src/content-script.js` | 614 | Safari-specific variant |

이 목록은 분리 지시가 아니라 review 진입점이다. 동작 응집도가 높거나 분리 비용이 더 큰 경우에는 큰
함수를 유지할 수 있으며, 반대로 작은 모듈도 의존 방향이 잘못되면 DIP 검토 대상이 된다.

## 3. 원칙별 review 질문

| 원칙 | frontend review 질문 | 필요한 evidence |
|------|----------------------|-----------------|
| SRP | 변경 이유가 다른 event routing, DOM wiring, persistence, WASM conversion, rendering policy가 한 단위에 섞였는가? | 변경 이력, 함수/모듈 책임 목록, 관련 test |
| OCP | 이미 존재하는 extension point가 있고, 새 variant가 안정된 코드를 반복 수정하지 않고 추가되는가? | adapter/policy 등록 지점, browser별 diff |
| LSP | 동일 interface를 구현하거나 대체 관계를 약속한 adapter/implementation이 pre/postcondition과 오류 의미를 보존하는가? | 공통 contract test, substitute별 결과 |
| ISP | 구체 consumer가 사용하지 않는 넓은 interface에 의존하고 변경 파급을 받는가? | consumer별 호출 surface와 type dependency |
| DIP | policy가 DOM, browser API, storage, generated WASM 같은 detail을 직접 소유하는가, 경계가 주입 가능한가? | import graph, adapter ownership, test double 가능성 |

`@rhwp/editor`, Studio, VS Code, browser extension은 서로 다른 public surface이므로 그 자체로 LSP의
상호 대체 대상이 아니다. LSP는 같은 interface를 구현하는 renderer, storage, browser adapter 또는
명시된 compatibility layer에만 적용한다. 표면 간 message/byte 차이는 contract 호환성 문제로 별도
검토한다.

## 4. 프론트 원칙별 채점 앵커

`solid_scoring_guide.md`의 공통 점수 정의를 다음 frontend evidence에 적용한다.

| 원칙 | 20점 | 16점 | 12점 | 8점 이하 |
|------|------|------|------|----------|
| SRP | 타입·모듈 경계와 gate가 책임 혼합을 막음 | 소수 국소 hotspot이 문서화·추적됨 | 여러 event/dialog/bridge hotspot에 변경 이유 누적 | God coordinator 수정 시 여러 layer 확인 필요 |
| OCP | 새 browser/backend/asset variant가 등록·adapter 추가로 끝남 | 소수 기존 build/policy 지점 수정이 명시됨 | variant마다 여러 target을 반복 수정 | target 추가가 핵심 흐름 재작성을 강제 |
| LSP | 같은 interface의 모든 구현이 공통 contract test를 통과 | 국소 구현 예외가 문서화·검증됨 | substitute별 의미 차이가 문서 없이 존재 | 호출부가 구체 구현을 확인해야 동작 |
| ISP | consumer별 최소 interface가 타입으로 강제됨 | 일부 넓은 surface가 추적됨 | bridge/type surface 때문에 미사용 API 의존이 반복됨 | 대부분 consumer가 거대 API/blob/`any`에 의존 |
| DIP | policy는 abstraction에 의존하고 DOM/WASM/browser/storage detail이 주입됨 | 직접 의존이 국소 adapter에 격리됨 | 고수준 흐름 여러 곳이 concrete detail을 직접 호출 | core policy 변경이 target별 수정으로 연쇄됨 |

4점 이하는 해당 원칙의 경계나 일관된 적용이 사실상 없는 경우다. 중간 점수는 두 앵커 사이 evidence를
명시해 보간한다. LSP는 서로 다른 product surface가 아니라 실제로 같은 interface를 구현하는 substitute만
평가한다.

## 5. 공식 채점 절차

| 단계 | 조건 |
|------|------|
| 평가 단위 고정 | 하나의 module, adapter family 또는 PR diff처럼 reviewer가 같은 코드를 볼 수 있어야 한다. |
| 계약과 책임 명시 | public contract, 허용된 동작 변화, module의 변경 이유를 먼저 적는다. |
| 정량 evidence 첨부 | metrics before/after와 관련 함수 diff를 제공하되 점수로 자동 변환하지 않는다. |
| 원칙별 독립 검토 | `solid_scoring_guide.md`의 0/4/8/12/16/20 anchor로 reviewer가 각각 근거를 남긴다. |
| 점수 조정 | maintainer/collaborator가 근거 차이를 조정하고 승인한 결과만 공식 점수로 기록한다. |

현재 상태:

| 원칙 | 점수 | 상태 |
|------|------|------|
| SRP | 미채점 | hotspot과 책임 후보만 수집 |
| OCP | 미채점 | extension point와 browser variant 검토 필요 |
| LSP | 미채점 | 실제 substitute 단위 선정 필요 |
| ISP | 미채점 | consumer별 surface 분석 필요 |
| DIP | 미채점 | import/dependency 방향 분석 필요 |
| 합계 | 미채점 | maintainer/collaborator review 전 공식 합계 없음 |

## 6. 리팩터링 guardrail

| 원칙 | guardrail |
|------|-----------|
| SRP | 코드를 단순 이동하거나 wrapper만 추가한 뒤 책임 분리와 총량 개선으로 주장하지 않는다. |
| OCP | 현재 browser 중복을 공통화하면서 variant 조건을 더 복잡한 중앙 분기로 옮기지 않는다. |
| LSP | 명시된 public contract의 field, message, byte shape, fallback을 migration 없이 변경하지 않는다. |
| ISP | consumer evidence 없이 하나의 거대 공통 interface를 새로 만들지 않는다. |
| DIP | frontend가 Rust schema를 재정의하거나 generated output을 source of truth로 취급하지 않는다. |

`@rhwp/editor`의 zero-runtime-dependency 제약은 package contract guardrail이며 ISP 점수를 자동으로
결정하지 않는다. React/Vue/Svelte 등 runtime UI framework 도입 금지는 SOLID 점수 산식이 아니라
#2023 v2에서 승인된 프론트 전체 v1.0 거버넌스 규칙이다.

## 7. 단계별 선택 원칙

Phase A의 승인된 첫 실행 단위는 #2125 font canonical location 정리다. extension security, giant handler,
WASM adapter를 먼저 수행하도록 이 문서가 순서를 바꾸지 않는다. #2125는 #2124의 local gate와 리뷰
승인이 끝난 뒤 착수한다.

Phase B 후보는 다음 요소를 함께 비교해 별도 하위 이슈로 선택한다.

| 요소 | 판단 |
|------|------|
| 감소 잠재력 | Total CC, 고복잡도 합, 함수 LOC, 중복량 |
| 계약 위험 | public API/message/schema/font/security 영향 |
| 검증 가능성 | unit/contract/E2E gate 보유 여부 |
| 책임 경계 명확성 | 분리 후 ownership과 dependency 방향 설명 가능 여부 |
| upstream 충돌 | Rust/core 진행 영역과 파일 충돌 가능성 |

## 8. 재측정 규칙

| 항목 | 기준 |
|------|------|
| metrics | schema v2 snapshot의 aggregate와 function diff |
| complexity | Total CC, Top 20, CC>25/100 count·sum, Max CC |
| public contract | `task_m100_2124_public_contract_snapshot.md` |
| WASM boundary | `task_m100_2124_wasm_json_schema_snapshot.md` |
| font | `task_m100_2124_font_inventory.md` |
| security | `task_m100_2124_extension_security_snapshot.md` |
| smoke | `task_m100_2124_smoke_manifest.md`의 변경 유형별 gate |
| SOLID | 선택한 평가 단위와 reviewer 근거; 숫자 자동 산출 금지 |

## 9. 결론

정량 evidence는 큰 함수와 집중된 module이 존재하며 구조 검토 가치가 있음을 보여준다. 그러나 프론트
전체의 공식 SOLID 점수는 아직 없다. #2124의 기준선은 성급한 숫자 대신 재현 가능한 metrics, 계약,
검증 gate와 reviewer 채점 절차를 제공한다.
