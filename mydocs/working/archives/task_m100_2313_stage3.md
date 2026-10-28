# Task M100 #2313 Stage 3 완료 보고 — current 문서와 guardrail 현행화

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- Stage 2 commit: `50f9485c`
- 완료일: 2026-07-17
- 상태: Stage 3 local 완료, Stage 4 승인 대기

## 1. Stage 3 판정

current 한/영 local server manual을 Studio-only workflow로 동기화하고 font ownership 및 프론트
guardrail을 post-Phase-A/post-removal 상태로 현행화했다. current manual은 더 이상 Python HTTPS server,
certificate, `/web/*.html` 또는 legacy troubleshooting을 지원 경로로 안내하지 않는다.

current source reference scan에는 Studio 포팅 출처 주석 2개만 남았다. font 전략의 `web/fonts` 세 지점은
2026-04-07 historical 절임을 상단 note에서 명시하고 있으며, guardrail의 `/web` 표현은 제거 사실과 복원
금지를 설명한다. #2124 공식 snapshot과 과거 plan/report/working/changelog에는 diff가 없다.

## 2. upstream과 변경 범위

| 항목 | 결과 |
|------|------|
| Stage 3 시작 `upstream/devel` | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| merge-base | 동일 commit |
| branch 상태 | upstream 대비 ahead 3, behind 0 |
| upstream drift | 0 |
| current 문서 변경 | 4개 |
| product/tooling 변경 | 0 |
| historical evidence 변경 | 0 |

## 3. 한/영 local server manual

다음 두 파일을 같은 정보 구조로 정리했다.

- `mydocs/manual/local_web_server.md`
- `mydocs/eng/manual/local_web_server.md`

각 문서는 다음 8개 heading을 1:1로 유지한다.

1. title
2. supported Studio Vite server
3. overview
4. prerequisites
5. execution steps
6. WASM build
7. Vite server start와 one-shot command
8. port

유지한 current 계약:

- Node.js v24+, npm v11+
- repository Docker WASM build
- output `pkg/rhwp_bg.wasm`, `pkg/rhwp.js`, `pkg/rhwp.d.ts`
- `rhwp-studio`의 Vite server와 `npm run dev`
- port 7700

제거한 legacy 안내:

- Python HTTPS server와 custom port
- self-signed certificate 생성과 warning 우회
- generated WASM을 `web/`에 복사하는 절차
- legacy editor/viewer/clipboard test URL
- legacy 기능 테스트와 troubleshooting

## 4. font fallback current note

`mydocs/tech/font_fallback_strategy.md`의 current ownership 두 지점만 보정했다.

| 항목 | 변경 |
|------|------|
| 현행화 날짜 | 2026-07-13 → 2026-07-17 |
| current runtime | Studio `fonts/...`만 설명 |
| ownership | `assets/fonts`, 36개, 22,651,296 bytes 유지 |
| Studio link | `rhwp-studio/public/fonts -> ../../assets/fonts` 유지 |
| legacy compatibility link | current 설명에서 제거 |

2026-04-07 조사 당시 `web/fonts` 권장안과 inventory는 historical evidence로 그대로 두었다. font family,
fallback, substitution, license 판단과 roadmap에는 의미 변화가 없다.

## 5. #2023 contract guardrail

`mydocs/tech/task_m100_2023_frontend_contract_guardrails.md`는 dated metrics snapshot이 아니라 후속 실행 이슈가
참조하는 current guardrail이므로 다음을 현행화했다.

| 영역 | post-removal 상태 |
|------|-------------------|
| 보호 surface | Studio, `assets/fonts`, browser extensions, VS Code, npm contracts |
| `@rhwp/editor` files | `index.js`, `index.d.ts`, `transport.js` |
| Studio font | `../../assets/fonts` canonical link |
| legacy `/web` | #2125 ownership 분리 뒤 #2313 제거, current tree 복원·복제 금지 |
| font license authority | `assets/fonts/FONTS.md`, `THIRD_PARTY_LICENSES.md` |
| VS Code source | `assets/fonts`의 승인된 11개 subset |
| runtime UI framework | 후보가 아니라 v1.0까지 확정 금지 |
| Phase B 입력 | legacy group 제외 최신 metrics |

Chrome/Firefox/Safari security, VS Code CSP, npm public API와 Studio runtime asset guardrail은 유지했다.
canonical 위치와 legacy 제거를 open decision 목록에서 제거하고, 남은 정책 변화는 별도 RFC/실행 이슈
대상으로 유지했다.

## 6. reference allowlist

current source·tooling·manual·policy를 legacy file path pattern으로 검색한 결과는 다음과 같다.

| 파일 | 남은 표현 | 분류 |
|------|-----------|------|
| `rhwp-studio/src/core/font-loader.ts` | `web/editor.html` | TypeScript 포팅 출처 provenance |
| `rhwp-studio/src/core/font-substitution.ts` | `web/font_substitution.js` | TypeScript 포팅 출처 provenance |
| `font_fallback_strategy.md` 상단 | historical `web/fonts` 보존 안내 | historical classifier |
| 같은 문서 2026-04-07 절 2곳 | 당시 `web/fonts` 권장안 | historical evidence |

current manual의 legacy 실행 path는 0이다. #2023 guardrail은 `/web` 제거 사실과 current tree 복원 금지를
설명하지만 실행 가능한 path나 지원 계약으로 안내하지 않는다.

## 7. historical evidence 보호

다음 경로의 working-tree diff가 비어 있음을 확인했다.

- `mydocs/metrics/frontend/2026-07-11/**`
- #2124 baseline manifest, metrics scope와 public contract snapshot
- #2023 v2 plan
- #2125 final report
- `CHANGELOG.md`, `CHANGELOG_EN.md`

과거 plans/reports/working/archives는 전역 치환하지 않았다. Git history와 dated task evidence가 당시
구조를 보존한다.

## 8. 검증 결과

| 검증 | 결과 |
|------|------|
| 한/영 manual heading 구조 | PASS, 8개 1:1 |
| current manual legacy 실행 명령 | PASS, 0 |
| current source direct consumer | PASS, 0 |
| provenance allowlist | PASS, 2개 |
| font current/historical 표현 분리 | PASS |
| guardrail canonical/legacy/framework 결정 | PASS |
| historical evidence diff | PASS, 0 |
| `git diff --check` | PASS |
| trailing whitespace | PASS, 0 |

문서-only Stage이므로 metrics와 package build를 다시 실행하지 않았다. Stage 2 metrics output은 source
변경이 없으므로 유지되며, fresh WASM/package/browser 검증은 Stage 4에서 수행한다.

## 9. 다음 단계 경계

작업지시자 승인 후 Stage 4에서 다음을 수행한다.

1. Docker fresh WASM build
2. CI parity binding/editor/service worker/Studio/package gate
3. Studio text-flow, embed와 CanvasKit font browser smoke
4. Chrome/Firefox build, extension dist, VS Code compile과 full font contract
5. Safari source/manifest/Chrome dist inheritance static gate

Stage 4 승인 전에는 product source를 수정하거나 build/E2E를 실행하지 않는다. 필수 gate가 실패하면 원인을
분석하되 별도 승인 없이 기능 수정으로 범위를 넓히지 않는다.
