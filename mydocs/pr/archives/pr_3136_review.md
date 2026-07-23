# PR #3136 검토 기록 — 브라우저 인쇄/PDF 저장 경로 안정화

## 1. 메타

| 항목 | 값 |
| --- | --- |
| PR | [#3136](https://github.com/edwardkim/rhwp/pull/3136) |
| 작성자 | `postmelee` |
| base / head | `devel` / `task_m100_3126` |
| 관련 이슈 | [#3126](https://github.com/edwardkim/rhwp/issues/3126) |
| 결정 맥락 | [#2657](https://github.com/edwardkim/rhwp/issues/2657) |
| 처리 경로 | collaborator self-merge 후보 |
| 기준 | `upstream/devel@cbddc1cd87084b60685da9a2b4369a4511d86173` |
| 규모 | 33 files, +2,359 / -84, 11 commits |
| 작성일 | 2026-07-23 |

작성 시점 참고값으로 PR은 Draft, `MERGEABLE`, `BLOCKED`이며 초기 CI·CodeQL·Render Diff
preflight는 성공했다. head SHA, mergeable, merge state와 CI는 변동값이므로 merge 전 최신 PR head
기준으로 다시 확인한다.

## 2. 결정과 범위

#2657은 Studio가 직접 PDF bytes를 생성하는 v1 RFC를 보류하고 브라우저 인쇄 경로를 우선
안정화하도록 결정했다. #3126의 추가 결정은 `Ctrl+P`를 알아야만 PDF를 찾을 수 있던 문제를 해결하되,
브라우저가 담당하는 native print UI와 PDF 대상·경로 선택은 강제하지 않는 것이다.

이 PR은 다음 범위로 그 결정을 구현한다.

- `PDF로 저장…`
  - 파일 메뉴의 저장 영역에 별도 아이콘과 진입점 제공
  - 남은 브라우저 단계를 실행 전에 설명
  - 같은 모달에서 진행률을 표시한 뒤 hidden same-origin iframe으로 native print UI 자동 호출
  - 반복 안내 생략 설정과 환경 설정 복원 경로
- `인쇄`
  - same-origin `print.html` 별도 창에 전체 페이지 미리보기 제공
  - 사용자의 미리보기 `인쇄` 클릭으로 native print UI 호출
- 공유 출력 경로
  - opt-in `renderPageSvgWithProfile(page, 'print')`
  - 페이지별 named `@page`, SVG id namespace, font/layout 준비
  - file handle, 파일명, dirty/save 상태 불변
  - `about:blank` 의존 제거

직접 PDF bytes 생성, native PDF backend의 WASM 이식과 HWP/HWPX/HML `SaveFormat`의 PDF 확장은
포함하지 않는다.

## 3. 코드 검토

### Rust/WASM 출력 profile

기존 `renderPageSvg` 소비자를 바꾸지 않고 opt-in API를 추가했다. profile 문자열은 기존
`RenderProfile::parse`를 통과하며 Studio의 print pipeline만 `print`를 명시한다. 편집용 문단부호와
placeholder의 억제는 기존 profile 의미를 재사용하므로 전역 editor 상태를 임시 변경하지 않는다.

### print surface와 lifecycle

초기 same-origin 문서는 `public/print.html`로 고정하고 PDF iframe과 인쇄 미리보기 창의 소유권을
분리했다. 폰트 준비와 두 번의 animation frame 뒤에만 print를 호출하며, 실패·취소·완료 경로의
surface 정리와 중복 실행 게이트를 명시한다.

PDF는 native print 호출 전에 안내 모달을 제거하므로 브라우저 UI 뒤에 가려진 안내가 남지 않는다.
인쇄는 사용자 클릭의 동기 구간에서 미리보기 창을 확보해 popup activation을 보존한다.

### 사용자 경험

사용자 피드백에 따라 인쇄와 PDF를 같은 최종 동작으로 만들던 1차 구현을 수정했다. PDF는 안내 후
자동 native print, 인쇄는 별도 미리보기라는 의미를 유지한다. PDF 아이콘은 기존 테마 스프라이트를
재사용하며 메뉴 위치는 `다른 이름으로 저장…` 바로 아래다.

반복 안내를 끈 경우에도 진행률과 오류는 유지한다. 취소한 체크 상태는 저장하지 않고 설정 저장 실패도
출력 실행을 막지 않는다.

## 4. 검증

| 게이트 | 결과 |
| --- | --- |
| Studio/editor 전체 테스트 | 516 passed |
| focused 설정·인쇄 계약 | 11 passed |
| TypeScript/Vite production build | PASS |
| `cargo test --lib` | 2,530 passed / 7 ignored |
| `cargo clippy --lib -- -D warnings` | PASS |
| #2524/#2525 Rust focused regression | PASS |
| E2E manifest 검사 | PASS |
| Chrome headless #3126 E2E | PASS |
| macOS Chrome 사용자 수동 확인 | PASS |
| `git diff --check` | PASS |

Chrome E2E는 embedded font와 검색 가능한 SVG/PDF text, #2525 7쪽/named `@page`, 안내 생략 설정,
PDF iframe과 인쇄 미리보기 surface, 실행 전후 저장 상태 불변을 검증한다.

## 5. 시각·브라우저 검증 판단

renderer와 WASM API가 바뀌므로 출력 검증 대상이다. 다만 변경 목적은 한컴 기준 레이아웃 자체를
보정하는 것이 아니라 기존 renderer를 명시적인 print profile로 호출하고 브라우저 인쇄 surface를
안정화하는 것이다.

따라서 일반 화면 visual sweep 대신 다음 동등 근거를 사용했다.

- #2524 embedded font fixture의 1쪽 CDP PDF와 text 추출
- #2525 7쪽 fixture의 모든 page DOM, named `@page`, CDP PDF 페이지 수와 text 추출
- PDF 안내 모달과 별도 인쇄 미리보기의 Chrome 시각 확인
- 기존 screen profile 소비자를 변경하지 않는 opt-in API 계약

Windows Edge의 native print UI, popup, iframe print와 mixed named `@page`는 현재 환경에서 확인할 수
없으므로 merge 전 잔여 게이트다.

## 6. 리스크와 잔여 조건

- 브라우저 API로 native print UI의 PDF 대상·파일명·저장 성공을 강제하거나 판별할 수 없다.
  PR은 남은 단계를 안내하고 저장 성공 메시지를 표시하지 않는다.
- Edge가 hidden iframe `print()` 또는 page별 named `@page`를 Chrome과 다르게 처리할 가능성이 있다.
  실증 시에만 Stage 0에서 검토한 same-origin 전용 창 fallback을 구현한다.
- PR이 renderer, WASM과 Studio를 함께 변경하므로 GitHub Actions의 최신 head 전체 결과를 확인해야 한다.

## 7. merge 권고

현재는 Windows Edge 검증을 남긴 Draft를 유지한다. Ready 전환과 merge 후보 조건은 다음과 같다.

1. 이 review 문서를 포함한 최신 PR head의 required CI와 CodeQL이 성공한다.
2. Windows VDI Edge에서 PDF iframe, 인쇄 미리보기, 7쪽/mixed `@page`, 취소 뒤 상태 불변을 확인한다.
3. Edge 결과를 Stage 3/4 또는 최종 보고서에 반영한다.
4. 최신 `mergeable`과 branch protection 상태를 재확인한다.
5. 작업지시자의 Ready 전환 및 merge 승인을 받는다.

별도 `pr_3136_review_impl.md`는 만들지 않는다. 구현 단계와 fallback 조건은
`mydocs/plans/task_m100_3126.md`와 Stage 0~4 기록에 이미 분리돼 있어 같은 내용을 중복하지 않는다.
