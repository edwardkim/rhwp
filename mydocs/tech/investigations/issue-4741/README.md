---
kind: investigation
status: active
canonical: mydocs/tech/font_fallback_strategy.md
last_verified: 2026-08-15
---

# Issue #4741 — Local Font Access 부분 열거 누락 조사

## 결론

#4741은 KoPub 이름 목록이 빠진 문제가 아니라 로컬 글꼴 감지 상태 모델의 세 번째 경우가 누락된
문제다.

| 상태 | 기존 모델 | 필요한 모델 |
| --- | --- | --- |
| API 지원, 후보 열거됨 | Local Font Access complete | exact-enumerated |
| API 미지원 | 문서 후보 Canvas probe | exact-probed 또는 unresolved |
| **API 지원, 후보 일부 누락** | **complete로 오판, probe 생략** | **미해소 후보만 raw probe** |

Chrome 150 재현에서는 `queryLocalFonts()`가 748개를 반환했지만 KoPub은 0개였고, raw FontFace와
Canvas2D는 설치된 `KoPub바탕체 Light`를 사용했다. 따라서 함수 존재와 성공 응답만으로 문서 후보에
대한 완전성을 주장할 수 없다.

## 확인한 코드 경로

### `local-fonts.ts`

- `DetectLocalFontsOptions.candidateFamilies`는 “Local Font Access API가 없는 브라우저” 전용으로
  문서화되어 있다.
- `detectLocalFonts()`는 `isLocalFontAccessSupported()`가 참이면 열거 snapshot만 만들고,
  후보 probe는 `else if`에서만 실행한다.
- `getLocalFontState()`는 snapshot source가 `local-font-access`이면 `complete: true`로 반환한다.
- Canvas probe는 `context.font = ...`로 후보와 fallback 폭을 비교한다.

이 분기는 `3f9595c0c`(`Task #1328: Stage 1 local font state model`)에서 도입됐다. 이후 #2217의
다국어 이름·style 레코드 보강은 열거된 `FontData`의 품질을 개선했지만, 열거되지 않은 설치 face를
발견하는 분기 자체는 바꾸지 않았다.

### `wasm-bridge.ts`

`installCanvasFontSubstitution()`은 `CanvasRenderingContext2D.prototype.font` descriptor를 바꿔
모든 setter 입력에 `fontFamilyChainForDisplay()`를 적용한다. 원래 descriptor는 함수 closure 안에만
남는다. 이 patch 설치 후 일반 `context.font = ...`로 presence probe를 수행하면 미확인 후보가 먼저
fallback으로 치환될 수 있다.

따라서 부분 열거 보완은 기존 probe 호출 조건만 넓혀서는 안 된다. 원래 descriptor의 setter를
명시적으로 쓰거나 제품 patch와 격리한 raw Canvas 계약이 필요하다.

## 문서·테스트에서 확인한 누락 경로

### #1328의 설계 전제

계획과 작업 기록은 Chrome/Edge에서 Local Font Access 전체 목록, Firefox에서 문서 후보 probe라는
플랫폼 이분법을 반복한다. “API가 존재하지만 일부 설치 face가 빠짐”은 상태·위험·수용 기준에 없다.

### 단위 테스트의 partition gap

현재 `local-fonts.test.ts`에는 `queryLocalFonts = undefined`일 때 문서 후보만 probe snapshot으로
저장하는 테스트가 있다. API가 빈 배열이나 다른 face만 반환하면서 raw Canvas에서는 후보를 사용할 수
있는 테스트는 없다.

### #4739의 의도적 비범위와 환경 이동

#4739 계획은 불완전 `queryLocalFonts()` 보완을 제외하고 구현 뒤 남는 재현을 #4741에서 처리한다고
명시했다. #4739 최종 검증의 Chrome 151은 KoPub 6개 face를 모두 snapshot에 제공했다. Chrome 150의
부분 열거 음성 조건이 없어졌으므로, 정상 환경 E2E만으로는 #4741 잔여를 검출할 수 없었다.

### 운영 인계 누락

#4739가 merge된 뒤 #4741은 OPEN이었지만 담당자와 연결 PR이 없었다. 선행 계획의 비범위 표시는
기술적 분리에는 성공했으나, merge 후 관련 이슈를 재조회해 owner와 next action을 고정하는 절차가
없어 후속 처리가 자동으로 이어지지 않았다.

## 다른 폰트에 재발할 수 있는 유형

| 유형 | 예 | 잘못된 완료 신호 | 필요한 판정 |
| --- | --- | --- | --- |
| 부분 열거 | KoPub이 LFA 결과에 없지만 raw Canvas 사용 가능 | API 성공, count가 큼 | 문서 후보 exact raw probe |
| 지역화 이름 | `08서울한강체 M` 대 `08SeoulHangang M` | family 하나가 열거됨 | name table alias/full/PostScript 대조 |
| style face 누락 | Light만 빠지고 Regular/Bold만 열거 | family 존재 | style/weight별 face 해소 |
| family 선점 | custom loader가 Regular 하나로 Bold까지 처리 | glyph가 보임 | backend별 style matching |
| variable font 축 소실 | family는 같으나 weight/width 축 붕괴 | 동일 family 문자열 | axes와 실제 face key 분리 |
| blob 실패 | 메타데이터는 있으나 `FontData.blob()` 불가 | Canvas2D 성공 | CanvasKit SFNT 조달을 별도 판정 |
| probe 모호성 | 후보 폭이 fallback 폭과 우연히 동일 | width delta 0 | 여러 vector/fallback, ambiguous 유지 |
| 전역 patch 순환 | probe 입력이 먼저 fallback으로 치환 | probe가 실행됨 | raw setter/격리 realm 증명 |
| stale snapshot | 다른 browser/version 결과 재사용 | 저장 snapshot 존재 | version/generation/candidate scope 확인 |
| fallback 분류 오판 | `바탕체`를 monospace로 분류 | chain이 유효 CSS | 분류와 exact face를 별도 검증 |

## backend 경계

Canvas2D와 CanvasKit은 로컬 폰트를 조달하는 능력이 다르다.

- Canvas2D는 브라우저가 CSS 이름으로 face를 해석하면 실제 TTF/OTF 바이트 없이 그릴 수 있다.
- CanvasKit은 local Typeface 등록에 SFNT 바이트가 필요하다. raw Canvas probe 양성만으로 바이트를
  만들거나 `FontData.blob()` 성공을 추정할 수 없다.
- 따라서 `exact-probed`는 Canvas2D 사용 가능 상태이며 CanvasKit 사용 가능 상태가 아니다.

한 backend의 성공을 다른 backend 완료 근거로 사용하면 #2206/#2217에서 이미 분리했던
“레이아웃 메트릭, 이름 해소, 실제 glyph typeface” 축이 다시 섞인다.

## 재현과 검증에 필요한 상태 매트릭스

향후 폰트 이슈는 최소한 다음 네 열거 fixture를 갖는다.

1. API 미지원
2. API 지원 + exact face 열거
3. API 지원 + family/alias만 열거하고 style face 누락
4. API 지원 + 후보 전체 누락, raw Canvas exact face 사용 가능

각 fixture에서 다음 결과를 함께 기록한다.

- snapshot source와 candidate coverage
- exact/alias/style 해소 결과와 provenance
- raw Canvas와 patched Canvas의 effective font/폭
- probe 호출 횟수와 cache generation
- CanvasKit SFNT bytes/Typeface 등록 여부
- fallback chain과 unresolved/ambiguous 이유

브라우저 버전 변화로 자연 재현이 사라질 수 있으므로 3·4번은 CDP 실행 환경에서도 test harness로
강제할 수 있어야 한다. 실제 환경 관찰만으로 회귀 게이트를 대신하지 않는다.

## 장기 절차로 승격할 항목

이 조사에서 확정한 다음 절차는 #4741 구현 단계에서
`mydocs/manual/font_incident_response.md`로 승격한다.

1. 환경·원문·열거·probe·선택·backend·oracle의 7축 진단 매트릭스
2. 부분 열거·localized alias·style 누락·blob 실패를 포함한 RED fixture
3. 공개 폰트와 비공개/재배포 불가 폰트의 자산 경계
4. 관련 이슈 disposition과 PR 전·merge 후 owner/next-action 재확인

감지 및 fallback의 장기 기술 계약은
[`font_fallback_strategy.md`](../../font_fallback_strategy.md)에 반영하고, 이 문서는 #4741 당시의
원인과 증거를 보존한다.

## 관련 문서

- [#4741 수행계획](../../../plans/task_m100_4741.md)
- [#4739 조사](../issue-4739/README.md)
- [폰트 fallback 전략](../../font_fallback_strategy.md)
- [Studio CDP 가이드](../../../manual/e2e-cdp.md)
