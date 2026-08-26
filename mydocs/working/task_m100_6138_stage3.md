# Stage 3 처리 결과 — #6138 기본 도구 상자 통합·테마 검증

- **이슈**: [#6138](https://github.com/edwardkim/rhwp/issues/6138)
- **기준**: `upstream/devel@1011a8947`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **검증일**: 2026-08-26 KST
- **상태**: Stage 3 완료, #6118 통합 PR 승인 대기

## 1. 반응형 통합 검증

#6118의 서식 바 경계를 유지한 채 #6138 기본 도구 상자를 같은 12개 viewport에서 다시 검증했다.

| viewport | 기본 도구 상자 | 이동 버튼 | 서식 바 |
| ---: | --- | --- | --- |
| 1920, 1280px | 56px·1행·label 표시 | 숨김 | 36px·1행 |
| 1024, 976px | 56px·1행·label 표시 | 표시 | 36px·1행 |
| 975, 883, 768, 460px | 56px·1행·label 표시 | 표시 | 83px·2행 inline |
| 459, 412, 390, 375px | 56px·1행·label 표시 | 표시 | 83px·2행 더보기 |

모든 구간에서 document root와 두 toolbar 외부의 가로 overflow는 0이다. 기본 도구 상자의 1219px track만
내부 viewport에서 수평 이동하며 viewport 너비나 #6118 모드에 따라 button 폭·label·명령 순서가 바뀌지
않는다.

## 2. 이동·상태·접근성 검증

- 다음/이전 버튼은 인접한 가시 `.tb-group` 경계로 이동하고 시작·끝에서 native disabled가 된다.
- ArrowLeft/ArrowRight·Home·End, trackpad 수평 wheel, touch pan과 offscreen command focus를 지원한다.
- 머리말/꼬리말 mode 전환 때 시작 위치와 overflow를 다시 계산한다.
- #6115의 기본 도구 상자 숨김은 nav·viewport를 포함한 외부 `#icon-toolbar` 전체에 적용된다.
- 찾기 split menu는 수평 viewport에 잘리지 않고 화면 안에 고정 배치되며 toolbar 이동 시 닫힌다.
- nav의 `aria-label`·title과 viewport의 accessible name을 제공하고 숨긴 nav는 Tab 순서에서 제외한다.

## 3. 테마·스킨 매트릭스

`default`, `flat`, `oldschool` 각각에 light/dark를 적용하고 976px, 460px, 375px 세 경계를 전수
검사했다. 총 18개 조합에서 기본 도구 상자는 56px·1행, 서식 바는 예정된 1·2행을 유지했다. nav의
배경·경계·disabled 표현과 toolbar 배경 대비는 모두 자동 판정을 통과했으며 최소 대비는 4.10이다.

## 4. 대표 화면

| 모드 | 증적 |
| --- | --- |
| 1280px 전체 표시·nav 숨김 | [toolbar-wide-1280.png](../report/assets/task_m100_6138/toolbar-wide-1280.png) |
| 1024px 한 줄 group scroll | [toolbar-scroll-1024.png](../report/assets/task_m100_6138/toolbar-scroll-1024.png) |
| 375px 한 줄 group scroll | [toolbar-scroll-375.png](../report/assets/task_m100_6138/toolbar-scroll-375.png) |

Studio chrome의 DOM/CSS/접근성 변경이며 renderer·layout·typeset·paint 결과는 바꾸지 않는다. 시각 검증
거버넌스에 따라 PDF/SVG sweep 대신 viewport·테마 browser E2E와 대표 화면 육안 검토를 적용했다.

## 5. 검증 결과

| 검증 | 결과 |
| --- | --- |
| focused source/controller/theme 계약 | 33 passed, 0 failed |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,146 passed, 0 failed, 1 skipped |
| `npm run build` | 통과(230 modules transformed) |
| responsive/theme/#6118 통합 E2E | 609 passed, 0 failed |
| Markdown 상대 링크·`git diff --check` | 603문서 이상 없음·통과 |
| review checkout Rust suite manifest | 942 sources·32 harnesses·9 exceptions, 통과 |
| `cargo fmt --all`·`cargo fmt --all -- --check` | 통과 |

E2E는 `http://127.0.0.1:7718/`, 설치된 Chrome, DPR 1에서 수행했다. E2E manifest 전체 검사는 최신
기준선의 기존 미등재 파일 세 개(`loading-busy-cursor`, `status-page-number`, `toolbox-visibility`)만
보고하며 이번 변경의 `responsive.test.mjs` 항목은 등록되어 있다.

## 6. Stage 3 종료 판정

- [x] 12개 viewport와 #6118의 두 콘텐츠 경계를 함께 검증했다.
- [x] 세 스킨의 light/dark 18개 조합을 검증했다.
- [x] group 버튼·keyboard·wheel·focus·mode·visibility·split menu를 실제로 조작했다.
- [x] 1280, 1024, 375px 대표 화면을 추적 가능한 증적으로 남겼다.
- [x] Studio test/build, TypeScript, E2E와 문서·diff 게이트를 통과했다.
- [x] 파생 suite 전용 review checkout에서 Rust manifest와 format 게이트를 통과했다.

#6138의 로컬 구현과 Stage 3는 완료했다. #6118과 #6138은 이슈·계획·stage·커밋·검증을 분리해
추적성을 유지하고, 원격에는 두 작업을 함께 설명하는 PR 한 건으로 제출한다. push와 PR 생성은 사용자
승인을 받은 뒤 수행한다.
