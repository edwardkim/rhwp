# #8201 구현 계획 — 합성 PUA 기호의 일반화와 서명·날인 기호

상위 계획: [task_m100_8201.md](task_m100_8201.md)

## 설계 원칙

네모 숫자(#6127/#7642)는 이미 "평문 PUA를 1em으로 재고 paint에서 테두리+안쪽 글자를 합성"한다.
그 판정 `boxed_pua_number`는 Rust 백엔드·측정·정책 약 12곳과 TS 약 4곳에 같은 모양의 분기로
흩어져 있다. 서명·날인 기호를 위해 같은 분기를 한 벌 더 복제하지 않고, 판정과 기하를
"합성 PUA 기호" 하나로 일반화한 뒤 seal을 변형 하나와 데이터로 추가한다.

공통 결과의 소유는 다음과 같다.

```
enclosed_pua(ch)                         ← 판정 (renderer/mod.rs)
  ├─ 측정: char_width_decision → 1em (자간·장평은 일반 경로)
  ├─ 표시 투영: 원 키 유지 (실제 공급 원 glyph 다음 순위, 표시 대체보다 우선)
  └─ paint: enclosed_pua_geometry::geometry(glyph, font_size)
               → frame(사각|타원) + 안쪽 글자 투영 → SVG / Canvas2D / Skia / CanvasKit
```

## 판정과 데이터

- `src/renderer/mod.rs`: `EnclosedPua::{BoxedNumber(u32), CircledSeal}`와 `enclosed_pua(ch)`.
  `boxed_pua_number`는 CharOverlap 의미(`boxed_pua_char_overlap_semantics`)와 다자리 폴백이 계속 쓴다.
- `assets/fonts/boxed-digit-metrics.json` → `assets/fonts/enclosed-pua-metrics.json`(git mv).
  `innerFace`, `boxedDigit {frame, digits}`, `circledSeal {text, frame, source, target}`.
  seal 값(UPM 1024, frame은 바깥/안쪽 윤곽 중간의 중심선):
  - frame `[63.5, -829.5, 902, 892.5, 26.25]` (x, y-down top, w, h, 획 = 가로 27·세로 25.5 평균)
  - source `[116, -69, 912, 842]` (Haansoft Batang 일반 '인')
  - target `[181, 34, 801, 757]` (GID 3 안쪽 '인')
- `src/renderer/boxed_digit_geometry.rs` → `enclosed_pua_geometry.rs`(git mv).
  `geometry(glyph, font_size) -> Option<EnclosedGeometry>`가 frame 모양·중심선·획과 안쪽 글자의
  x/y 스케일·원점을 한 투영식으로 산출한다. 다자리(10~20) 네모 숫자는 기존처럼 `None` → 폴백.
- 패키지 목록(`Cargo.toml` include, `packaging/crates-io/runtime-files.json`,
  `packaging/crates-io/source-policy.json`)의 경로를 함께 바꾼다.

## 표시 투영 (composer)

`hancom_pua.rs`의 `(0xF012B, "(인)")`는 **읽기 문자열**로 남긴다. 텍스트 추출과 글자겹침 안쪽
표시는 이 값을 계속 쓰므로 새 분기가 필요 없다. HWP5 `0xA12B`의 평면 15 정규화 조건도 그대로다.

합성 기호는 기존 "실제 공급 원 glyph는 Unicode 표시 대체보다 우선"하는 분기에 합친다.

| 위치 | 변경 |
| --- | --- |
| `pua_surface.inc.rs` `expand_pua_surface` | `program` 또는 `enclosed_pua(key)`이면 원 키 유지 |
| `stored_tac_display.inc.rs` `convert_pua_display_text` | 같은 조건으로 원 키 유지(표시 문자열 변경 없음) |
| `pua_surface.inc.rs` `single_scalar_display_char` | 합성 기호는 정규화한 키 자체가 표시 기호 |

결과: 평문 F012B는 측정·paint 모두 원 키를 받는다. layer JSON의 `displayText`는 생기지 않으며
CanvasKit은 `text`의 원 코드포인트로 합성한다(네모 숫자와 같은 계약).

## 측정

`text_measurement.rs`의 `boxedPuaFullwidth` 분기를 `enclosed_pua(c)`로 일반화한다(전진 1em,
Haansoft Batang `/W` 1000 근거). supplemental metric 제외(같은 파일 2063,
`document_core/canvas_metric_requests.rs`, `skia/text_shaping.rs`)도 같은 판정을 쓴다.

## paint

| 경로 | 위치 | 변경 |
| --- | --- | --- |
| SVG | `svg.rs` 평문 cluster 루프·`draw_boxed_pua_number` | `draw_enclosed_pua`: `<rect>`/`<ellipse>` + 변환한 안쪽 `<text>` |
| Canvas2D | `web_canvas.rs` 효과·기본·shadow 경로 3곳 | 같은 기하. 타원은 `ellipse()` + stroke |
| Native Skia | `skia/text_replay.rs` | `draw_rect`/`draw_oval` + 안쪽 글자 typeface 선택 |
| CanvasKit 정책 | `canvaskit_policy.rs` `text_requires_complex_shaping` | 합성 기호는 nominal replay 대상 |
| CanvasKit TS | `canvaskit-renderer.ts`, `canvaskit/boxed-digit-geometry.ts`→`enclosed-pua-geometry.ts`, `nominal-text-replay.ts`, `core/host-font-requests.ts` | 같은 데이터·투영식, 원 glyph 없을 때만 합성, 안쪽 face 요청 |

다자리 네모 숫자의 기존 0.72em 폴백은 `BoxedNumber`에만 남긴다. CharOverlap 경로는 바꾸지 않는다.

## 접수증 날인선 (#2020/#4384)

`layout.rs`의 `receipt_seal_line_text`는 F081C 채움줄에 심긴 U+F012B 자리를 `----(인)----`
리터럴로 합성했다. `pdf/복학원서-2022.pdf`는 이 점선 가운데에도 서명란과 같은 타원 안 '인'을
찍으므로, 합성 문자열 가운데에 원 기호 U+F012B를 넣어 같은 합성 경로가 측정·paint를 소유하게 한다.

## 테스트

새 integration: `tests/cases/issue_8201_seal_pua.rs` (공개 API만 사용).

| 검사 | 독립 기대값 | 수정 전 |
| --- | --- | --- |
| 이슈 서식 1쪽 seal 전진 = 같은 run의 한글 전진 | PDF 57 vs 58 device unit, 같은 1em·자간 | FAIL(3글자 확장) |
| 이슈 서식 seal 칸 SVG: 타원 1개 + '인', `(인)`·원 PUA 없음 | PDF GID 3 | FAIL |
| 복학원서 서명란: `(Signature)` 앞 타원 + '인' | `pdf/복학원서-2022.pdf` | FAIL |
| HWP5 `0xA12B`(`samples/issue5800-hancom-symbol.hwp`) 도장란도 같은 합성 | #5800 정규화 계약 | FAIL |
| 텍스트 표면·글자겹침 표시는 `(인)` 유지 | 기존 계약 | PASS(반례) |
| 인접 F012A/F012C는 원문 유지 | 미등록 PUA 계약 | PASS(반례) |

기존 검사 교정(근거: 복학원서 PDF, 수정 전 95.40706%):

- `tests/issue_937.rs`: 표시 문자열 → 합성 기호·SVG 타원 검사.
- `tests/issue_948.rs`: displayText 계약 예시를 F081C 채움 run으로 교체(의도 유지).
- `tests/issue_2020.rs`: 날인선 `(인)` 텍스트 검사 → 날인선 run(U+F012B) 상자 안 타원 검사.
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`: 이 golden을 읽는 검사가 현재 없다
  (`svg_snapshot.rs`에 `issue_677` 항목 없음). 소비자가 없는 파일이라 바꾸지 않는다.
- `tests/issue_3216_hf_field_display_space.rs`: 투영 기호를 U+F02FB(`▸`)로 바꾸고, PUA 조각의
  자기 표시값을 단언해 검사 전제를 고정한다.
- `paint/json.rs` 테스트 2개, `canvaskit_policy.rs` work-unit 테스트: 1→3 투영 예시를 옛한글
  PUA U+E0BC(자모 셋)로 바꾼다. `shaped_script…` 목록의 F012B는 합성 기호의 direct replay
  계약으로 그대로 둔다.
- `tests/cases/issue_5800_*`: `0xA12B`가 평면 15 키 U+F012B로 정규화되는 계약으로 교정.
- TS: `rhwp-studio/e2e/renderer-contract.test.mjs`에 seal replay(drawOval + '인') 추가.

## 검증

CONTRIBUTING·AGENTS의 Rust lint 묶음(fmt, native/WASM/workspace Clippy, manifest·unit tier),
`cargo nextest` 전체, 코퍼스 래칫 일곱과 보안 입력(`samples/issue8201/*.hwp`), `oracle_page_count`
원장 등록, Studio `tsc`·관련 e2e, fresh WASM, Native/fresh WASM Visual Sweep(이슈 서식 2쪽,
복학원서 1쪽, `issue5800-hancom-symbol.hwp`, 네모 숫자 대조군)과 seal 칸 확대 직접 판독.

### 검증 환경 주의 (이 Mac)

사용자 `~/Library/Fonts/NanumMyeongjo.ttf`와 시스템 자산 `NanumMyeongjo.ttc`가 PostScript 이름을
공유해 Native Skia가 cmap과 윤곽을 서로 다른 파일에서 가져온다. 명조 대체 체인의 한글이 두 글리프씩
밀린다(복학원서 수정 전 Native PNG "본 인"→"볶 읶"). 코드와 무관한 환경 결함이므로 Native 측정은
수정 전·후 모두 시스템 ttc 사본을 `RHWP_FONT_PATH`로 공급해 다시 잰다. 초기 95.40706%는 오염값이다.
