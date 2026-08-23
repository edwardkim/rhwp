---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-5 Canvas2D·webfont·CanvasKit 전환

## 1. 판정

Stage W7-5는 통과했다. Studio의 `SUBST_TABLES` 265행, 정부상징 successor 10행과
`FONT_LIST` 153행의 literal payload를 제거하고 canonical registry가 생성한 TypeScript projection을
소비하도록 전환했다. 기존 public resolver·supply·CanvasKit plan API의 반환 형태와 행동은 유지했다.

document `substFont`, 로컬 글꼴 탐색·probe, offline 필터, local asset 존재 검사, glyph coverage와 실제
SFNT byte 판정은 생성 규칙으로 옮기지 않았다. 이들은 현재 상태를 관측하는 hand-written 알고리즘이며,
유한 mapping과 공급 payload만 projection이 제공한다.

## 2. 소비자 경계

| projection | 생성 규칙 | Studio 소비 |
| --- | ---: | --- |
| Canvas2D paint | 281 | substitution 265, 정부상징 successor 10, display-chain 정책 5, canvas patch 정책 1 |
| Canvas2D webfont | 153 | family·URL·format·unicode-range 공급 catalog |
| CanvasKit SFNT | 158 | 공급 snapshot 153, substitute 3, plan 정책 1, SFNT byte capability 1 |

`font-rule-runtime.ts`는 세 projection을 서로 독립적으로 색인한다. `font-substitution.ts`와
`font-loader.ts`에는 W1 source selector가 소유자를 계속 찾을 수 있도록 얇은 alias만 남겼고, 기존 대형
배열 literal은 남기지 않았다. W1 candidate collector는 generated projection에서 원래 candidate identity를
재구성하므로 30개 boundary·1,352개 candidate의 원장 폐합과 기준선 drift 검사가 계속 동작한다.

## 3. 전환 전후 행동 동등성

W7-1에서 동결한 Studio runtime snapshot을 현재 consumer로 다시 실행했다. 다음 tuple 수와 hash가 모두
전환 전 값과 같다.

| 보호 대상 | 수량 | SHA-256 |
| --- | ---: | --- |
| substitution lookup | 265 | `013037ce38c8fb332357fc1a1e8bbe48f59bd0d2a8d57c138afdb136ef559024` |
| 정부상징 successor probe | 65 | `58f20a5526166623a73c6a661bceac8e7b6808597ef6e477a110c73a762bee79` |
| display fallback probe | 8 | `d03d324fe84bdc646d6377df83f8ffb2457e30a6f97765db6290805c64d09af6` |
| registered font catalog | 153 | `36da093ae28426cdedb750ddcd7e85dd672a2124d01233ba262051220b0b39e6` |
| webfont supply snapshot | 153 | `1bb8f0b160e46aae28edcc2607a3ee3b2061e5e77fa946db4797a0c927976a92` |
| webfont load snapshot | 153 requests | `a10b365a27c71e7852fe3e537ac19acd5299694c1e5027045feac7877ddd7071` |
| CanvasKit online·offline plan | 153 | `137b1eb63ddcf357278511c2576e1f62e7da50f4dae413c534e3f53c4fc86c9b` |

집중 회귀에서는 system font 우선순위, 확인되지 않은 로컬 이름 배제, document substitution, generic
fallback, 외부 웹폰트 비활성화, local asset 누락, URL alias grouping과 KoPub·한양중고딕 계획을 함께
검사했다. 기존 반환 API는 generated `ruleId`를 제거한 과거 형태를 유지하고, W2 trace만 상세 API를 통해
실제 선택에 참여한 rule ID를 받는다.

## 4. plan과 capability의 분리

CanvasKit projection의 font-list 153행 중 28행만 SFNT 공급 capability가 선언돼 있고, 125행은
`unavailable`이다. 그러나 기존 `resolveCanvasKitFontPlan`은 URL 확장자나 실제 byte 검증 전에 모든
공급 URL을 계획 후보로 만들었다. 따라서 이 125행에도 online plan source가 존재한다.

`canvasKitSfntPlanned`를 선언 capability로 바꾸면 기존 16개 기준선 판정이 달라졌다. 이 단계에서는 의미를
승격하지 않고 기존 계약대로 **URL 확장자 기반 계획 신호**를 보존했다. 실제 SFNT 성공은 renderer가 받은
byte와 W2 backend snapshot으로 별도 판정한다. 따라서 Canvas2D webfont 공급이나 plan 존재만으로
CanvasKit 사용 성공을 주장하지 않는다.

## 5. provenance와 generated hash

TypeScript projection에도 Rust와 같은 단일 `sourceBoundaryId`를 포함했다. runtime은 boundary별 유한
규칙을 안전하게 분리하고, Studio trace는 Canvas2D paint·webfont와 CanvasKit supply·substitute·policy의
`ruleId`가 실제 generated 집합에 존재하는지 검사한다.

| 항목 | SHA-256 |
| --- | --- |
| generator | `4ffa352261d006ff60bf8abda19c64b5c4e39ba11b1e2385bdaf79103bc54c4d` |
| content bundle | `07cc556414620e4361c2cf0efb85422d27487216a5500b2e7c5a571ebf612920` |
| projection bundle | `533c1ea77d70658be513b62bd77fb631c5099703bef4c4fdfb8629fd477c1ac8` |
| Canvas2D paint | `c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4` |
| Canvas2D webfont | `730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee` |
| CanvasKit SFNT | `d9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668` |

## 6. 원격 통합과 검증

작업 중 최신 `upstream/devel@87a8d3dca`의 #5944·#5945 변경을 로컬 task branch에 merge했다. 제품
source의 겹침은 없었고 오늘할일 문서만 자동 병합됐다. 통합 뒤 다음 검증을 통과했다.

- projection generator·registry·pre-migration semantic baseline check: 통과
- W1·W2·W6·W7 Node contract: 61/61
- Studio consumer 집중 회귀: 23/23
- Studio 전체 Node test와 production TypeScript/Vite build: 통과
- `git diff --check`: 통과

## 7. Stage W7-6 인계

다음 단계는 새 기능 이동이 아니라 최종 통합 검증과 운영 인계다. 전체 Rust test·Clippy, Studio
production build, native와 Docker WASM build, 공개 fixture의 native/WASM parity를 실행한다. 이어서 규칙
추가·수정·폐기 절차와 canonical fallback·Decision Trace 문서를 새 registry authority에 맞게 갱신하고
최종 보고서를 작성한다.
