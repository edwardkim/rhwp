---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
pr: 6570
issue: 6187
author: postmelee
---

# PR #6570 검토 — 눈금자 상시 표시와 resize 무깜빡임

## 결론

**승인.** PR #6570의 code candidate
`88ca4d1bfd1d766aa6e0ff8b426576a285daa443`은 가로·세로 눈금자와 교차 코너를 모든
viewport에 유지하고, 입력 종류에 따라 mouse만 핀을 조작하도록 제한한다. resize 때 canvas
backing bitmap을 먼저 비우던 순서를 제거하고, 예약된 한 `update()` 안에서 최신 크기 동기화와
두 축 paint를 끝내므로 원 이슈 #6187의 창 드래그 깜빡임 원인을 직접 해소한다.

로컬 focused·전체 회귀, 177개 browser snapshot, 실제 mouse drag/undo, 최신 통합 head의
browser smoke와 작업지시자의 native OS 창 드래그 확인을 통과했다. code candidate의 CI,
CodeQL, Canvas visual diff, Proptest와 Adapter inter-diff도 모두 성공했으며 GitHub 상태는
`MERGEABLE`·`CLEAN`이다. 이 문서와 오늘 작업 기록만 추가하는 trailing review commit은
review-only fast-pass로 확인한 뒤 정상 merge commit 방식으로 병합한다.

이 문서의 `승인`은 maintainer self-review 판정이다. 자기 PR이므로 별도 reviewer를 지정하거나
GitHub approve review를 만들지 않는다. 사용자는 “권장 순서대로 진행”을 승인해 최신-head CI 확인,
정상 merge, merge 후 #6187 기록과 대체된 PR #6432의 기여 설명·종료까지 허용했다.

## 라우팅

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`,
  `visual_fixture_evidence.md`, `post_merge.md`
- 24 files, `+4205/-103`이며 1,000줄을 넘으므로 large change 재검토 경로를 적용했다.
- 사용자 화면의 Studio chrome/canvas 변경이므로 문서 조판·한컴 출력 비교 대신 실제 브라우저
  viewport·입력·창 resize 증적을 적용했다.
- PR #6432는 외부 기여자 PR 종료 경로를 적용하되, #6570이 실제 merge된 뒤에만 comment와 close를
  수행한다.

## 메타데이터와 계보

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6570](https://github.com/edwardkim/rhwp/pull/6570) / @postmelee |
| 관련 이슈 | [#6187](https://github.com/edwardkim/rhwp/issues/6187) (`Closes #6187`) |
| 대체 대상 | [#6432](https://github.com/edwardkim/rhwp/pull/6432) (`Supersedes #6432`) |
| base / draft | `devel@0d1540931d59a8712c27f339fcbb71e1c00fd4b1` / 아님 |
| code candidate | `88ca4d1bfd1d766aa6e0ff8b426576a285daa443` |
| devel 통합 commit | `7d4f4a18f023880e602d3c61a70b30a939e1fbca` |
| 변경 규모 | 24 files, `+4205/-103`, 7 commits |
| GitHub 상태 | `OPEN`, `MERGEABLE`, `CLEAN`, code candidate checks 성공 |
| reviewer | self PR이므로 지정하지 않음 |

변경량 대부분은 계획·단계 보고와 공개 브라우저 증적이다. 단일 파일 최대 증가는
`mydocs/report/studio-ruler-6187/browser-snapshots.json`의 2,320줄이며, 제품 코드는
`responsive.css`와 `ruler.ts`, 검증 코드는 ruler 관련 Node/E2E test와 harness에 한정된다.
Rust·WASM·문서 조판·fixture·dependency·lockfile은 바꾸지 않는다.

PR #6432의 contributor head
`276d944a80574350c02ba0f56b4532ff5fbdd81c`은 분석 참고로만 사용했다. commit을 cherry-pick하거나
제품 코드를 재사용하지 않았으므로 #6570에 `Co-Authored-By`를 추가하지 않는다. 대신 PR 본문과 merge
후 #6432 comment에서 선행 시도가 1023/1024 breakpoint 문제를 좁히는 데 기여했음을 명시한다.

## 코드 검토와 보호 불변식

### 표시와 입력 정책

- 반응형 CSS는 너비·높이 breakpoint에서 ruler를 숨기거나 `#editor-area`를 flex로 바꾸지 않는다.
  가로·세로 눈금자와 교차 코너의 20px grid 슬롯은 모든 편집 viewport에서 유지된다.
- 인쇄 media의 눈금자 숨김은 유지한다. 문서를 열지 않은 상태도 배경만 표시하며 존재하지 않는 핀을
  만들지 않는다.
- `pointerType === "mouse"`인 주 포인터의 왼쪽 버튼만 drag를 시작한다. touch, pen, 빈/알 수 없는
  pointer type은 읽기 전용이며 commit을 만들지 않는다.
- 활성 `pointerId`를 추적하고 pointer capture, `pointercancel`, capture 상실, window blur와
  `dispose()`에서 수명주기를 정리한다. 실제 이동 없는 click은 commit하지 않고, 완료한 drag만 기존
  `onCommitPin` 경로를 한 번 호출한다.
- touch에 전역 `preventDefault()`나 `touch-action: none`을 추가하지 않아 문서 scroll·pinch 경로를
  가로채지 않는다.

### resize와 paint

- `viewport-resize`와 document-view-loaded는 갱신만 예약한다. 이벤트 callback에서 canvas width나
  height를 먼저 대입하지 않는다.
- 예약된 `update()`는 두 container 크기와 한 DPR 값을 먼저 읽고, 실제 물리 크기가 달라진 bitmap만
  맞춘 뒤 가로·세로 눈금자를 같은 callback에서 paint한다.
- 동일 크기 resize는 canvas backing store를 초기화하지 않는다. 연속 이벤트는 한 예약으로 합쳐져
  마지막 geometry를 사용한다.
- 문서 재열기·테마·scroll·zoom·focus 갱신은 공통 paint 계약을 유지하며, `dispose()` 뒤 예약 callback과
  전역 pointer listener가 남지 않는다.
- resize는 문서 모델이나 편집 focus를 바꾸는 command가 아니며, 기존 여백·들여쓰기 commit/undo
  경로를 보존한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `npx --no-install tsc --noEmit` | 통과 |
| `npm test` | 1,350 passed / 0 failed / 1 skipped, 총 1,351 |
| focused ruler 입력·resize·표시 계약 | 통과; 입력 내부 23개 시나리오, resize 내부 19개 시나리오 포함 |
| E2E manifest | tracked 125개 / manifest 125행, 통과 |
| 최신 통합 head browser smoke | 767px·1024px 모두 두 눈금자·교차 코너·20px grid, warning/error 0건 |

기존 skip 1개는 `pkg-node/rhwp.js`가 필요한 `pending-char-shape.test.ts`의 자식 process·WASM
왕복 테스트다. 이번 변경과 무관하며 pass로 세지 않았다. Rust source/test와 WASM API diff가 없으므로
필수 Rust lint·Native Skia·WASM build 묶음은 변경 범위상 적용하지 않았다.

## 브라우저·사용자 시각 검증

- 10/50/100% × 세로/가로 쪽 이동과 1023↔1024 각 10회 왕복을 포함한 177개 browser snapshot이
  통과했다.
- 767px에서 실제 mouse로 여백·들여쓰기 핀 drag, command commit, undo와 수치 입력 경로를 확인했다.
- 어두운 테마의 1,280px·375px, 가로 10%와 새 문서 밝은 테마를 별도 화면으로 남겼다.
- 작업지시자가 로컬 서버의 실제 OS 브라우저 창을 직접 드래그해 눈금자 상시 표시와 resize
  깜빡임 제거를 확인하고 Stage 3를 승인했다.

장기 증적:

- `mydocs/report/studio-ruler-6187/desktop-1280-100.jpg`
- `mydocs/report/studio-ruler-6187/mobile-375-10.jpg`
- `mydocs/report/studio-ruler-6187/horizontal-10-1280.jpg`
- `mydocs/report/studio-ruler-6187/new-document-light.jpg`
- `mydocs/report/studio-ruler-6187/browser-snapshots.json`

이번 변경의 정답지는 문서 렌더링 결과가 아니라 Studio 눈금자 chrome의 표시·입력·resize 수명주기다.
따라서 한컴 PDF나 pixel visual sweep은 적용하지 않았고, 공개 sample의 실제 브라우저 화면과 사용자의
native 창 드래그를 동등 증거로 사용했다. 화면별 pixel-match 또는 visual-accuracy 수치는 측정하지
않았으므로 만들거나 주장하지 않는다.

## code candidate GitHub Actions

- [CI 33497285551](https://github.com/edwardkim/rhwp/actions/runs/33497285551): CI preflight,
  Frontend package gates, Build & Test 성공. Rust·archive 계열은 Studio-only impact에 따라 정책대로 skip.
- [CodeQL 33497285532](https://github.com/edwardkim/rhwp/actions/runs/33497285532): preflight와
  JavaScript/TypeScript·Rust·Python 분석 성공.
- [Render Diff 33497285245](https://github.com/edwardkim/rhwp/actions/runs/33497285245): preflight와
  Canvas visual diff 성공.
- [Proptest roundtrip 33497285623](https://github.com/edwardkim/rhwp/actions/runs/33497285623): preflight와
  prop roundtrip 성공.
- [Adapter inter-diff 33497285579](https://github.com/edwardkim/rhwp/actions/runs/33497285579): preflight와
  adapter inter-diff 성공.

## 잔여 위험과 분리한 후속 결함

- Browser 도구로 실제 touch/pen을 dispatch하지 못했다. 입력별 Node 계약은 통과했지만 실제 모바일
  장치의 조작 불가와 scroll/pinch 공존은 후속 실기 검증 대상이다.
- 기존 E2E 원본 전체 자동 실행과 모든 compositor frame의 직접 관측은 수행하지 못했다. 177개
  snapshot·동작 회귀·native 창 드래그를 결합했고 사용자가 이 제한을 본 뒤 Stage 3를 승인했다.
- 세로 눈금자 마지막 라벨 `42`는 가로 마지막 라벨 숨김 정책을 소유한 PR #6458에 대칭 보정한다.
  #6187의 상시 표시·resize 수정과 분리한다.
- macOS Firefox에서 10%·13%·14% pinch가 브라우저 zoom으로 이탈하는 문제는 #6187이 바꾸지 않은
  `ViewportManager` wheel listener 범위의 별도 결함이다. 이 PR의 회귀로 판정하지 않는다.

## 최종 판정과 merge 조건

- 판정: **승인**
- 판정 대상: code candidate `88ca4d1bfd1d766aa6e0ff8b426576a285daa443`
- 완료 조건: 최신 `devel` 통합, 로컬 전체 회귀, 사용자 Stage 3 승인, code candidate GitHub Actions
  성공, `MERGEABLE`·`CLEAN` 확인
- trailing 조건: 이 review·오늘 작업 기록만 추가한 최신 head에서 fast-pass checks 성공과
  `MERGEABLE`·`CLEAN`을 다시 확인
- merge 방식: `--admin` 없이 정상 2-parent merge commit
- GitHub review: self PR이므로 approve/reviewer 지정 없음

## merge 후 처리 계획

1. #6570의 실제 merge commit과 시간을 확인한다.
2. #6187 자동 종료 여부를 확인하고, 해결 정책·검증·후속 분리를 maintainer comment로 기록한다.
3. #6432에 replacement PR과 merge SHA, 독립 구현·미체리픽 사실, 로컬·CI 검증, breakpoint 분석에
   도움을 준 기여를 기록한다.
4. #6432를 “기여 부족”이 아니라 최종 정책 변경으로 superseded 처리해 close한다.
5. contributor fork branch는 삭제하지 않는다. 로컬 작업 브랜치는 clean 상태와 `devel` 동기화를
   확인한 뒤 정리하며, 원격 task branch는 별도 삭제 승인 없이 남긴다.
