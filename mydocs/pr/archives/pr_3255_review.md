---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3255 검토 기록 — perf(studio): improve responsive UX and Korean input latency

## 1. PR 메타 (작성 시점 참고값)

| 항목 | 값 |
| --- | --- |
| 번호 | [#3255](https://github.com/edwardkim/rhwp/pull/3255) |
| 작성자 | humdrum00001010 (IlYoung) · 외부 contributor |
| base | `devel` |
| head | `1ea25c2fc81f807cc2df7e6d71d7e8745df8349c` (작성 시점 참고값, merge 전 재확인) |
| rebase base | `upstream/devel` `4b55144573b299b944651bb7f19017d7cbd70f0b` |
| 규모 | 73 files, +8919 / −391 (코드 약 +4045, 나머지는 계획 문서·테스트·asset) |
| mergeable | `MERGEABLE` / `CLEAN` (작성 시점 참고값, merge 전 재확인) |
| 종결 이슈 | [#3243](https://github.com/edwardkim/rhwp/issues/3243) ~ [#3254](https://github.com/edwardkim/rhwp/issues/3254) 12건 일괄 |

**최종 merge 조건**: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인.

## 2. 관련 이슈 요약

| Issue | 범위 | 대표 커밋 |
| --- | --- | --- |
| [#3243](https://github.com/edwardkim/rhwp/issues/3243) | 좁은 화면 아이콘 툴바 그룹 줄바꿈 | `2307a5c9` |
| [#3244](https://github.com/edwardkim/rhwp/issues/3244) | 트랙패드 줌 응답·정착 속도 튜닝 | `b7f7af24`, `e6f88746` |
| [#3245](https://github.com/edwardkim/rhwp/issues/3245) | 포인터·중심 줌 앵커, fit 왕복 보존 | `a1abbe00`, `0a1b59f8` |
| [#3246](https://github.com/edwardkim/rhwp/issues/3246) | 문서 폭 경계 수평 팬 좌표 안정화 | `92d4a72f` |
| [#3247](https://github.com/edwardkim/rhwp/issues/3247) | 세로 휠의 미세 가로 델타 차단 | `74912dd6` |
| [#3248](https://github.com/edwardkim/rhwp/issues/3248) | 본문 page-local 입력 경로 + idle 경계 pagination | `3754fb05`, `26c5b643`, `141299fd` |
| [#3249](https://github.com/edwardkim/rhwp/issues/3249) | 반응형 grouped 서식 리본·모바일 글자 크기 | `625f7667`, `1bc0709b`, `37efe2b7` |
| [#3250](https://github.com/edwardkim/rhwp/issues/3250) | 복구 대화상자 viewport containment | `a57d343e` |
| [#3251](https://github.com/edwardkim/rhwp/issues/3251) | 문서 없이 기본 스타일 표시 | `09798c87` |
| [#3252](https://github.com/edwardkim/rhwp/issues/3252) | landmark·입력 이름·숨은 제목 접근성 | `440b25fc` |
| [#3253](https://github.com/edwardkim/rhwp/issues/3253) | dev 전용 Rust 렌더러 Subsecond hotpatch | `88127b16` |
| [#3254](https://github.com/edwardkim/rhwp/issues/3254) | 표 셀 한글 IME 원자 교체 | `81d476df`, `e9fa2c53`, `e389cd18`, `95b5dc2e` |

## 3. 변경 범위 분석

### 3.1 핵심 기능 (동작 변경)

- **본문/셀 short-edit 경로 (#3248/#3254)**: Rust에 `replace_body_text_local_native`,
  `replace_text_in_cell_native_deferred_pagination` 신설. `insert_text_in_cell_native_impl`을
  `replace_text_in_cell_native_impl(delete_count, ...)`로 리팩터. 매 입력마다 전체 pagination을
  돌리던 IME/본문 입력을 "문단 로컬 reflow + idle 경계 문서 pagination"으로 분리. WASM export:
  `replaceBodyTextLocal`, `replaceTextInCellDeferredPagination`.
- **Frontend edit 라우팅**: `command.ts`에 `canUseLocalBodyTextReplace`,
  `canUseDeferredCellTextReplace`, `replaceBodyTextWithMutationEffects`,
  `replaceCellTextWithMutationEffects`. `TextMutationEffects` 필드를 셀 전용
  (`deferredPagination`/`cellFlowChanged`)에서 문서 공통(`documentPaginationPending`/`flowChanged`)
  으로 일반화. IME 조합 삭제+삽입을 `replaceTextAtRaw` 단일 원자 경로로 통합.
- **Deferred pagination 정책**: 10s/30쪽 조건부 auto-flush → 120ms idle flush로 교체,
  navigation/undo/redo/blur/deactivate/dispose 경계 pre-flush 추가.
- **줌/팬/반응형 (#3243~#3247, #3249, #3250)**: `viewport-manager` smooth zoom,
  `zoom-anchor`/`zoom-fit` 신규 헬퍼, `virtual-scroll` 수평 팬/세로 휠 락, 반응형 CSS,
  복구 대화상자 containment.
- **접근성 (#3251/#3252)**: 편집 입력 `aria-label`, landmark, 문서 없이 기본 스타일 표시.
- **Subsecond dev hotpatch (#3253)**: `src/subsecond_dev.rs`, `wasm_api.rs`의 렌더 함수를
  free-function으로 추출해 `HotFn` 래핑, `tools/rhwp-subsecond` workspace 멤버,
  `subsecond-runtime.ts` (devtools WS + rAF revision watcher).

### 3.2 메타/구조 변경

- 루트 `Cargo.toml`에 `[workspace]`(멤버 `.`, `tools/rhwp-subsecond`) 추가,
  `crate-type`을 `["rlib","cdylib"]`로 재정렬(Dioxus CLI hot-patch 인자 캡처용 주석 동반),
  optional dep `subsecond = "=0.7.9"` + feature `subsecond-dev`(default 미포함).
- `vite.config.ts`: `RHWP_SUBSECOND=1`일 때만 dev wasm alias/`/_dioxus` 프록시 활성.
- `mydocs/plans/task_m100_{issue}.md`(+`_impl.md`) 이슈별 계획 문서, `mydocs/orders/20260724.md`,
  `mydocs/pr/assets/pr_3255/studio_korean_ime_typing.webm`.

### 3.3 범위 외 / 미포함

- canonical manual, `CONTRIBUTING.md` 무변경 (PR 본문 명시, 확인함).
- 파서/serializer/golden/baseline/기존 샘플 무변경.

## 4. 렌더 영향 · visual sweep 판정

- **판정: visual sweep 후보 해당** (§2.6). 줌/팬/pagination/canvas-view 경로가 바뀐다.
- **단, HWP 문서 콘텐츠 렌더 출력(레이아웃/타이포/paint)을 고치는 PR이 아니라 Studio UX
  변경**이므로 기준 PDF 픽셀 대조의 비중은 낮다. §3.5 원칙상 이 유형의 시각 차이는 참고
  자료이며 그 자체로 merge 보류 근거가 아니다.
- 완화 근거: CI `Canvas visual diff` 통과. 저자 제출 증적으로 한글 IME 빠른 입력 WebM
  (11.9초, `mydocs/pr/assets/pr_3255/studio_korean_ime_typing.webm`, #3248/#3254 근거).
- 재현성 공백: 이 PR은 렌더 콘텐츠 회귀가 아닌 UX 변경이라 원본 HWP/HWPX 기준 PDF 산출은
  해당 없음. 대신 Studio 브라우저 동작이 근거이며 저자·CI가 이를 커버.

## 5. 사전 검증 결과

### 5.1 저자 보고 (rebased head, 종료 코드 0)

`cargo build --release` / `cargo test --release --lib` 2,897 passed·7 ignored /
`cargo test --profile release-test --tests` 실패 0 / native-skia 3종(skia 56, 2225 2, p37 4) /
`cargo fmt --check` / `git diff --check` / `cargo clippy --all-targets -D warnings` /
`cargo test --doc` 4 passed / `tsc --noEmit` / `npm test` 619 passed /
`wasm-pack build` / frontend gates 3 passed / 문서 metadata·링크 검사 errors 0.

### 5.2 GitHub Actions (head `1ea25c2f`, 작성 시점 참고값 — merge 전 재확인)

전 required check pass: Default-feature tests 8-shard, Native Skia tests, Lint(fmt/clippy/WASM),
Frontend package gates, Canvas visual diff, CodeQL, Analyze(rust/js-ts/python).

### 5.3 리뷰어 로컬 독립 재현 (PR head 체크아웃)

- `cd rhwp-studio && npx tsc --noEmit` → 종료 코드 0.
- `cd rhwp-studio && npm test` → **619 passed / 0 fail** (저자 보고 수치와 일치).
- `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs`
  → 로컬 `pkg/rhwp.d.ts` stale로 `frontend-wasm-bindings`만 실패. 테스트 자체가
  `"pkg/rhwp.d.ts is stale; rebuild WASM before frontend gates"`를 보고하는 freshness 게이트이며,
  신규 `js_name` export(`replaceBodyTextLocal`, `replaceTextInCellDeferredPagination`)를
  담지 못한 옛 로컬 빌드가 원인. **PR 결함 아님** — CI는 WASM 선빌드 후 통과.
- Rust 전체 게이트는 CI 결과로 갈음(로컬 재빌드는 CI와 중복, 10분+ 소요).

## 6. 주요 리스크 / 관찰

1. **대형 일괄 PR (블로커 아님, 정책 사항)**: >1000라인, 12개 이질 관심사(줌/팬/리본/IME/hotpatch/
   접근성)를 한 PR에 통합. §2.3상 "별도 검토 사이클" 대상. 회귀 시 bisect 난이도↑. 다만 커밋이
   이슈별로 분리되고 이슈별 계획 문서가 있어 추적성은 확보.
2. **Subsecond hotpatch 프로덕션 격리 — 3중 확인 완료**: ① Rust `subsecond-dev` feature가
   `default` 미포함 ② vite alias 기본값 `pkg/rhwp.js`, dev wasm은 `RHWP_SUBSECOND=1` 전용
   ③ 런타임 no-op — `SubsecondRevisionWatcher.start()`는 `subsecondProbe` export 부재 시 즉시
   반환, `connectSubsecondDevtools`는 `applySubsecondDevtoolsMessage` 부재 시 null(WS 미연결).
   신규 dep `subsecond` optional·dev 전용 → WASM 번들 무영향.
3. **edit 라우팅 안전성 — 확인 완료**: 머리말/꼬리말/각주 분기는 `insertTextAtRaw`/`deleteTextAt`
   상단에서 먼저 걸러진 뒤에야 body-local 경로에 도달. `replaceTextAtRaw`도 fast-path마다
   `!isInHeaderFooter && !isInFootnote` 가드. body-local 경로가 헤더/푸터 문단을 오편집할 여지 없음.
4. **page-count drift — 없음**: flow 미변경 본문 편집은 `afterPageLocalEdit`/`afterEdit`에서
   `deferredPaginationPending` 확인 후 120ms idle flush 예약, 경계 pre-flush가 안전망.
   `parseLocalBodyTextReplaceResult`가 `flowChanged && documentPaginationPending` 동시 참을
   불변식 위반으로 차단.
5. **WASM 브리지 graceful fallback**: 신규 export 부재 구버전 wasm에 대해 delete+insert 폴백 보유.
6. **`crate-type` 재정렬 / 루트 `[workspace]` 추가**: wasm-pack은 crate-type 순서 무관, CI WASM
   check·전체 빌드 통과로 무해 확인.
7. **사소한 문서 공백**: #3252만 `task_m100_3252.md` 계획 문서 부재(타 이슈는 있음). 블로커 아님.

## 7. 최종 권고

**admin merge 권고.** 순수 코드 리뷰 관점에서 correctness blocker 없음. 설계(프로덕션 격리,
edit 라우팅 가드, deferred pagination 배선, 브리지 폴백)가 견고하고, CI 전 green + 리뷰어 로컬
프론트엔드 독립 검증(tsc + 619 테스트) 통과. 저자 결정 사항은 "12개 이슈 일괄 PR을 그대로 수용"
이며, 작업지시자가 그대로 수용을 확정함.

- 최종 merge 전 PR head 최신 커밋 기준 GitHub Actions 재확인.
- merge 후 후속: [#3243](https://github.com/edwardkim/rhwp/issues/3243)~
  [#3254](https://github.com/edwardkim/rhwp/issues/3254) close 상태 확인 및 auto-close 여부와
  무관하게 이슈별 검증 요약 코멘트, 원 PR 감사 코멘트.
