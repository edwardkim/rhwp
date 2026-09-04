# Task M100 #3137 Stage 4 완료보고서 — focused page partial repaint

## 1. 결론

거대 표 셀의 stable 연속 입력에서 남아 있던 전체 page-tree rebuild와 full Canvas replay를
focused line의 partial repaint로 바꿨다.

최종 production WASM과 새 headless Chrome으로 실행한 24개 시나리오, 800개 측정 sample에서
다음 결과를 확인했다.

| 항목 | 결과 |
| --- | ---: |
| focused geometry / page-tree patch / dirty payload | 800 / 800 / 800 |
| 실제 partial repaint | 713 |
| full repaint / exact cursor query | 0 / 0 |
| sync flush / begin / step | 0 / 0 / 0 |
| stable operation p95 범위 | 0.7–1.3ms |
| mutation p95 범위 | 0.4–0.5ms |
| page repaint p95 범위 | 1.4–1.6ms |
| partial WASM replay p95 범위 | 1.3–1.4ms |
| input → 2-rAF p95 범위 | 8.4–15.6ms |
| long task | 0 |
| frame-budget gate | 24 / 24 통과 |

800개 mutation에 대해 repaint가 713회인 것은 0ms cadence와 IME의 여러 mutation이 같은 animation
frame에 합쳐졌기 때문이다. 같은 page의 dirty rect를 합집합으로 만든 뒤 한 번만 그렸으며,
측정된 repaint 713회는 전부 partial path였다.

Stage 3의 `input → 2-rAF` p95 67.8–85.2ms가 최종 8.4–15.6ms로 내려갔다. 따라서 #3137의
stable 입력 범위에서는 동기 cursor와 화면 반영이 모두 16.7ms frame budget 안에 들어왔다.

## 2. 잔여 지연 분해

먼저 stable tail edit가 기존 page tree 전체를 무효화하지 않고, 캐시된 focused `TextLine`의
`TextRun`만 교체하도록 했다. 이 변경으로 Stage 2에서 확인한 44–66ms `build_page_tree(0)`은
repaint 경로에서도 사라졌다.

그러나 cache patch만 적용한 production 중간 행렬은 다음과 같았다.

| 항목 | 결과 |
| --- | ---: |
| full Canvas replay p95 | 17.9–26.1ms |
| input → 2-rAF p95 | 20.5–31.3ms |
| frame-budget gate | 0 / 24 |

같은 상태에서 `PageRenderTree → PageLayerTree` lowering을 별도로 재면 평균 HWP 0.183ms,
HWPX 0.174ms였다. 따라서 남은 비용은 Rust tree 생성이나 lowering이 아니라, clip 여부와 관계없이
페이지의 모든 text op를 브라우저 Canvas에 다시 넘기는 replay와 font shaping이었다.

## 3. 구현

### 3.1 cached focused line patch

deferred insert/replace/delete의 same-flow 결과에서 다음 조건을 모두 증명한 경우에만 캐시된 마지막
`TextLine`의 자식을 새 `TextRun`으로 교체한다.

- 가로 plain text이며 마지막 visual line의 tail edit
- page-tree cache에서 대상 line이 정확히 하나
- BMP text이고 control, field/range tag, tab, 줄바꿈, numbering, footnote marker가 없음
- left 또는 마지막 줄 justify이고 지원 language run만 사용
- run background, display expansion, char overlap이 없음
- 새 자연 폭이 기존 line의 available width 안
- 문단부호·control code·debug overlay가 꺼져 있음

성공하면 layer-tree JSON cache만 비우고 page tree는 유지한다. mutation 결과에는
`focusedPageTreePatched=true`와 line bbox에 3px padding을 더한 `focusedPagePatch`를 넣는다.
새 cache line은 같은 edit 직후 fresh page build와 bitwise 동등한지 HWP/HWPX insert, IME replace,
delete에서 직접 대조했다.

### 3.2 partial Canvas replay

WASM에 `renderPagePatchToCanvasFilteredWithProfile`을 추가했다. 이 API는 다음 계약을 갖는다.

- rect의 유한값·양수·page 교집합을 검사
- 현재 page tree와 render profile로 `PageLayerTree`를 생성
- 기존 Canvas extent가 현재 page와 정확히 같을 때만 실행
- Canvas 크기와 rect 밖 픽셀을 유지
- transform을 저장·초기화한 뒤 page-space rect만 clip/clear/replay
- plain `TextRun`/`FootnoteMarker`는 `2 × max(line height, font size)`로 확장한 replay
  envelope가 clip 밖일 때만 Canvas 호출과 font shaping 전에 제거
- italic, shadow, outline, emboss/engrave, rotation, vertical text, char overlap처럼 독립적인
  잉크 범위를 갖는 text와 paragraph/control editor mark는 culling하지 않고 fail-closed replay

배경·선·도형 등 나머지 op는 correctness를 위해 기존 순서로 replay하되 Canvas clip이 실제 변경 영역을
제한한다. API 오류는 호출자에 반환해 기존 full repaint로 복구한다.

PR #3745 review에서 layout bbox가 실제 glyph ink를 포함한다는 계약이 없다는 점을 보정했다.
text culling을 전부 제거한 80ms smoke는 정확성에는 안전했지만 focused patch p95 17.6–18.0ms,
page repaint p95 17.7–18.3ms, `input → 2-rAF` p95 19.8–20.4ms로 6개 시나리오 모두
frame budget을 넘겼다(gate 0/6). 위 보수적 envelope와 fail-closed 정책은 같은 smoke에서 각각
1.3–1.4ms, 1.4–1.5ms, 8.3–9.0ms로 6/6을 통과했다. 고정 line box보다 큰 font extent와
독립 잉크 효과의 culling 금지는 native unit test로 고정했다.

no-cull 비교 산출물은 `output/poc/task3137/pr3745-no-cull-smoke/`에 있으며 production WASM
SHA-256은 `5640e034517ec42e1df58874dc97e2bcad4c3a550956210e5dc907a69bcc9be7`이다.
선택 증거인 conservative 80ms smoke는
`output/poc/task3137/pr3745-conservative-cull-smoke/`에 있다. 두 smoke는 정책 선택을 위한
비교이고, 최종 canonical 성능 기준은 §6의 24개 행렬이다.

### 3.3 Studio 전달과 coalescing

WASM bridge, command effect, input handler, Canvas view, page renderer에 typed
`focusedPagePatch`를 전달했다.

- mutation effect가 같은 page의 여러 rect를 받으면 합집합으로 누적
- page가 다르면 patch를 버려 full invalidation 사용
- 다음 `document-page-invalidated`에 one-shot으로 전달
- 이벤트 page와 patch page가 같고 payload가 유효할 때만 partial path 선택
- page에 image 또는 RawSvg plane이 있으면 full repaint 유지
- partial API가 실패하면 같은 refresh에서 즉시 기존 full render 실행

성능 하네스 schema를 5.0으로 올리고 `--require-focused-repaint`를 추가했다. 이 gate는 모든 측정
mutation의 cache patch와 dirty payload, 모든 실제 repaint의 partial 사용, full repaint 0회를
동시에 요구한다.

## 4. 안전한 fallback

다음 경우는 최적화 대상이 아니라 correctness 경계이므로 기존 cache invalidation, exact cursor,
full repaint를 유지한다.

- 저장된 `LineSeg`가 첫 local reflow에서 정규화되는 첫 입력
- non-tail 또는 line/flow 경계를 바꾸는 edit
- pending flow change, shadow pagination commit, 동기 flush
- vertical/복합 line, control·field·tab·줄바꿈·배경·unsupported language가 있는 text
- patch 후보가 없거나 둘 이상, 새 text가 line width를 넘는 경우
- page/revision/target 불일치 또는 유효하지 않은 dirty rect
- image/RawSvg page, Canvas extent 불일치, partial API 예외

이 경계들은 느린 경로가 아니라 문서 전체 기하나 비동기 resource가 바뀔 수 있는 명시적
재동기화 지점이다. #2214 trace에서도 첫 입력, 56번째 flow 경계, shadow commit 뒤 exact rect
갱신은 기존대로 유지됐다.

이 목록은 편집 대상의 focused fast path admission 조건이다. 같은 페이지의 이웃 text에 위험한
잉크 효과가 있는 경우에는 full-page fallback하지 않고 해당 op를 partial replay에 포함하며,
최종 변경 영역은 Canvas clip을 authority로 삼는다.

## 5. 실행 환경

| 항목 | 값 |
| --- | --- |
| 브랜치 | `issue-3137-pr-review-fixes` |
| review 보정 기준 | `23967640f7aaeb991eb1d2d48938b5c4ce469a4c` + text replay 보정 |
| 측정 worktree | dirty=true, clone 제거 전 culling-fix production WASM |
| 최초 Stage 4 구현 기준 `upstream/devel` | `ad16eb45799645ea96f3ef533b24fd07320ec476` |
| review 24개 행렬 head의 `devel` ancestry | `cc382911684c05c041a6b5ae86a873f7321f2153` |
| Chrome | `150.0.7871.187`, 새 headless 임시 프로필 |
| Node | `v24.15.0` |
| production WASM | 7,452,527 bytes |
| WASM SHA-256 | `dfa390f7e9785b4094396567903ef8ae5bfbc5c7a7d8865331991506a0c32cb4` |
| 최종 성능 결과 | `output/poc/task3137/pr3745-conservative-cull-matrix/` |
| cache-only 중간 결과 | `output/poc/task3137/stage4-final-full-matrix/` |
| correctness 결과 | `output/poc/task3137/stage4-partial-issue2214/` |

## 6. 최종 성능 행렬

각 셀은 `page repaint p95 / input → 2-rAF p95`(ms)다. 영문·숫자는 cadence마다 20개,
IME는 `ㅎ → 하 → 한` 20회에 해당하는 60개 event sample을 측정했다.

| 포맷·입력 | 0ms | 80ms | 150ms | 250ms |
| --- | ---: | ---: | ---: | ---: |
| HWP 영문 | 1.5 / 14.6 | 1.4 / 9.3 | 1.5 / 9.0 | 1.6 / 15.1 |
| HWP 숫자 | 1.5 / 14.7 | 1.5 / 9.2 | 1.5 / 8.5 | 1.5 / 13.5 |
| HWP IME | 1.4 / 15.1 | 1.4 / 9.2 | 1.4 / 8.7 | 1.5 / 13.5 |
| HWPX 영문 | 1.4 / 15.6 | 1.4 / 8.4 | 1.4 / 8.5 | 1.4 / 13.6 |
| HWPX 숫자 | 1.5 / 14.6 | 1.4 / 8.4 | 1.5 / 8.7 | 1.5 / 13.7 |
| HWPX IME | 1.5 / 15.0 | 1.4 / 8.4 | 1.4 / 8.5 | 1.5 / 15.2 |

24개 시나리오와 800개 event sample 모두 최종 text, cursor offset, 원본 format, 115쪽,
deferred mutation 계약을 만족했다. geometry/page-tree patch/dirty payload는 800/800/800,
frame coalescing 뒤 실제 repaint 713회는 모두 partial이었다. exact cursor query, full repaint,
long task와 동기 pagination 작업은 모두 0회였다.

단계별 변화는 다음과 같다.

| 단계 | 지배 측정값 | 결과 |
| --- | --- | ---: |
| Stage 2 | exact cursor/page-tree build p95 | 44–66ms |
| Stage 3 | stable operation p95 | 0.7–2.3ms |
| Stage 3 | input → 2-rAF p95 | 67.8–85.2ms |
| Stage 4 cache patch | full Canvas replay p95 | 17.9–26.1ms |
| Stage 4 cache patch | input → 2-rAF p95 | 20.5–31.3ms |
| Stage 4 partial repaint + review 보정 | page repaint p95 | 1.4–1.6ms |
| Stage 4 partial repaint + review 보정 | input → 2-rAF p95 | 8.4–15.6ms |

## 7. #2214/#2424 correctness 게이트

최종 WASM과 TypeScript 상태에서 HWP/HWPX를 각각 1회 실행했다.

| 항목 | HWP | HWPX |
| --- | ---: | ---: |
| stable operation p95 | 0.7ms | 0.8ms |
| flow boundary | 56 | 56 |
| boundary operation | 85.5ms | 83.0ms |
| boundary begin | 36.5ms | 35.5ms |
| begin / steps / flush | 1 / 115 / 0 | 1 / 115 / 0 |
| Backspace / Delete WASM | 1.9 / 1.7ms | 1.5 / 1.6ms |
| raw stable/boundary IME·iOS | GREEN | GREEN |
| save barrier | HWP 229,376 bytes | HWPX 226,139 bytes |
| print barrier | 115쪽 | HWP suite에서 통과 |

flow 경계 전후 visual crop과 pagination 완료 crop은 기존 비교 계약을 통과했다. 저장과 인쇄는
pending pagination을 먼저 flush한 뒤 export/render하는 순서를 유지했다.

## 8. 테스트와 판정

| 검증 | 결과 |
| --- | --- |
| Rust cached line과 fresh page tree HWP/HWPX 대조 | 통과 |
| Rust `cargo test --lib` | 2,983 passed / 0 failed / 7 ignored |
| Rust `cargo clippy --lib -- -D warnings` | 통과 |
| Rust `cargo fmt --all --check` | 통과 |
| Rust `cargo check --lib` | 통과 |
| production `wasm-pack build --target web --out-dir pkg --release` | 통과 |
| Studio `npm test` | 679 passed / 0 failed |
| Studio `npm run build` | 통과 |
| Stage 4 전체 성능 행렬 | 24 / 24, 800 / 800 |
| #2214 HWP/HWPX focused/raw/delete/IME/save | 전 단계 통과; print는 HWP 1회 통과 |

PR #3745 review 보정을 모두 반영한 source head `6dd0795af35fd030c2ef3fae0fb22cc28092d10c`에서는
전체 행렬을 반복하지 않고 변경 범위에 맞춘 제한 검증을 수행했다.

| 2026-08-02 review 최종 검증 | 결과 |
| --- | --- |
| text replay envelope native unit | 2 / 2 |
| focused exact-tree / IME replace / deferred delete Rust | 3 / 3 |
| Studio type check | `npx tsc --noEmit` 통과 |
| focused Studio unit | 76 / 76 |
| production WASM | 7,452,333 bytes, SHA-256 `5c42bdf6d6d775bc27a5f0c9181d9c4414b8b65bb3dbe9ab0d9ffb3317da22a7` |
| 최종 source 80ms smoke | 6 / 6, operation p95 0.7–1.1ms, page repaint p95 1.4–1.5ms, 2-rAF p95 7.5–9.0ms |
| 최종 source repaint 계약 | partial 200, full/exact/flush/long task 0 |
| #2214 HWP/HWPX 1회 | focused/raw/delete/IME/save 통과; print는 HWP 1회 통과 |

시각 crop은 HWP/HWPX 모두 55→56 입력에서 7,404 pixel이 바뀌어 4→5줄 전환을 포착했고,
56 입력의 2-rAF 뒤 100ms·850ms·1600ms·pagination 완료 이미지는 changed pixel 0과 동일
SHA-256을 유지했다. 대표 자산은
`mydocs/pr/assets/pr_3745_issue3137_partial_repaint_review.png`이다. 이 transient Canvas 경로는
정적 PDF visual sweep으로 재현되지 않으므로, browser crop과 `--require-focused-repaint` trace를
결합해 판정했다.

Stage 4로 #3137의 stable 셀 입력 병목은 cursor와 repaint 양쪽에서 해소됐다. 첫 입력 정규화,
flow 경계, pagination commit, 복합 content page는 correctness를 위한 의도적 full fallback이므로
별도 범위를 요구하지 않는 한 #3137 완료 조건에는 포함하지 않는다.
