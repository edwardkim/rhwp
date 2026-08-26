# Stage 2 처리 결과 — #6118 서식 도구 모음 1·2행과 동적 더보기

- **이슈**: [#6118](https://github.com/edwardkim/rhwp/issues/6118)
- **기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **구현일**: 2026-08-26 KST
- **상태**: Stage 2 완료, Stage 3 테마 전수 검증 승인 대기

## 1. 구현 결과

Stage 1에서 고정한 콘텐츠 경계 세 개를 그대로 제품 계약으로 반영했다.

| viewport | 배치 | 높이 | paragraph 명령 |
| ---: | --- | ---: | --- |
| 976px 이상 | field + command 단일 행 | 36px | inline |
| 975~460px | field 1행 + command 1행 | 83px | inline |
| 459~375px | field 1행 + command 1행 | 83px | 더보기 panel |

`#style-bar` 외곽은 전체 viewport의 배경과 경계를 계속 소유한다. 내부 `.sb-field-grid`는 최대 500px,
`.sb-command-track`은 `max-content`와 `max-width:100%`로 제한해 왼쪽 정렬하고 빈 공간까지 늘어나지
않는다. 375px에서는 필드 열 합이 내부 363px에 맞으며 root와 style bar의 가로 overflow가 모두 0이다.

## 2. 단일 DOM authority와 더보기 동작

- 기존 paragraph button 여섯 개의 ID와 순서를 한 번만 유지했다.
- 460px 이상에서는 overflow host/panel을 `display:contents`로 풀어 기존 명령을 inline에 둔다.
- 459px 이하에서는 같은 paragraph DOM을 panel로 표시하며 command clone과 runtime reparent는 없다.
- trigger는 `aria-controls`, `aria-expanded`, 현재 정렬을 포함한 label·아이콘과 disabled 상태를 동기화한다.
- trigger 표면은 panel이 열린 동안에만 활성 상태로 표시한다.
- click·ArrowDown으로 열고 첫 사용 가능한 명령으로 focus를 옮긴다.
- Escape·외부 pointer·명령 실행으로 닫으며 Escape와 명령 실행 뒤 trigger로 focus를 돌린다.
- viewport가 460px 이상으로 넓어지면 열린 상태를 해제하고 paragraph 명령을 즉시 inline으로 복귀시킨다.

기존 `btn-align-*`의 mousedown/click command wiring은 수정하지 않았다. 글자 효과, 글자색, 형광펜의
dropdown과 alignment mask icon도 기존 자산을 재사용했다.

## 3. 변경 파일

| 구역 | 파일 | 결과 |
| --- | --- | --- |
| DOM | `rhwp-studio/index.html` | command track과 paragraph overflow host/panel 추가 |
| layout | `src/styles/style-bar.css`, `responsive.css` | 976/460 경계의 최대 2행 구조와 375px field 계약 |
| runtime | `src/ui/style-toolbar-overflow.ts`, `src/main.ts` | panel 상태·focus·active/disabled 동기화와 초기화 |
| static test | `tests/style-toolbar-grouped-ribbon.test.ts`, `responsive-toolbar-layout.test.ts`, `style-toolbar-overflow.test.ts` | 단일 DOM, 경계, 행 수, controller 계약 |
| browser E2E | `e2e/responsive.test.mjs`, `e2e/MANIFEST.md` | 7개 경계의 행·높이·overflow·더보기 실행 검증 |

`#icon-toolbar`, 도구 상자 visibility/단축키, settings, command 구현, WASM과 renderer는 변경하지 않았다.

## 4. 검증 결과

| 검증 | 결과 |
| --- | --- |
| `npx tsc --noEmit` | 통과 |
| Stage 2 정적 계약 | 16 passed, 0 failed |
| `npm test` | 1,139 passed, 0 failed, 1 skipped |
| `npm run build` | 통과 |
| headless responsive E2E | 97 passed, 0 failed |
| 실제 렌더 경계 | 976px 1행·36px, 975/460px 2행, 459/375px 2행+더보기 |
| Markdown 상대 링크 | 603문서, 이상 없음 |
| `git diff --check` | 통과 |

E2E는 `http://127.0.0.1:7718/`, Puppeteer headless shell, DPR 1에서 1280, 976, 975, 768, 460,
459, 375px을 실행했다. 375px 더보기의 열기, 첫 명령 focus, 명령 실행 뒤 닫힘과 trigger focus 복귀까지
통과했다. 저장소 E2E manifest 전체 검사는 이번 변경과 무관한 기존 미등재 파일 세 개
(`loading-busy-cursor`, `status-page-number`, `toolbox-visibility`)만 보고했다.

`cargo fmt --all`과 `cargo fmt --all -- --check`는 Rust diff가 아니라 source 작업 트리에 정책상 만들지
않은 review/CI 파생 파일 `tests/generated/regression_suite_001.rs`~`032.rs`가 없어 시작 전에 종료했다.
Rust source 변경은 0건이다. PR/push 전 review 준비 checkout에서 파생 suite를 준비한 뒤 두 필수 명령을
다시 통과시켜야 한다.

## 5. Stage 2 종료 판정

- [x] 976px 이상에서 전체 1행·36px 이하이다.
- [x] 975~375px에서 최대 두 행만 사용한다.
- [x] 460/459px에서 paragraph inline↔더보기 전환이 일어난다.
- [x] 375px에서 page/style bar 가로 overflow가 없다.
- [x] 기존 명령 ID·순서·listener와 접근성 이름을 유지한다.
- [x] 더보기 focus·Escape·외부 click·현재 정렬 표시/disabled 계약을 구현했다.
- [x] #6115·#6138의 `#icon-toolbar` 범위를 변경하지 않았다.

Stage 2는 완료했다. 다음 단계는 사용자 승인 뒤 1920~375px 확대 검증과 default/flat/oldschool ×
light/dark 전수 시각·상호작용 검증, 최종 보고서와 PR 제출 근거를 준비하는 Stage 3다.
