# Stage 1 — task_m100_4155 재현·계측

- **이슈**: [#4155](https://github.com/edwardkim/rhwp/issues/4155)
- **계획서**: [`mydocs/plans/task_m100_4155.md`](../plans/task_m100_4155.md)
- **브랜치**: `task_m100_4155_hwp3_char_shade` (분기 기준 `upstream/devel` `e48fe8694`)
- **작업 시각**: 2026-08-09 KST
- **프로덕션 코드 변경**: 0

## 1. 계측 방법

수정 커밋을 되돌린 코드(`git checkout HEAD~1 -- src/parser/hwp3/mod.rs src/model/style.rs
src/parser/hml/reader.rs src/document_core/builders/exam_paper.rs src/model/mod.rs`)에 신규 계약
테스트를 실행하는 **변이 검증**으로 잰다. 별도 프로브 스크립트를 쓰지 않았다 — 계약 테스트
자체가 계측기이므로 측정과 회귀 고정의 대상이 어긋날 여지가 없다.

- HWP5: CFB `DocInfo` 를 압축 해제(`FileHeader[36] & 0x01`)하고 `HWPTAG_CHAR_SHAPE` payload 를
  순회. 오프셋 **60..64** 가 `shade_color` 다(레이아웃 정본 `src/parser/doc_info.rs`, 라이터
  `src/serializer/char_shape.rs` 가 같은 순서로 쓴다).
- HWPX: `Contents/header.xml` 의 `shadeColor` 속성.
- HML: 왕복 저장 XML 의 `ShadeColor` 속성.

## 2. 저장 바이트 축 — HWP3 표본 전수 결함

수정 전 계약 테스트 6건이 **전부 red** 였다.

| 축 | 결과 |
| --- | --- |
| ① HWP3 표본 전수 검정 음영 0건 | **FAIL** — 15표본 15건 실패, CHAR_SHAPE 68,744개 |
| ② SO-SUEOP 전건 음영 없음 | **FAIL** — 2,512개 중 2,512개가 `0x00000000` |
| ③ 한컴 실측 회색 정합 | **FAIL** — 4케이스 전부 회색 부재 |
| ④ public 저장 경로 | **FAIL** |
| ⑤ HWPX 축 | **부분 FAIL** — 아래 §4 |
| ⑥ HML 축 | **FAIL** |

이슈 본문의 실측과 일치한다. `hwp3-sample16.hwp` 에서 음영 없음이 아닌 값이
`[0x00000000, 0x0000ff00]` 로 나온 것도 이슈 표의 "`0x00000000` × 6,516, `0x0000ff00` × 4" 와
같다 — `0x0000ff00` 은 팔레트 인덱스 2(초록) × 비율 0 이다.

## 3. 합성 공식 — 기존 셀 helper 는 실측과 어긋난다

이슈는 표 셀용 `hwp3_table_cell_shade_color`(`src/parser/hwp3/mod.rs:378`) 재사용을 제안했다.
그 함수는 흰 바탕 lerp 를 `255 - (255-c)*r/100` 로 쓰는데, 뺄셈이 정수 나눗셈 **바깥**에 있어
절상 쪽으로 구른다.

| 음영 비율 | 한컴 저장본 | 셀 helper `255-(255-c)*r/100` | 실측 정합 `(c*r + 255*(100-r))/100` |
| ---: | --- | --- | --- |
| 15% | `0xd8` | `0xd9` ✗ | `0xd8` ✓ |
| 6% | `0xef` | `0xf0` ✗ | `0xef` ✓ |
| 40% | `0x99` | `0x99` ✓ | `0x99` ✓ |

같은 lerp 의 두 정수 구현이고 40% 만 나눗셈이 딱 떨어져 우연히 일치한다. 이슈 본문의 검산
`255 - 255×15/100 = 216.75 → 216` 은 실수 연산으로 맞았지만, 기존 helper 가 그 식을 구현했다는
전제가 어긋났다. **글자용 helper 를 신설**하고 셀 helper 는 손대지 않는다 — 셀 축은 대응하는
한컴 실측이 없고 기존 기대값(`hwp3_table_cell_shade_color(6, 10) == 0x00E6FFFF`)은 자체 작성이라
근거 없이 바꾸면 검증 없는 변경이 된다.

## 4. "음영 없음" sentinel — IR 은 이미 `0xFFFFFFFF` 로 수렴해 있었다

이슈는 라이터 두 곳(HWPX 는 번역, HWP5 는 통과)만 보고 "HWP5 라이터가 `0 → 0xFFFFFFFF` 로
번역"을 제안했다. 파서·소비자 축을 재니 정본은 이미 `0xFFFFFFFF` 였다.

| 축 | "음영 없음" 표현 | 근거 |
| --- | --- | --- |
| HWPX 파서 | `parse_color_str("none")` → `0xFFFFFFFF` | `src/parser/hwpx/utils.rs` |
| HML (한/글 산출) | `ShadeColor="4294967295"` | `samples/hml/*.hml` 2/2 |
| HWP5 (한컴 산출) | `0xffffffff` × 22,189 | 코퍼스 380건, 검정 0건 (이슈 실측) |
| 은닉 판정 | 상위 바이트 ≠ 0 → 색 없음 | `hidden_text.rs` `opaque_rgb` |

즉 HWPX 라이터의 `if cs.shade_color == 0 { "none" }` 는 canonical 규칙이 아니라 **HWP3 전용
반창고**였다. IR sentinel 을 통일하면 라이터 3종이 무수정으로 정합한다 — `color_hex` 는 이미
`0xFFFFFFFF → "none"` 이다.

변이 검증이 이를 뒷받침한다. ⑤ HWPX 축에서 **SO-SUEOP 의 `shadeColor="none"` 전수 절은
수정 전에도 통과**했고, `hwp3-sample11` 의 실제 음영(`#D8D8D8`)이 없다는 절에서만 실패했다.
이슈가 말한 "HWPX 축은 정상"이 실제로 확인된 지점이다.

## 5. "음영 없음" 판정이 7곳에 4종으로 흩어져 있다

| 판정식 | 위치 | 상태 |
| --- | --- | --- |
| `rgb != 0xFFFFFF && rgb != 0` (마스크 후) | `renderer/svg.rs:2764`, `web_canvas.rs:2257`, `skia/text_replay.rs:385`, `html.rs:372`, `canvaskit_policy.rs:2061` | 정합 |
| `shade_color != 0x00FFFFFF` (마스크 **없음**) | `paint/text_v2.rs:790` | **불일치** — `0xFFFFFFFF`·`0` 둘 다 "효과 있음"으로 샌다 |
| `(shade & 0xFFFFFF) == 0xFFFFFF` | `paint/paint_op.rs:1558` | **불일치** — `0` 을 음영으로 본다 |
| `opaque_rgb` + 흰/검정 | `hidden_text.rs:247,260` | 정합, 그러나 private |

[#3546](https://github.com/edwardkim/rhwp/issues/3546)·[#3557](https://github.com/edwardkim/rhwp/issues/3557)·[#4141](https://github.com/edwardkim/rhwp/issues/4141)·#4155
가 전부 이 계열이다. 그리고 IR 값이 `0 → 0xFFFFFFFF` 로 바뀌는 것만으로
`paint_op.rs:1558` 의 `is_fill_only_glyph_replay` 가 HWP3 에서 false→true 로 뒤집히므로,
술어 통일을 미뤄도 시각 검증 레인은 어차피 필요하다.

## 6. 부수 실측 — 문서화된 단서 하나가 틀렸다

`src/model/style.rs` 의 #4141 Default 가드 테스트 주석은 `shade_color = 0` 에 대해
"hidden_text 판정과 **HML preflight** 가 0 에 의존한다"고 적고 있었다. 확인 결과
`serializer/hml/preflight.rs` 의 `validate_char_shape` 는 리터럴 0 이 아니라
`CharShape::default()` 와 **상대 비교**하고, `shade_color` 는 검사 목록에 **없다**.
`hidden_text` 쪽도 `opaque_rgb` 가 상위 바이트로 먼저 거르므로 `0xFFFFFFFF` 를 "색 없음"으로
받는다. 기본값 변경을 막는 근거가 아니었다.

## 7. 환경 메모

작업 시작 시점의 로컬 체크아웃은 `222e873ab`(2026-07-19)로 `upstream/devel`
`e48fe8694`(2026-08-09)보다 3주 뒤처져 있었다. 이슈가 인용한
`src/serializer/char_shape.rs`·`hwp3_table_cell_shade_color`·
`tests/issue_4141_hwp3_relative_size_contract.rs` 가 로컬에 존재하지 않아 조사 전체를
`git show upstream/devel:` 로 다시 수행했다. 분기도 `upstream/devel` 에서 땄다.
