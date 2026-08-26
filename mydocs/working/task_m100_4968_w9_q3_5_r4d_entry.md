# Task M100 #4968 — Stage W9-Q3-5R4D 진입 감사와 수정 수행계획

- 작성일: 2026-08-27 KST
- 작업 브랜치: `task_m100_4968`
- 기준 커밋: `838d78145` R4C 증적 마감
- 최신 통합 기준: `upstream/devel@9be8b0562455`, merge `dda3eb075`
- 상태: **진입 감사·수정 수행계획 승인, R4D-0 진입**
- 이번 감사의 제품 source 변경: 0
- remote push·PR·comment: 수행하지 않음

## 1. 결론

R4C는 exact kerning measurement로 token 폭과 fresh line boundary를 결정하지만 그 결정 positions를
`TextRunNode`에 보존하지 않는다. 현재 SVG·Web Canvas2D·HTML·native Skia·paint JSON·portable GlyphRun은
layout 뒤에 `compute_char_positions`를 다시 호출한다. 따라서 K1은 줄이 달라져도 glyph paint 위치는 기존 K0
scalar 위치를 재생하며, run 다음 x·bbox·장식·탭 마커도 pair delta를 소비하지 않는다.

R4D는 backend별 shaping을 추가하는 단계가 아니다. **최종 emitted run 경계에서 layout owner가 한 번 계산한
bounded positions를 `TextRunNode`에 선택적으로 게시하고 모든 visual consumer가 같은 값만 재생하는 단계**로
수정한다.

단순히 `TextRunNode`에 `Vec<f64>` 필드만 추가하면 안 된다. footnote/TAC/display text 분할 뒤의 실제 run,
줄 단위 pair crossing 제거, 최종 장평·자간·배분 간격, CanvasKit bounded-work 회계를 동시에 맞춰야 한다.

## 2. 진입 기준선과 감사 증거

### 2.1 R4C 완료 기준선

- #4968 review target: 24/24 통과
- 최신 upstream 집중 회귀: 4/4 통과
- 전체 nextest: 8,427/8,427 통과, 43 skip
- native-skia gate: 통과
- Docker WASM: 9,586,229 bytes
- WASM SHA-256:
  `89bbeee3a6f64eb99347d2feedaef7eebc77c258064e6d6a1e8ed0e9b2ae1be3`
- K0 native/WASM canonical SHA-256:
  `e682aba0e3f80cb4669fded895b0bd1763490b6e339c50070dc3359744d36d45`

`K0`·`K1`·exact font source·identity의 뜻은
`mydocs/working/task_m100_4968_w9_q3_5_r4c4.md` 1.1을 정본으로 삼는다.

### 2.2 현재 유실 지점

```text
KerningParagraphMeasurement / line boundary decision       [R4C: K1]
  -> ComposedLine / ComposedTextRun                         [positions 없음]
  -> TextRunNode                                            [positions 없음]
  -> PaintOp::TextRun                                       [run clone만 전달]
  -> SVG·Canvas2D·HTML·Skia·paint JSON·portable GlyphRun
       compute_char_positions(text, style) 재실행            [K0 재계산]
```

source 감사 결과는 다음과 같다.

- `TextRunNode` literal: 80곳. 필드 추가는 제품·test literal 전건을 명시적으로 갱신해야 한다.
- run/display text를 기준으로 positions를 다시 계산하는 주요 지점: 11곳.
- 직접 재계산 소비 파일:
  - `src/renderer/svg.rs`
  - `src/renderer/web_canvas.rs`
  - `src/renderer/html.rs`
  - `src/renderer/skia/text_replay.rs`
  - `src/paint/json.rs`
  - `src/paint/font_glyph.rs`
  - `src/renderer/canvaskit_policy.rs`
- `src/renderer/canvas.rs`의 command-only `CanvasRenderer`는 positions를 표현할 command 자체가 없고
  `FillText(text, x, y)`만 보존한다.
- `PaintOp::TextRun`과 `SvgLayerRenderer`는 `TextRunNode`를 그대로 복제하므로 공통 필드가 있으면 새 font
  lookup 없이 전달할 수 있다.

### 2.3 최종 emitted run에서 측정해야 하는 이유

문단 전체 adjusted map을 그대로 잘라 쓰면 줄 끝 glyph와 다음 줄 첫 glyph의 crossing pair가 남을 수 있다.
반대로 최종 `TextRunNode`가 확정된 뒤 그 run만 측정하면 다음 경계가 hard boundary가 된다.

- 물리적 줄 경계
- char-shape·language slot 경계
- footnote marker와 TAC 분할
- 탭·inline control
- source text 1자와 display text N자의 projection 경계

또한 paragraph layout은 줄 정렬 뒤 `extra_word_spacing`·`extra_char_spacing`·`extra_dash_advance`를
`TextStyle`에 넣는다. 최종 run 측정은 이 scalar positions를 base로 사용하고 exact pair delta만 마지막에
더해야 bbox·다음 run x·paint가 같은 total width를 소비한다.

## 3. 수정 R4D 절편

### R4D-0 — replay positions 계약과 K0 무변화

`TextRunNode`에 optional `layout_positions`를 추가한다.

- 의미: `display_or_text()`의 Unicode scalar N개에 대한 run-relative N+1 경계값
- 저장 조건: exact source K1 측정 결과가 `PairAdjusted`일 때만 `Some`
- K0·source 부재·unsupported·fail-closed·빈 run: `None`
- serialization: `None`은 `skip_serializing_if`로 완전히 생략
- payload: positions 숫자만 보존하고 source bytes·경로·family provenance·원문 복제는 추가하지 않음

공통 validator/accessor는 다음을 강제한다.

1. 길이 = replay text scalar count + 1
2. scalar count ≤ 4,096, positions count ≤ 4,097
3. 첫 값 0, 전항 finite·non-negative·monotonic
4. 마지막 값과 run bbox/advance가 유한 범위
5. 검증 실패 시 positions를 소비하지 않고 기존 `compute_char_positions`로 fail-closed

80개 literal에는 기계적으로 `layout_positions: None`을 넣되, R4D producer로 확인된 제품 방출 지점만
후속 절편에서 `Some`을 만든다. R4D-0만으로 K0 layer JSON·SVG·Canvas command를 바꾸지 않는다.

### R4D-1 — paragraph layout의 단일 producer

`layout_composed_paragraph` 한 번에 `KerningLayoutSession` 하나만 열고 `emit_line_runs`의 최종 sub-run
방출까지 빌려준다. run마다 독립 session을 만들어 같은 face를 반복 parse하지 않는다.

producer는 다음 순서로 동작한다.

1. footnote/TAC/display projection까지 반영된 실제 replay text와 최종 `TextStyle`을 확정한다.
2. `(char_style_id, lang_index)` exact slot을 사용한다. family 이름으로 source를 다시 찾지 않는다.
3. final scalar `compute_char_positions`를 base로 bounded run measurement를 실행한다.
4. `PairAdjusted`에서만 owned positions를 게시한다.
5. positions 마지막 값을 run bbox width와 다음 run x advance의 단일 값으로 사용한다.
6. leading-space 배경, RangeTag, tab leader, char-x map, decoration도 같은 positions range를 읽는다.

stored-valid 문단은 저장 line boundary를 유지한 채 각 line 내부 K1 positions만 적용한다. fresh 문단은 R4C가
정한 line boundary 안에서 같은 run-local measurement를 적용한다. 세로쓰기·char overlap·회전·구조 marker처럼
현재 pair replay 계약을 온전히 만족하지 못하는 특수 run은 K1을 추정하지 않고 `None`으로 닫는다.

### R4D-2 — 모든 visual consumer의 공통 replay

backend가 직접 `compute_char_positions`를 호출하지 않도록 `TextRunNode` 공통 accessor로 수렴시킨다.

- SVG와 SvgLayer: glyph/cluster x, shade, underline, middle dot, control mark, tab leader
- Web Canvas2D: cluster x, effect pass, shade, decoration, control mark
- HTML: character box와 display projection
- native Skia: cluster x, shade, decoration, paragraph/control mark
- paint JSON: TextRun positions, cluster origin, decoration, tab/control sidecar
- portable GlyphRun: producer positions와 advances
- CanvasKit: TextRun/GlyphRun replay plan의 positions와 동일성
- command-only CanvasRenderer: K1에서 positions를 잃지 않는 positioned command를 별도로 표현하고 K0 command
  형식과 count는 유지

portable/host glyph resolver가 별도 shaping positions를 반환해도 K1 placement의 정답으로 채택하지 않는다.
glyph identity와 cluster가 R4 경계에 맞을 때 layout positions로 projection하고, 맞지 않으면 TextRun fallback을
유지한다. backend에서 kerning을 다시 켜거나 pair delta를 중복 적용하지 않는다.

### R4D-3 — bounded work·검증·결과 보고

optional positions는 새 공격 면이므로 기존 text 길이 검사만 믿지 않는다.

- CanvasKit render-tree·paint-op preflight에 positions count와 byte work를 포함
- JSON bounded prefix는 전체 positions를 무조건 복제하지 않고 허용 범위 slice만 사용
- malformed length·NaN·Infinity·역행 positions fixture는 K0 fail-closed
- 4,096 scalar / 4,097 position 상한 전후 fixture
- K1 positions가 없는 run은 현재 serialization과 byte identity 유지

검증 결과와 backend별 disposition을 별도 R4D 결과 보고서로 작성하고, 메인테이너 승인 전에는 R4E로
진입하지 않는다.

## 4. 보호 불변식

1. K0 `TextRunNode` JSON·paint JSON·SVG·Canvas command는 필드 추가 전과 byte-for-byte 같다.
2. K1 layout positions의 producer는 layout owner 하나이며 backend는 source lookup·shaping을 다시 하지 않는다.
3. line·slot·language·inline-control 경계를 넘어 pair adjustment를 적용하지 않는다.
4. positions와 bbox width·다음 run x·장식·control marker가 같은 좌표계를 소비한다.
5. positions validation 실패는 일부 glyph만 K1으로 남기지 않고 해당 run 전체를 K0로 되돌린다.
6. source bytes·private path·family provenance·private corpus identity를 render tree나 trace에 추가하지 않는다.
7. stored `LineSeg` validity feature detection과 line boundary는 R4D가 바꾸지 않는다.
8. 4,096 scalar 상한과 CanvasKit bounded-work budget을 우회하는 clone·serialization을 만들지 않는다.
9. display text positions는 보이는 문자열 기준이며 source `char_start`·offset 공간을 바꾸지 않는다.
10. GSUB·complex-script·vertical shaping을 이번 단계에서 임의 근사하지 않는다.

## 5. 검증 계획

### 5.1 focused gate

- 기존 `tests/cases/issue_4968_kerning_capability_provider.rs`를 확장하고 새 integration source는 만들지 않음
- K0 `layout_positions=None`과 layer/paint JSON·SVG byte identity
- K1 `AV` positions, bbox total width, 다음 run x, line crossing 제거
- body·table-cell·text-box fresh path와 stored-valid path
- footnote/TAC/tab/display-text hard boundary
- malformed·oversized positions fail-closed
- SVG·Web Canvas2D·HTML·native Skia·CanvasKit·portable GlyphRun canonical positions parity

source-side `#[cfg(test)]`를 바꾸면 `node scripts/rust-unit-test-tiers.mjs --check`를 실행한다.

### 5.2 final gate

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets -- -D warnings`
- review worktree generated manifest prepare/sync/check
- 전체 nextest
- native-skia lib + 관련 PDF/placeholder gate
- Docker WASM build와 R4C 기준선 대비 code-only size/time
- K0 native/WASM canonical identity
- K1 native/WASM measurement·line boundary·positions parity

기존 `wasm-pack test --test`가 모든 native 전용 test를 함께 컴파일하는 토폴로지 한계는 우회 패치를 넣지 않는다.
R4E에서 Docker WASM runtime parity 증거와 별도 harness 후속 필요성을 분리 판정한다.

## 6. target 용량 운영

R4D 동안 같은 파생물을 여러 이름으로 누적하지 않는다.

- root focused 검증은 기본 `target` 하나만 재사용
- `target/r4d-*`, `target/pr-review-*` 같은 장기 named target을 root에 만들지 않음
- 전체 nextest·native-skia는 임시 review worktree 내부 target 하나에서 실행
- review 종료 즉시 worktree와 그 target을 함께 제거
- 단계 경계마다 `du -sh target`과 filesystem 여유를 기록
- R4D 증적 확정 뒤 root Cargo 파생물은 `cargo clean`으로 제거
- Docker가 만든 `pkg`는 메인테이너의 studio 검증 산출물이므로 유지

진입 전 정리에서 root와 도구 workspace Cargo 파생물 483.6GiB를 제거했고 filesystem 여유는 155GB에서
593GB로 늘었다. Git 변경과 `pkg` 9.9MB는 보존했다.

## 7. 중단·재계획 조건

- K0 optional field 추가만으로 기존 JSON/SVG/Canvas command가 변함
- final run positions와 R4C line width를 같은 exact source·scale로 재현할 수 없음
- display text·footnote·TAC 분할에서 source offset과 replay position을 함께 보존할 수 없음
- backend가 layout positions 위에 자체 kerning을 다시 적용함
- portable GlyphRun의 glyph identity가 R4 경계를 만족하지 않아 layout positions를 투영할 수 없음
- positions payload가 bounded-work 또는 WASM 크기에 유의미한 회귀를 만듦
- stored-valid 문단의 line boundary나 private identity가 변경됨

## 8. 승인 경계

R4D-0 → R4D-1 → R4D-2 → R4D-3 순서로 진행한다. 각 절편은 focused gate와 단계 커밋 뒤 다음 절편으로
넘어간다. 메인테이너가 이 수정 수행계획을 승인했으므로 R4D-0의 positions 계약과 K0 무변화 구현부터
시작한다. R4D-0 결과 승인 전에는 producer·backend 동작을 바꾸는 R4D-1에 진입하지 않는다.
