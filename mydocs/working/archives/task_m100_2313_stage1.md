# Task M100 #2313 Stage 1 완료 보고 — 제거 전 inventory와 동일-base 기준선

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- 계획 문서 commit: `4f8dedacf53d7a254666e49b0c88cd0c89eb423c`
- 완료일: 2026-07-17
- 상태: Stage 1 local 완료, Stage 2 승인 대기

## 1. Stage 1 판정

최신 `upstream/devel` drift는 없고 계획 문서 commit 뒤 작업 트리는 clean이었다. tracked `web/` 18개
entry와 repository 참조를 다시 분류한 결과, legacy 앱을 production/build/package source로 직접 소비하는
경로는 0이다.

canonical font와 Studio public link도 #2125 완료 상태와 일치한다. 따라서 계획대로 legacy tree를
Studio에 복제하지 않고 제거할 수 있는 선행 조건을 충족했다. 이 단계에서는 `web/`, CI, metrics, test,
manual과 product source를 수정하지 않았다.

## 2. 기준 브랜치와 clean provenance

| 항목 | 값 |
|------|----|
| `upstream/devel` | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| branch plan HEAD | `4f8dedacf53d7a254666e49b0c88cd0c89eb423c` |
| merge-base | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| upstream drift | 0 commit |
| plan HEAD 상태 | upstream 대비 ahead 1, working tree clean |
| metrics `git.clean` | `true` |
| metrics `measuredSourceClean` | `true` |
| dirty path | 0 |

branch의 한 commit은 orders·수행 계획·구현 계획만 포함한다. measured frontend source는 base와 동일하다.

## 3. tracked `/web` inventory

### 3.1 전체 entry

| 분류 | 개수 | 대상 | 규모 |
|------|-----:|------|------:|
| metrics 대상 legacy 앱 | 10 | JS 6, CSS 2, HTML 2 | 6,592 lines, 5,795 code lines |
| 개발·테스트 | 4 | Python server, clipboard HTML, cert/key | text 197 lines, cert/key 2,859 bytes |
| tracked generated glue | 3 | `rhwp.js`, `rhwp.d.ts`, `rhwp_bg.wasm.d.ts` | 8,892 lines |
| compatibility link | 1 | `web/fonts -> ../assets/fonts` | symlink |
| 합계 | 18 | tracked `web/` 전체 | text 15,681 lines + cert/key + symlink |

metrics 대상 10개는 다음과 같다.

- `app.js`, `char_shape_dialog.js`, `editor.js`, `font_substitution.js`
- `format_toolbar.js`, `text_selection.js`
- `editor.css`, `style.css`
- `editor.html`, `index.html`

`clipboard_test.html`, Python server, generated glue, cert와 font link는 현재 metrics exclude 규칙에 따라
complexity 모집단에 포함되지 않는다.

### 3.2 symlink와 canonical font

| 항목 | 값 | 판정 |
|------|----|------|
| `web/fonts` | `../assets/fonts` | 제거 대상 compatibility link |
| `rhwp-studio/public/fonts` | `../../assets/fonts` | 유지할 current link |
| canonical WOFF2 | 36개 | 일치 |
| canonical bytes | 22,651,296 | 일치 |
| `NotoSansKR-Regular.woff2` SHA-256 | `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a` | 일치 |

`assets/fonts`와 Studio link에는 Stage 2 diff가 생기면 안 된다.

## 4. repository dependency 분류

### 4.1 product/build/package

production/build/package path에서 legacy 파일을 직접 읽거나 import하는 소비자는 0이다.

| 분류 | 남은 참조 | 처리 |
|------|-----------|------|
| Studio source | `font-loader.ts`의 `web/editor.html` 포팅 출처 주석 | provenance로 유지 |
| Studio source | `font-substitution.ts`의 `web/font_substitution.js` 포팅 출처 주석 | provenance로 유지 |
| Chrome/Firefox/Safari | legacy source/path 직접 참조 0 | 변경 없음 |
| VS Code | legacy source/path 직접 참조 0 | 변경 없음 |
| npm editor | legacy source/path 직접 참조 0 | 변경 없음 |
| root build/Docker | legacy source/path 직접 참조 0 | 변경 없음 |

두 Studio 주석은 current runtime dependency나 지원 URL이 아니라 TypeScript port의 출처를 설명한다.

### 4.2 current tooling

| 파일 | 결합 | Stage 2 처리 |
|------|------|--------------|
| `.github/workflows/ci.yml` | frontend detector의 `web/` prefix | prefix 제거 |
| `scripts/frontend-metrics.mjs` | legacy include group, 11개 전용 판별·exclude·link 지점 | 계획된 legacy 분기만 제거 |
| `scripts/frontend-font-assets.test.mjs` | `web/fonts` compatibility assertion | legacy entry만 제거 |

CI나 test 결합은 legacy 앱의 runtime 소비가 아니라 제거 전 구조를 검증·감지하기 위한 관리 코드다.

### 4.3 current 문서

| 파일 | 현재 상태 | Stage 3 처리 |
|------|-----------|--------------|
| 한/영 `local_web_server.md` | legacy Python HTTPS server와 `/web/*.html` 실행을 안내 | legacy 절 삭제 |
| `font_fallback_strategy.md` | current note에 legacy font link가 남음 | current 두 지점만 보정 |
| #2023 contract guardrail | pre-Phase-A ownership과 open decision이 남음 | post-removal current 계약으로 갱신 |

font fallback 문서의 2026-04-07 historical 절, #2023 diagnosis와 plan, #2124 snapshot, #2125 plan/stage/report,
과거 archives와 changelog는 당시 사실이므로 수정하지 않는다.

## 5. pre-removal metrics

### 5.1 실행 환경과 artifact

| 항목 | 값 |
|------|----|
| schema | 2 |
| Node.js | `v24.15.0` |
| ESLint | `10.6.0` |
| sonarjs | `4.1.0` |
| TypeScript parser | `8.63.0` |
| TypeScript | `6.0.3` |
| platform | `darwin arm64`, release `25.5.0` |
| output | `output/frontend-metrics/task2313/pre/metrics.json` |
| summary | `output/frontend-metrics/task2313/pre/summary.md` |

`npm --prefix scripts/frontend-metrics ci`로 lockfile 기준 93 package를 설치한 뒤 실행했다. output은 ignored
local evidence이며 commit하지 않는다.

| artifact | SHA-256 |
|----------|---------|
| `scripts/frontend-metrics.mjs` | `4008301769eca77bfc25233556fa8fbe3bb9b2560f3083b49db030af9d5354d3` |
| metrics package lock | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| pre-removal metrics JSON | `f81a720f455b82d57809d1fcfc7de1272aa78c27f68d8cfbbfd46a68095e5aa5` |

### 5.2 current 전체 값

| 지표 | 값 |
|------|---:|
| included files | 224 |
| reported CC functions | 2,516 |
| Total CC | 13,118 |
| global Top 20 합 | 2,700 |
| CC>25 개수 / 합 | 73 / 4,479 |
| CC>100 개수 / 합 | 7 / 1,839 |
| Max CC | 453 |

### 5.3 legacy group

| 지표 | 값 |
|------|---:|
| files / lines / code lines | 10 / 6,592 / 5,795 |
| AST functions | 251 |
| reported CC functions | 149 |
| Total CC | 828 |
| group Top 20 합 | 442 |
| CC>25 개수 / 합 | 4 / 207 |
| CC>100 | 0 |
| Max CC | 86 |

group Top 20 합 442는 global Top 20 삭제 delta가 아니다. legacy entry 제거 뒤 전체 순위를 다시 계산해야
한다.

### 5.4 legacy 제외 예상값

pre-removal `cognitiveComplexityEntries`에서 `web/` entry만 제외해 계산했다.

| 지표 | pre | projected post | expected direct delta |
|------|----:|---------------:|----------------------:|
| included files | 224 | 214 | -10 |
| reported CC functions | 2,516 | 2,367 | -149 |
| Total CC | 13,118 | 12,290 | -828 |
| global Top 20 합 | 2,700 | 2,662 | -38 |
| CC>25 개수 / 합 | 73 / 4,479 | 69 / 4,272 | -4 / -207 |
| CC>100 개수 / 합 | 7 / 1,839 | 7 / 1,839 | 0 / 0 |
| Max CC | 453 | 453 | 0 |

Stage 2·5 actual post 값은 이 표와 비교한다. non-legacy stable function diff는 0건이어야 한다.

### 5.5 #2124 공식 snapshot 대비 누적값

| 지표 | 누적 delta |
|------|-----------:|
| reported CC functions | +234 |
| Total CC | +1,313 |
| global Top 20 합 | +119 |
| CC>25 개수 / 합 | +11 / +547 |
| CC>100 개수 / 합 | +1 / +107 |
| Max CC | 0 |

이는 `3077f96d` 이후 upstream 전체 변화이며 #2313 직접 delta가 아니다. 공식 snapshot은 변경하지 않는다.

## 6. Stage 1 gate 결과

| 관문 | 결과 |
|------|------|
| latest upstream 확인 | PASS, drift 0 |
| 계획 문서 clean commit | PASS, `4f8dedac` |
| tracked inventory | PASS, 18 entries |
| product/build/package 직접 소비자 | PASS, 0 |
| current tooling/doc 결합 분류 | PASS, 미분류 0 |
| canonical font inventory/hash | PASS |
| pre-removal metrics clean provenance | PASS |
| expected structural deletion delta | PASS, 계획값과 일치 |
| source/tree/tooling/manual 변경 | 0 |

## 7. 다음 단계 경계

작업지시자 승인 후 Stage 2에서만 다음을 수행한다.

1. tracked `web/` 전체 삭제
2. CI frontend detector의 `web/` prefix 제거
3. metrics legacy include/exclude/helper/link metadata 제거
4. font contract의 legacy link assertion 제거
5. syntax, actionlint와 pre/post metrics compare

Stage 2 승인 전에는 삭제 또는 위 source/tooling 수정을 수행하지 않는다. current 한/영 manual과 guardrail은
Stage 3 승인 전까지 변경하지 않는다.
