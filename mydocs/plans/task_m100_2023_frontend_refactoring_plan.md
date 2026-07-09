# 프론트 웹 리팩터링 계획 초안 — Task M100 #2023

- 이슈: #2023 (umbrella: #2022)
- 작성일: 2026-07-07
- 기준 커밋: `upstream/devel` `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`
- 상태: **초안 — maintainer/collaborator 리뷰 요청 전**
- 거버넌스: **SOLID + 복잡도**
  - SOLID: `mydocs/manual/solid_scoring_guide.md`를 프론트 구조에 맞게 재해석
  - 복잡도: `mydocs/tech/task_m100_2023_frontend_diagnosis.md`의 예비 baseline 사용
  - guardrail: `mydocs/tech/task_m100_2023_frontend_contract_guardrails.md`

## 0. 대원칙 — 프론트도 빅뱅 리팩터링 금지

프론트 웹 리팩터링은 `@rhwp/editor`만의 문제가 아니다. `rhwp-studio`, `/web` legacy,
브라우저 확장, VS Code extension, npm 문서·패키지 계약이 같은 자산과 메시지 계약을 공유한다.
따라서 한 번에 폴더 이동·UI 구조 변경·확장 공통화·API 변경을 묶으면 회귀 원인 판정이
불가능해진다.

원칙:

1. **계약 정리 → baseline freeze → 작은 실행 이슈** 순서로 진행한다.
2. 각 실행 이슈는 독립적으로 종료 가능한 상태여야 한다.
3. 코드 이동과 동작 수정은 같은 PR에 섞지 않는다.
4. `/web` legacy 삭제는 `web/fonts` ownership 이전 후에만 검토한다.
5. `@rhwp/editor`는 주 타깃이 아니라 public embed guardrail로 취급한다.
6. React/Vue/Svelte 등 runtime UI framework 도입은 이번 리팩터링 범위에서 금지하는 방향으로
   리뷰를 요청한다.

## 1. 현재 영점

상세 수치: `mydocs/tech/task_m100_2023_frontend_diagnosis.md`

| 항목 | 측정값 | 판정 |
|------|-------:|------|
| `rhwp-studio/src` 코드 LOC | **53,525** | 대형화 |
| `rhwp-studio/src` 1,200 LOC 초과 파일 | **10** | 해체 대상 |
| `rhwp-studio/src` 2,000 LOC 초과 파일 | **3** | 최상위 해체 대상 |
| `rhwp-studio/src` CC>25 함수 | **49** | S 축 하락 근거 |
| `rhwp-studio/src` 최대 CC | **321** (`onKeyDown`) | 최우선 |
| legacy `/web` 코드 LOC(font 제외) | **4,550** | 삭제/보존 판단 필요 |
| `rhwp-shared` CC>25 함수 | 0 | 공통화 방향은 양호 |
| `@rhwp/editor` 코드 LOC | 241 | 리팩터링 타깃 아님, 계약 guardrail |

SOLID 예비 점수: **54/100**

| 원칙 | 예비 점수 | 핵심 근거 |
|------|----------:|-----------|
| S | 8/20 | God coordinator/function 다수, `/web`와 font asset ownership 혼재 |
| O | 12/20 | asset/font/build 변경 시 studio/extension/VS Code/npm 동시 수정 |
| L | 14/20 | renderer/e2e 계약은 있으나 embed/webview/font 계약 문서화 부족 |
| I | 8/20 | `core/types.ts` exports 105, `wasm-bridge.ts` `any` 151, `this: any` 다수 |
| D | 12/20 | thin wrapper는 좋지만 asset/platform detail 의존이 강함 |

## 2. 금지 목록 — 이번 계획이 확정되기 전 하면 안 되는 것

아래 항목은 계획 리뷰 전 실행 금지다.

- `/web` 전체 삭제.
- `web/fonts` 위치 변경.
- `rhwp-studio/public/fonts` symlink 변경.
- Chrome/Firefox/Safari/VS Code 빌드 경로 변경.
- `@rhwp/editor` iframe/postMessage 계약 변경.
- `@rhwp/editor` runtime dependency 추가.
- React/Vue/Svelte 등 runtime UI framework 도입.
- 확장 CSP 완화.
- inline script 추가.
- `web_accessible_resources` 확대.
- sender/URL/file signature/size 검증 약화.
- 폰트 fallback 변경.
- 저작권 보호 대상 한컴/MS 폰트 번들.
- 대형 모듈 분해와 버그 수정 혼합.

판단이 애매하면 보류하고 별도 실행 이슈로 분리한다.

## 3. Phase 0 — Frontend Baseline Freeze

목표: 실행 이슈 착수 전, 회귀 판정 기준을 먼저 고정한다.

| baseline | 등급 | 내용 |
|----------|------|------|
| 정량 복잡도 baseline | 필수 | 2단계 진단 스크립트 재실행 결과 보관 |
| public contract snapshot | 필수 | `@rhwp/editor` API, `@rhwp/core` README 계약, VS Code message type |
| font asset inventory | 필수 | `web/fonts` 파일 목록, license 문서, 참조 경로 |
| extension security snapshot | 필수 | manifest CSP, `web_accessible_resources`, shared security helper |
| studio render/e2e smoke | 필수 | `npm run build`, 주요 e2e/render-diff 중 실행 가능 항목 |
| extension unpacked smoke | 실행 이슈별 필수 | Chrome/Firefox viewer load, CSP/404 console 확인 |
| VS Code packaging smoke | font/wasm 경로 변경 시 필수 | webpack build, webview load, font/WASM 확인 |

Phase 0 산출 후보:

- `mydocs/tech/task_m100_{issue}_frontend_baseline.md`
- `mydocs/metrics/frontend_task_{issue}_baseline.json`

## 4. Phase A — `/web` font ownership 정리

성격: 저위험이지만 배포 표면이 넓다. 가장 먼저 분리해야 한다.

목표:

- legacy `/web`와 실사용 font asset root를 분리한다.
- `/web` 삭제 가능성을 만들되, 이 phase에서는 삭제를 기본 목표로 삼지 않는다.

실행 하위 이슈 후보:

| 후보 | 작업 | 성공 기준 |
|------|------|-----------|
| A1 | `web/fonts` canonical 위치 결정 | maintainer 승인. 새 위치와 문서 경로 확정 |
| A2 | studio/extension/VS Code build path 이전 | 웹앱·Chrome·Firefox·Safari·VS Code에서 font 404 없음 |
| A3 | font/license/npm 문서 경로 갱신 | `FONTS.md`, `THIRD_PARTY_LICENSES.md`, npm README 경로 일치 |
| A4 | legacy `/web` 삭제/보존 판단 | `web/fonts` 외 실사용 0 확인 후 별도 삭제 이슈 등록 |

권장 canonical 후보:

1. `rhwp-studio/public/fonts`를 실체 디렉터리로 승격.
2. 저장소 공통 asset root(예: `assets/fonts`) 신설.

판단 기준:

- `rhwp-studio` 기본 public path와 가까운 것은 1안.
- 확장/VS Code/npm이 함께 쓰는 공통 asset임을 드러내는 것은 2안.
- 어떤 안이든 `/web/fonts`는 canonical이 아니어야 한다.

금지:

- `/web` legacy 삭제와 font path 이전을 같은 PR에 섞지 않는다.
- 폰트 파일 추가/삭제와 경로 이전을 같은 PR에 섞지 않는다.
- render/layout 차이를 만드는 font fallback 변경을 경로 이전 PR에 섞지 않는다.

## 5. Phase B — `rhwp-studio` 복잡도 해체

성격: 중위험. public contract와 render behavior를 유지하는 순수 구조 리팩터링만 허용한다.

### B1. InputHandler typed context

문제:

- `input-handler.ts` 4,104 LOC.
- `onKeyDown` CC 321, `onClick` CC 227, `onMouseMove` CC 97.
- 위임 모듈에 `this: any`가 다수 존재.

목표:

- `this: any` 위임 구조를 제거한다.
- keyboard/mouse/table/text/picture handler별 최소 context interface를 정의한다.
- 대형 이벤트 함수는 command routing, selection, table, picture, navigation 책임으로 나눈다.

성공 기준:

- `this: any` 사용 0 또는 예외 등재.
- `onKeyDown`, `onClick` CC>100 해소.
- 기존 keyboard/mouse/table/picture e2e 통과.
- public command behavior 불변.

### B2. Dialog state/apply 분리

문제:

- `picture-props-dialog.ts` 2,466 LOC.
- `handleOk` CC 275, `populateFromProps` CC 159.
- `para-shape-dialog.ts::collectMods` CC 132.
- `char-shape-dialog.ts::collectMods` CC 93.

목표:

- DOM build, state hydrate, validation, command apply, diff/mod collection을 분리한다.
- dialog별 state model과 apply adapter를 명확히 둔다.

성공 기준:

- `PicturePropsDialog.handleOk` CC>100 해소.
- dialog visual/interaction smoke 통과.
- generated command payload 동등성 확인.

### B3. WasmBridge와 type surface 분리

문제:

- `wasm-bridge.ts` 1,883 LOC, `any` 151.
- `core/types.ts` exports 105.
- WASM JSON boundary와 studio 내부 타입이 섞인다.

목표:

- WASM raw response type, normalized studio type, UI view model을 분리한다.
- `core/types.ts`를 도메인별 type module로 나눌 후보를 확정한다.
- `@rhwp/core`/`@rhwp/editor` 공개 계약과 내부 타입을 분리한다.

성공 기준:

- `wasm-bridge.ts` `any` 감소.
- `core/types.ts` export 수 감소 또는 도메인별 명시.
- npm/extension/VS Code public contract 불변.

### B4. Diff engine 분해

문제:

- `diff-engine.ts` 2,907 LOC.
- snapshot extraction, identity matching, text diff, visual diff 책임이 집중되어 있다.

목표:

- snapshot builder, identity matcher, text diff, visual diff, report formatter를 분리한다.

성공 기준:

- `diff-engine.ts` 1,200 LOC 이하 또는 단계적 감소.
- 기존 compare e2e/report 동등성 유지.

## 6. Phase C — 확장·VS Code·npm 계약 정리

성격: 보안과 배포 계약이 우선인 phase다. 복잡도보다 guardrail 유지가 중요하다.

### C1. Extension commonization

목표:

- `rhwp-shared`의 낮은 복잡도(CC>25 0)를 유지하며 보안 helper 중심으로 공통화한다.
- Chrome/Firefox/Safari lifecycle 차이는 보존한다.

후보:

- `sw/message-router.js`
- `sw/fetch-security.js`
- `sw/viewer-launcher.js`
- content-script hover card/rendering helper
- build copy manifest checklist

성공 기준:

- CSP 완화 없음.
- `web_accessible_resources` 확대 없음.
- sender/URL/file 검증 약화 없음.
- Chrome/Firefox/Safari smoke 통과.

### C2. VS Code webview contract 정리

목표:

- font canonical 이전 후 VS Code webpack copy 경로를 안정화한다.
- webview message type과 WASM/font asset 경로를 문서화한다.

성공 기준:

- nonce 기반 CSP 유지.
- `localResourceRoots` 확대 없음.
- HWP open, SVG export, debug overlay smoke 통과.

### C3. npm contract docs 정리

목표:

- `@rhwp/editor` README의 self-hosted `web/fonts` 안내를 새 canonical 위치로 갱신한다.
- `@rhwp/core` README와 실제 WASM/package surface의 불일치 여부를 점검한다.

성공 기준:

- README 예제와 실제 파일 경로 일치.
- `@rhwp/editor` package dependency 없음 유지.
- iframe/postMessage API 불변.

## 7. Phase D — 6차 프론트 리뷰와 점수 재평가

목표:

- #2023 영점 대비 개선 여부를 SOLID + 복잡도 기준으로 재평가한다.
- 실행 하위 이슈에서 남은 부채를 다음 사이클로 넘긴다.

측정:

- `rhwp-studio/src` 1,200 LOC 초과 파일 수.
- `rhwp-studio/src` CC>25 함수 수.
- `this: any` 수.
- `wasm-bridge.ts` `any` 수.
- `core/types.ts` export 수.
- `/web` legacy 잔여 여부.
- extension duplicated file count.

목표치:

| 지표 | 영점 | 1차 목표 | 장기 목표 |
|------|-----:|---------:|---------:|
| `rhwp-studio/src` 최대 CC | 321 | < 150 | < 100 |
| `rhwp-studio/src` CC>25 | 49 | 상위 10 해소 | 0 또는 예외 등재 |
| 1,200 LOC 초과 파일 | 10 | 5 이하 | 0 또는 예외 등재 |
| `this: any` hotspot | 79 | 20 이하 | 0 또는 예외 등재 |
| SOLID 예비 점수 | 54/100 | 70 이상 | 85 이상 |

## 8. 실행 하위 이슈 분리안

계획 리뷰 후 바로 만들 후보는 다음 순서가 적절하다.

| 순서 | 이슈 제목 후보 | Phase | 성격 |
|-----:|----------------|-------|------|
| 1 | `[프론트] baseline freeze: 계약·렌더·확장·폰트 기준선 고정` | 0 | 계획 실행 전 관문 |
| 2 | `[프론트] web/fonts canonical 위치 이전 계획 및 경로 갱신` | A | asset ownership |
| 3 | `[프론트] /web legacy JS/HTML/CSS 삭제 가능성 검증` | A | legacy cleanup |
| 4 | `[프론트] InputHandler typed context 도입` | B1 | 복잡도/SOLID |
| 5 | `[프론트] keyboard/mouse event handler 고복잡도 함수 해체` | B1 | 복잡도 |
| 6 | `[프론트] PicturePropsDialog state/apply 분리` | B2 | 복잡도/SOLID |
| 7 | `[프론트] dialog collectMods 계열 공통 패턴 정리` | B2 | 복잡도 |
| 8 | `[프론트] WasmBridge JSON boundary typed adapter 분리` | B3 | 인터페이스 |
| 9 | `[프론트] core/types.ts 도메인별 type surface 분리` | B3 | 인터페이스 |
| 10 | `[프론트] diff-engine 책임 분리` | B4 | 복잡도 |
| 11 | `[확장] Chrome/Firefox/Safari 공통 보안 helper 확대` | C1 | 보안/공통화 |
| 12 | `[VS Code] webview WASM/font/CSP 계약 문서화 및 smoke` | C2 | 배포 계약 |
| 13 | `[npm] @rhwp/editor/@rhwp/core 문서 계약 갱신` | C3 | 공개 문서 |

처음부터 13개를 모두 만들지 않는다. 리뷰 후 Phase 0과 Phase A만 먼저 만들고, Phase B 이후는
Phase A 결과를 보고 분리한다.

## 9. Feature Freeze

전면 freeze는 과하다. 대신 phase별 좁은 freeze를 적용한다.

| Phase | freeze 범위 | 허용 |
|-------|-------------|------|
| Phase 0 | 프론트 계약 문서·측정 기준 | 긴급 버그 수정 허용 |
| Phase A | `web/fonts`, `rhwp-studio/public/fonts`, extension/VS Code font copy 경로 | font fallback 변경 금지 |
| Phase B1 | `rhwp-studio/src/engine/input-handler*` | unrelated UI/extension 변경 허용 |
| Phase B2 | dialog 대상 파일 | engine/extension 변경 허용 |
| Phase B3 | `wasm-bridge.ts`, `core/types.ts` | UI 동작 변경 금지 |
| Phase C | extension/VS Code/npm surface | studio engine 리팩터링과 병행 금지 |

## 10. 회귀 게이트

문서 계획 phase에서는 빌드/테스트를 요구하지 않는다. 실행 이슈에서는 변경 표면별로 아래 게이트를
선택한다.

| 변경 표면 | 필수 게이트 |
|-----------|-------------|
| `rhwp-studio` TypeScript | `npm run build`, 관련 `npm run test`, 관련 e2e |
| render/layout 영향 | render-diff 또는 대표 문서 visual smoke |
| font 경로 | 웹앱 build, Chrome/Firefox extension build, Safari 간접 build 영향 검토, VS Code webpack |
| 확장 CSP/public asset | unpacked load, viewer console CSP/404 확인 |
| VS Code webview | webpack build, HWP open, SVG export/debug overlay smoke |
| npm docs/API | README 예제와 package files 확인 |

## 11. 리뷰 요청 안건

#2023에 게시할 리뷰 요청은 다음으로 고정한다.

1. #1883과 동일하게 SOLID + 복잡도 2축을 프론트 리팩터링 거버넌스로 적용해도 되는가.
2. 예비 SOLID 점수 54/100과 S/I 우선순위 판단이 타당한가.
3. `/web` legacy 삭제보다 `web/fonts` canonical 이전을 먼저 하는 순서가 맞는가.
4. `web/fonts` 새 canonical 위치는 `rhwp-studio/public/fonts` 실체화와 공통 `assets/fonts`
   신설 중 어느 쪽이 적절한가.
5. React/Vue/Svelte 등 runtime UI framework 도입 금지를 이번 리팩터링 금지 목록에 명시해도
   되는가.
6. `@rhwp/editor` iframe/무의존 계약을 public contract로 고정해도 되는가.
7. 확장 CSP/`web_accessible_resources`/sender·URL 검증 guardrail을 실행 PR 체크리스트에
   넣어도 되는가.
8. 실행 하위 이슈는 Phase 0과 Phase A만 먼저 만들고, Phase B 이후는 재평가 후 분리하는
   방식이 적절한가.

## 12. 결론

프론트 웹 리팩터링은 진행할 가치가 있다. 다만 실행 순서는 다음처럼 제한해야 한다.

1. 계약과 baseline을 먼저 고정한다.
2. `/web`와 `web/fonts` ownership을 먼저 분리한다.
3. `rhwp-studio` 고복잡도 함수는 typed context와 state/apply 분리부터 시작한다.
4. 확장·VS Code·npm은 구조 미화보다 보안·배포 계약 유지가 우선이다.
5. 실행 이슈는 작게 만들고, 각 phase 완료 후 재측정·승인을 받는다.

본 문서는 실행 계획 초안이며, 코드 변경은 포함하지 않는다. 리뷰 승인 후 Phase 0 실행 이슈부터
별도로 등록한다.
