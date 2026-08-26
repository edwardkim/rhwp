# Stage 3 처리 결과 — #6118 서식 도구 모음 브라우저·테마 검증

- **이슈**: [#6118](https://github.com/edwardkim/rhwp/issues/6118)
- **기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **검증일**: 2026-08-26 KST
- **상태**: Stage 3 완료, #6138 통합 검증 완료·통합 PR 승인 대기

## 1. 확대 반응형 검증

Stage 2의 7개 viewport를 12개로 늘려 콘텐츠 경계와 일반 구간을 함께 검증했다.

| 구간 | viewport | 결과 |
| --- | --- | --- |
| 전체 압축 1행 | 1920, 1280, 1024, 976px | 1행, 36px, paragraph inline, root/style overflow 0 |
| 압축 2행 inline | 975, 883, 768, 460px | 2행, 83px, paragraph inline, root/style overflow 0 |
| 압축 2행 더보기 | 459, 412, 390, 375px | 2행, 83px, paragraph panel, root/style overflow 0 |

경계 ±1px은 976/975px과 460/459px에서 정확히 전환됐다. 375px에서도 field 행이 내부 폭을 넘지
않았고, 문단 명령은 더보기로 모두 도달 가능했다.

## 2. 테마·스킨 매트릭스

`default`, `flat`, `oldschool` 각각에 light/dark를 적용하고 976px 단일 행, 460px inline 2행,
375px 더보기 2행을 전수 검사했다. 총 18개 조합 모두 배경·경계·아이콘 대비와 panel 대비가 판정
기준을 통과했다.

| 스킨 | 단일 행 | 2행 | 아이콘 최소 대비 | 더보기 panel 최소 대비 |
| --- | ---: | ---: | ---: | ---: |
| default | 36px | 83px | 11.09 | 9.81 |
| flat | 36px | 83px | 11.05 | 9.81 |
| oldschool | 36px | 84px | 6.94 | 6.94 |

첫 실행에서 oldschool 단일 행만 상·하단 베벨을 모두 더해 37px이 되는 회귀를 발견했다. 전체 행의
상단 padding을 스킨 토큰으로 1px 줄여 두 베벨과 control 높이를 보존하면서 36px 계약을 회복했고,
정적 회귀 테스트를 추가했다.

## 3. 상호작용 검증

- 976px에서 글자 크기 입력, 글자 효과 dropdown, 형광펜 palette, 글자색 input을 실제로 조작했다.
- 459~375px에서 click·ArrowDown 열기, 첫 명령 focus, Escape 닫기와 trigger focus 복귀를 확인했다.
- 외부 pointer와 문단 명령 실행으로 panel이 닫히고, 명령 실행 뒤 trigger로 focus가 복귀했다.
- paragraph active 상태와 전체 disabled 상태가 더보기 trigger에 반영됐다.
- 실제 인앱 브라우저 375×812px에서도 더보기 click 뒤 `aria-expanded=true`, 첫 명령
  `btn-align-left` focus, panel viewport 내부 배치와 root 가로 overflow 0을 확인했다.

## 4. 대표 화면

| 모드 | 증적 |
| --- | --- |
| 976px 전체 압축 1행 | [stylebar-full-976.png](../report/assets/task_m100_6118/stylebar-full-976.png) |
| 460px 압축 2행 inline | [stylebar-inline-460.png](../report/assets/task_m100_6118/stylebar-inline-460.png) |
| 375px 압축 2행 더보기 | [stylebar-overflow-375.png](../report/assets/task_m100_6118/stylebar-overflow-375.png) |

Studio chrome의 DOM/CSS/접근성 변경이며 renderer·layout·typeset·paint 결과는 바꾸지 않는다. 시각 검증
거버넌스에 따라 PDF/SVG renderer sweep 대신 viewport·테마 browser E2E와 실제 UI smoke를 적용했다.

## 5. 검증 결과

| 검증 | 결과 |
| --- | --- |
| `npx tsc --noEmit` | 통과 |
| Stage 3 focused 정적 계약 | 27 passed, 0 failed |
| `npm test` | 1,141 passed, 0 failed, 1 skipped |
| `npm run build` | 통과 |
| responsive/theme E2E | 388 passed, 0 failed |
| `git diff --check` | 통과 |

E2E는 `http://127.0.0.1:7718/`, Puppeteer headless shell, DPR 1에서 수행했다. 저장소 전체 E2E manifest
검사는 이 변경과 무관한 기존 미등재 파일 세 개(`loading-busy-cursor`, `status-page-number`,
`toolbox-visibility`)만 보고한다. Rust source 변경은 0건이며 source 작업 트리에 review/CI 파생
`tests/generated/regression_suite_001.rs`~`032.rs`가 없는 source checkout에서는 `cargo fmt --all`을
실행하지 않았다. #6138 통합 Stage 3에서 파생 suite를 준비한 별도 review checkout의 Rust manifest와
`cargo fmt --all`·`cargo fmt --all -- --check`가 모두 통과했다.

## 6. Stage 3 종료 판정과 #6138 통합 원칙

- [x] 12개 viewport와 두 경계 ±1px을 검증했다.
- [x] 세 스킨의 light/dark 18개 조합을 검증했다.
- [x] field·dropdown·color·paragraph 더보기를 실제로 조작했다.
- [x] oldschool 37px 회귀를 36px로 수정하고 회귀 테스트를 추가했다.
- [x] 대표 화면과 재현 가능한 자동 검증 근거를 남겼다.

## 7. 사용자 시각 검토 후속 — 문단 정렬형 더보기

초기 `⋯ + ▾` 트리거는 일반 overflow와 dropdown 의미가 겹쳐 숨겨진 명령을 설명하지 못했다. 사용자
시각 검토를 반영해 기존 정렬 SVG mask를 재사용한 `현재 문단 정렬 아이콘 + ▾`로 교체했다. 활성 정렬이
바뀌면 trigger 아이콘과 `aria-label`·title이 함께 바뀌며, panel이 열리면 화살표와 버튼 표면도 열린
상태를 표시한다. 문단 명령 실행 뒤 다음 frame에 trigger focus를 확정해 편집기 focus 갱신과의 경쟁도
막았다. focused 28건, TypeScript, build와 전체 responsive/theme E2E 388건이 다시 통과했다.

#6118의 로컬 구현과 Stage 3에 이어 #6138도 같은 브랜치에서 별도 계획·구현·커밋으로 완료했다.
[통합 Stage 3](task_m100_6138_stage3.md)의 12개 viewport·18개 theme 매트릭스에서 두 영역을 다시
검증했다. 최종 PR 하나에서 `#style-bar`와 `#icon-toolbar`의 책임을 각각 설명하고 두 이슈를 함께
연결한다.
