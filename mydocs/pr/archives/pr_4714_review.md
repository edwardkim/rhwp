---
kind: review
status: active
canonical: mydocs/pr/archives/pr_4714_review.md
last_verified: 2026-08-13
---

# PR #4714 검토 기록 - 옵트인 플랫 스킨과 테마 제작 가이드

## PR 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4714](https://github.com/edwardkim/rhwp/pull/4714) |
| 작성자 | @keepYaoung (기존 기여자, #4699 merge 이력) |
| 대상 브랜치 | `devel` |
| code candidate | `31d474e1db91e27de9177bb7b1564b3ac48e2556` |
| 작성 시점 참고 상태 | `mergeable=CLEAN`, `MERGEABLE` |
| 변경 규모 | 12개 파일, +379/-22, contributor 3 commit + 메인터너 1 commit |
| 관련 논의 | [Discussions #4706](https://github.com/edwardkim/rhwp/discussions/4706) |

PR 본문에는 issue closing keyword가 없으므로, merge 뒤 수동으로 close할 GitHub issue는 없다.

## 변경 범위와 판단

- `theme.skin` (`default`/`flat`)을 사용자 설정에 저장하고, 초기 `theme-init.js`와 런타임
  `theme.ts`가 같은 `data-theme-skin` 계약으로 적용한다.
- 보기 메뉴에 스킨 라디오 항목과 접근성 체크 상태 동기화를 추가한다.
- 플랫 CSS는 스킨 속성 아래로만 범위를 제한하고, 색 팔레트는 다크 모드를 덮지 않도록 라이트 가드를 둔다.
- 토큰 계층과 새 스킨 등록 절차를 manual에 추가한다.

이번 변경은 Studio UI chrome CSS와 설정 배선만 바꾼다. 문서 Canvas 렌더러, pagination, HWP/HWPX
serializer는 바꾸지 않으므로 PDF visual sweep은 필요하지 않다. 대신 실제 Studio 브라우저 동작을 확인했다.

## 메인터너 보정

contributor 변경에는 `package.json` 변경 없이 `package-lock.json`의 Linux native binding `libc`
조건 30줄이 제거돼 있었다. 이 메타데이터가 없으면 glibc/musl 선택 범위가 불필요하게 넓어진다.

- `31d474e1d` `build(studio): Linux 네이티브 바인딩 libc 조건 복원`
- `@rolldown/binding-*`와 `lightningcss-*`의 `glibc`/`musl` 조건을 최신 `devel` 값으로 정확히 복원했다.
- 보정은 lockfile 외 기능 코드·테스트·문서를 바꾸지 않았고 contributor commit을 rewrite하지 않았다.

## 검증

### 로컬

- 최신 `upstream/devel` 병합 시뮬레이션은 충돌 없이 통과했고 `git diff --check`도 통과했다.
- current-base merge simulation에서 `npm test` 890건을 실행해 모두 통과했다.
- current-base merge simulation에서 `npm run build` (`tsc && vite build`)를 통과했다.
- 보정 뒤 `npm ci --ignore-scripts --dry-run`을 통과했고, lockfile은 `devel`의 native binding
  `libc` 메타데이터와 일치한다.
- headless Chromium에서 저장된 `flat`/`light` 설정의 초기 dataset 적용, 메뉴 radio 체크 상태와
  localStorage 왕복을 확인했다. 작업지시자는 foreground Vite에서 실제 메뉴·스킨 표시를 직접 확인했다.

### GitHub Actions

code candidate `31d474e1d`의 최신 PR head에서 다음을 확인했다.

- CI run `31691756020`: Frontend package gates 성공(6분 6초), Build & Test aggregate 성공.
- CodeQL run `31691755844`: JavaScript/TypeScript 분석 성공(2분 43초), Python/Rust 선택 경로 성공.
- Render Diff run `31691755778`: Canvas visual diff 성공(5분 54초).

Rust source 변경이 없고 정확한 code candidate의 GitHub CI가 위 범위를 완료했으므로, 전체 Cargo 회귀는
중복 실행하지 않았다.

## 위험과 후속

- 플랫 스킨은 opt-in이며 기본 스킨은 `data-theme-skin` 속성을 제거해 기존 CSS가 그대로 적용된다.
- 다크 전용 스킨 팔레트는 아직 제공하지 않는다. 새 스킨은
  [테마 토큰과 스킨 제작 가이드](../../manual/rhwp_studio_theming.md)의 다크 가드 규칙을 따라야 한다.
- 이 PR은 discussion 제안을 구현한 것이며 별도 issue close 대상은 없다.

## 최종 권고

메인터너 lockfile 보정과 code candidate CI를 확인했다. source branch의 오래된 오늘할일 파일은 최신
`devel`의 같은 날짜 기록과 충돌하므로, 최신 `devel` 병합을 문서만을 위해 강제하지 않고 archive review만
trailing docs-only 기록으로 남긴다. 해당 head의 fast-pass aggregate와 merge 직전 최신 상태를 다시 확인한
뒤, 작업지시자 승인에 따라 merge한다.
