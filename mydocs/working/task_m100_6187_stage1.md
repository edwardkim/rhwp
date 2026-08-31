# Stage 1 보고 — Task M100 #6187

- Issue: [#6187](https://github.com/edwardkim/rhwp/issues/6187)
- 작성일: 2026-08-31 KST
- 상태: 상시 표시·입력 정책 구현 및 focused 검증 완료. 작업지시자 결과 승인 대기.
- 브랜치: `codex/issue-6187-always-visible-ruler`
- 기준: `upstream/devel@e50792c6341a0b61afc3ffeb687a92fc6a807e69`
- 계획 승인 기록 commit: `ddd9fe37d`
- 수행 계획: [task_m100_6187.md](../plans/task_m100_6187.md)
- 구현 계획: [task_m100_6187_impl.md](../plans/task_m100_6187_impl.md)

## 1. 승인 범위와 결과

독립 구현으로 정정한 계획에 대한 작업지시자의 `진행해줘.`를 수행·구현 계획 및 Stage 1 착수
승인으로 기록했다. 최신 devel 기반에서 직접 구현했으며, 원 PR #6432의 commit을 cherry-pick하거나
해당 PR 브랜치에 보정하지 않았다. 기존 검토 브랜치는 보존했다.

이번 단계는 표시 정책과 입력 정책만 구현했다. resize의 bitmap 초기화·paint 순서는 변경하지
않았으며, 영상에서 관측된 깜빡임을 해결했다고 주장하지 않는다.

## 2. 구현 내용

### 화면 크기에 관계없는 표시

- `responsive.css`의 1023px 이하 및 모바일 눈금자 숨김과 editor grid→flex 전환을 제거했다.
- 모바일 + 500px 이하 높이에서 editor를 flex로 바꾸던 규칙도 제거했다. 메뉴 등 도구 영역 축약은 유지했다.
- 기존 `editor.css`의 20px 눈금자 행·열과 `minmax(0, 1fr)` 문서 영역을 그대로 사용한다.
- 모바일 문서 영역의 `pan-x pan-y pinch-zoom`과 인쇄 시 눈금자 숨김은 유지했다.

### 실제 포인터 종류에 따른 조작

- `Ruler`의 핀 조작을 pointer event로 일원화했다. 실제 `mouse` 주 포인터의 왼쪽 버튼만 허용하며,
  화면 너비나 UA로 조작 권한을 정하지 않는다.
- 승인된 허용 목록에 따라 touch/pen/미확인 입력은 읽기 전용이다. 이 입력에 `preventDefault()`를
  적용하지 않으며, 숫자·눈금·문맥상 표시되는 핀을 숨기거나 흐리게 하지 않는다.
- touch 이후의 호환 mouse event는 구독하지 않으므로 핀 commit을 발생시키지 않는다.
- drag를 시작한 pointer ID와 canvas capture를 추적한다. 다른 pointer가 이동·해제해도 현재 drag에
  영향을 주지 않는다. ID가 0인 경우도 처리한다.
- cancel, capture 상실, 창 blur, 왼쪽 버튼 해제 감지, dispose에서 commit 없이 상태·listener를 정리한다.
- 정상 완료도 commit 전에 정리하므로 commit callback이 예외를 던져도 drag가 남지 않는다.
- 기존 핀 좌표·clamp·`onCommitPin` 경로를 유지했다. 클릭만 하거나 출발점으로 돌아온 drag는 commit하지 않는다.
- 실제 클래스를 테스트에서 로드할 수 있도록 타입 전용 의존성 네 개를 `import type`으로 명시했다.
  WASM API나 문서 변경 경로는 추가하지 않았다.

## 3. 회귀 테스트

| 파일 | 목적 |
| --- | --- |
| `rhwp-studio/tests/ruler-visibility.test.ts` | 화면 CSS의 grid 유지, 인쇄 예외, 모바일 touch-action 계약 |
| `rhwp-studio/tests/ruler-input.test.ts` | 실제 Ruler 입력 시나리오를 실행하는 자식 테스트의 성공·실패 판정 |
| `rhwp-studio/tests/support/ruler-harness.mjs` | DOM/canvas/geometry/rAF를 제어하면서 실제 Ruler와 EventBus 실행 |
| `rhwp-studio/tests/support/ruler-input.cases.mjs` | 입력별 허용, 호환 mouse 차단, 다섯 핀 commit, 혼입·취소·정리 등 23개 시나리오 |
| `rhwp-studio/e2e/responsive.test.mjs` | 모든 기존 viewport 및 767/1023px 경계, 모바일 낮은 높이에서 표시·grid 정렬·overflow 검증 추가 |
| `rhwp-studio/e2e/MANIFEST.md` | 반응형 E2E의 추가 검증 범위 기록 |

입력 harness는 375px 편집 화면에 해당하는 문서 컨테이너 geometry에서 실행한다. 브라우저 레이아웃이나
실제 모바일 장치를 재현하는 테스트는 아니다. E2E에 추가한 검사는 정착한 화면의 표시 계약이며,
resize 중 공백 프레임을 증명하는 검사가 아니다. E2E 파일 작성만 완료했고 실제 실행은 Stage 3 범위다.

### 수정 전 실패와 테스트 실행기 보정

- 기존 CSS에서 화면용 눈금자 숨김/배치 override가 남아 표시 계약 테스트가 실패했다.
- 입력 테스트의 첫 실행은 자식이 부모의 `NODE_TEST_CONTEXT`를 상속해 출력 판정이 실패했다.
  이는 제품 결함 증거가 아니다. 해당 환경값을 자식에서 제외해 독립 TAP 결과를 판정하도록 보정했다.
- 실행기 보정 후, 제품 수정 전 입력 시나리오 21개 중 8개가 실패했다. touch 후 호환 mouse event가
  왼쪽 여백 변경 commit(`hwpunit: 4500`)을 발생시키는 실패를 확인했다. 신규 pointer 조작 계약도 실패했다.
- 제품 수정 후 21개가 통과했고, 취소 시 실제 capture 시작 여부를 강화하고 pointer ID 0·commit 예외
  정리 시나리오를 추가한 최종 23개도 모두 통과했다. 최종 23개 전체를 구 구현에서 실행했다는 뜻은 아니다.

## 4. 실행 검증

검증 환경은 macOS / Node.js 24.15.0이다. 자식 process를 사용하는 focused 테스트는 프로젝트 메모리의
EPERM 방지 지침에 따라 sandbox 밖에서 실행했다. 아래 검증 대상 코드는 이 보고서와 함께 commit한다.

`rhwp-studio`에서 실행:

```sh
node --test \
  tests/ruler-visibility.test.ts \
  tests/ruler-input.test.ts \
  tests/ruler-pin-geometry.test.ts \
  tests/ruler-scale.test.ts \
  tests/ruler-document-load-refresh.test.ts \
  tests/active-page.test.ts \
  tests/active-page-integration.test.ts \
  tests/responsive-toolbar-layout.test.ts \
  tests/mutation-routing-guard.test.ts \
  tests/theme-skin.test.ts
node --experimental-transform-types --no-warnings --test --test-reporter=spec tests/support/ruler-input.cases.mjs
npx --no-install tsc --noEmit
```

| 검사 | 결과 |
| --- | --- |
| 위 focused 테스트 10개 파일 | 56 passed, 0 failed, 0 skipped |
| 입력 시나리오 직접 실행 | 23 passed, 0 failed, 0 skipped; 위 56개 중 입력 테스트 내부 검사이며 별도 합산하지 않음 |
| TypeScript | 통과 |
| `node --check rhwp-studio/e2e/responsive.test.mjs` | 통과 |
| `python3 scripts/check_e2e_manifest.py` | 통과: tracked 파일 122개 / manifest 122행 |
| `python3 scripts/check_markdown_links.py` — 수행·구현 계획 및 이 보고서 | 통과: 문서 3개 내부 상대 링크 이상 없음 |
| `git diff --check` | 통과 |

## 5. 미실행 및 다음 승인 게이트

- resize bitmap 초기화→paint 공백의 실패 재현과 수정: **Stage 2 미착수**.
- Studio 전체 `npm test`, 실제 브라우저 E2E·입력·연속 resize, 시각 증거: **Stage 3 미실행**.
- 실제 모바일/펜 장치 확인은 수행하지 않았다. 모의 입력 테스트 결과와 구분한다.
- Rust 변경이 없으므로 Cargo 검사·WASM 재빌드는 수행하지 않았다.
- 이번 단계에서 로컬 서버의 최종 후보 검증·전달은 하지 않았다. Stage 3에서 실행 코드를 확인하고
  작업지시자가 직접 검증할 URL과 체크리스트를 제공한다.
- GitHub comment/review, remote push, 새 PR 생성, 기존 PR/이슈 종료는 수행하지 않았다.

Stage 1 코드·테스트·보고서를 commit한 뒤 작업을 멈춘다. 작업지시자가 Stage 1 결과와 Stage 2 착수를
승인하면, 기존 resize 갱신 경로의 실패를 먼저 고정하고 눈금자 내부에서 크기 변경과 paint를 원자화한다.
