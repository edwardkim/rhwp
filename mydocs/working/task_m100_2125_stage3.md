# Task M100 #2125 Stage 3 완료 보고 — font 운영 문서 정합

- 이슈: #2125
- 브랜치: `task2125-assets-fonts-canonical`
- source 기준: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- Stage 2 commit: `acedc288`
- 완료일: 2026-07-13
- 상태: Stage 3 완료, Stage 4 승인 대기

## 1. 수행 내용

1. canonical `assets/fonts/FONTS.md`와 root `THIRD_PARTY_LICENSES.md`의 source ownership을 갱신했다.
2. Noto Sans KR 3종과 누락됐던 Source Han Old Hangul·Latin Modern Math inventory를 현행화했다.
3. Chrome/Firefox/Safari build manual의 source와 36개 전체 copy 계약을 갱신했다.
4. Firefox AMO source archive 입력을 `assets/fonts`로 변경했다.
5. `@rhwp/editor` package와 rhwp-studio 배포본의 font ownership을 분리해 설명했다.
6. fallback/equation/subset 문서의 현재 canonical source와 runtime 경로를 구분했다.
7. 과거 task snapshot과 당시 실행 경로는 변경하지 않았다.

## 2. source와 runtime 계약

| 구분 | 문서화한 계약 |
|------|---------------|
| canonical source | `assets/fonts/` |
| Studio runtime | 배포본의 `fonts/...` URL |
| legacy `/web` runtime | 기존 `fonts/...` URL, compatibility link 유지 |
| browser extension dist | `assets/fonts`의 WOFF2 36개 전체를 `dist/fonts`로 복사 |
| VS Code | 기존 11개 subset과 `dist/media/fonts` 계약 유지 |
| `@rhwp/editor` | package 자체 font/UI runtime dependency 없음, iframe target이 font 제공 |

## 3. inventory correction

| 항목 | 이전 문서 | 현재 사실 |
|------|-----------|-----------|
| canonical WOFF2 | 경로·총량 미명시 | 36개, 22,651,296 bytes |
| Noto Sans KR | root license 표 2종 | Regular/Bold/ExtraLight 3종 |
| Source Han Old Hangul | canonical inventory 누락 | WOFF2와 OFL 파일 명시 |
| Latin Modern Math | canonical inventory 누락·과거 filename | `LatinModernMath-Regular.woff2` 명시 |
| extension copy | 14개 | canonical 36개 전체 |

폰트 binary, filename, fallback chain과 runtime URL은 변경하지 않았다.

## 4. 역사 문서 보존

`mydocs/tech/font_fallback_strategy.md`의 2026-04-07 로드맵·권장안에 남은 `web/fonts` 2건은 당시
구현 경로를 설명하는 기록이다. 문서 상단에 현재 canonical source와 역사 표현 보존 이유를 명시했다.

`mydocs/manual/chrome_edge_extension_build_deploy.md`의 2026-04-23 크기 표도 당시 WOFF2 14개 build의
실측값이므로 숫자를 추정해 덮어쓰지 않고 역사적 snapshot임을 명시했다. 현재 36개 build 크기는 Stage 4
fresh build에서 측정한다.

다음 경로는 diff가 없음을 확인했다.

- `mydocs/metrics/frontend/2026-07-11/**`
- `mydocs/tech/investigations/issue-2124/task_m100_2124_font_inventory.md`
- `mydocs/report/task_m100_2190_report.md`
- archive와 과거 feedback 문서

## 5. 검증

| 검증 | 결과 |
|------|------|
| active manual/package/root license의 `web/fonts` source reference | 0 |
| fallback 문서 잔여 `web/fonts` 판정 | 역사 기록 2, compatibility/보존 설명 2 |
| canonical 36개 basename의 `FONTS.md` 표현 | PASS, 누락 0 |
| AMO source archive 입력 경로 | PASS, 누락 0 |
| `@rhwp/editor` dependencies/peerDependencies | 각각 0 |
| history-only 경로 diff | 0 |
| source/runtime/manifest 코드 diff | 0 |
| `git diff --check` | PASS |

## 6. Stage 4 진입 판단

current operational documentation과 구현된 ownership 계약이 일치한다. 다음 단계에서는 Docker로 fresh
WASM을 생성하고 Studio, Chrome, Firefox, Safari, VS Code build와 E2E, font hash contract, frontend
metrics를 실행한다.

작업지시자 승인 전에는 Stage 4 dependency install, WASM build, package build/E2E와 metrics 수집을
시작하지 않는다.
