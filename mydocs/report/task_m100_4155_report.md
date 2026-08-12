# task_m100_4155 최종 보고서 — HWP3 글자 음영 검정 오염

- **Issue**: [#4155](https://github.com/edwardkim/rhwp/issues/4155)
- **계획서**: [`mydocs/plans/task_m100_4155.md`](../plans/task_m100_4155.md)
- **단계 기록**: [stage1](../working/task_m100_4155_stage1.md) · [stage2](../working/task_m100_4155_stage2.md) · [stage3](../working/task_m100_4155_stage3.md)
- **브랜치**: `task_m100_4155_hwp3_char_shade` (분기 기준 `upstream/devel` `e48fe8694`)
- **커밋**: `71f607188`(수정) · `604eaf2f8`(계약) · `5c95b4619`(술어 통일) · `463f44992`(한컴 오라클)
- **작성 시각**: 2026-08-09 KST

## 1. 요약

HWP3 문서를 `rhwp convert` 로 HWP5 변환한 뒤 한컴으로 열면 **본문 전체가 검정 막대**였다.
글자는 제 위치·제 크기로 그려지고 그 위에 순검정 사각형이 칠해진다.

원인은 `convert_char_shape` 가 HWP3 글자 음영의 **음영 비율(글자 모양 offset 25)을 무시**하고
팔레트 인덱스(offset 23)만 읽은 것이다. HWP3 의 글자 음영은 두 값의 조합이고 비율 0 은 음영
없음인데, 실문서에서 그 비율 0 이 압도적 다수다 — `samples/SO-SUEOP.hwp` 는 2,511건 전건이
0 이다. 인덱스 0(검정)만 보고 `0x00000000` 을 썼다.

이슈는 ① 표 셀 helper 재사용 ② HWP5 라이터의 sentinel 번역을 제안했는데, 조사 중 **둘 다
실측으로 조정**했다(§3). 그리고 이 결함이 속한 계열의 뿌리 — "음영 없음" 판정이 7곳에 4가지로
흩어져 있고 그중 둘이 이미 렌더러와 어긋난 것 — 을 함께 접었다(§4).

## 2. 왜 지금까지 안 잡혔나

rhwp 는 이 결함을 **구조적으로 볼 수 없다.** 렌더러가 검정을 "음영 없음" sentinel 로 쓰기
때문이다(`renderer/svg.rs` 등 5곳의 `shade_rgb != 0x00FFFFFF && shade_rgb != 0`). 따라서
`export-svg`·자체 렌더·`convert --verify` 가 전부 정상으로 나온다.

HWPX 축도 가려져 있었다. 라이터에 `if cs.shade_color == 0 { "none" }` 반창고가 있어
`shadeColor="none"` 이 나갔다. **HWP5 바이너리 축만** 깨졌고, 그 축을 보는 눈은 한컴뿐이었다.

[#4141](https://github.com/edwardkim/rhwp/issues/4141) 이전에는 글자가 0.12pt 로 찌부러져
음영 사각형도 0×0 이라 보이지 않았을 뿐이다. 두 결함에 인과는 없다 — 변환본의 `shade_color`
바이트가 #4141 전후 동일하다.

## 3. 이슈 원안에서 조정한 두 지점

### 3.1 합성 공식 — 기존 셀 helper 는 실측과 1씩 어긋난다

이슈는 표 셀용 `hwp3_table_cell_shade_color` 재사용을 제안했다. 그 함수는 흰 바탕 lerp 를
`255 - (255-c)*r/100` 로 써서 뺄셈이 정수 나눗셈 **바깥**에 있고, 절상 쪽으로 구른다.

| 음영 비율 | 한컴 저장본 | 셀 helper | 실측 정합 `(c*r + 255*(100-r))/100` |
| ---: | --- | --- | --- |
| 15% | `0xd8` | `0xd9` ✗ | `0xd8` ✓ |
| 6% | `0xef` | `0xf0` ✗ | `0xef` ✓ |
| 40% | `0x99` | `0x99` ✓ | `0x99` ✓ |

이슈의 검산 `255 − 255×15/100 = 216.75 → 216` 자체는 맞았다. 어긋난 것은 기존 helper 가 그
식을 구현했다는 **전제**다. 글자용 `hwp3_char_shade_color` 를 신설하고 셀 helper 는 손대지
않았다 — 셀 축은 대응하는 한컴 실측이 없고 기존 기대값은 자체 작성이라, 근거 없이 바꾸면
검증 없는 변경이 된다(§7 후속).

### 3.2 sentinel — IR 은 이미 `0xFFFFFFFF` 로 수렴해 있었다

이슈는 라이터 두 곳(HWPX 는 번역, HWP5 는 통과)을 보고 "HWP5 라이터가 `0 → 0xFFFFFFFF` 로
번역"을 제안했다. 파서·소비자 축을 재니 정본은 이미 `0xFFFFFFFF` 였다.

| 축 | "음영 없음" 표현 |
| --- | --- |
| HWPX 파서 | `parse_color_str("none")` → `0xFFFFFFFF` |
| HML (한/글 산출) | `ShadeColor="4294967295"` (`samples/hml/*.hml` 2/2) |
| HWP5 (한컴 산출) | `0xffffffff` × 22,189 / 코퍼스 380건, 검정 0건 |
| 은닉 판정 | 상위 바이트 ≠ 0 → 색 없음 (진짜 COLORREF 규약) |

즉 HWPX 라이터의 `== 0` 가드는 canonical 규칙이 아니라 **HWP3 전용 반창고**였다. IR sentinel 을
통일하면 HWP5·HML 라이터는 그대로 정합하고, HWPX는 `color_hex`의 `0xFFFFFFFF → "none"` 매핑만
사용해야 한다. `0x00000000`은 "진짜 검정 음영"이므로 `#000000`으로 보존해야 하며, 2026-08-11
메인터너 검토에서 남아 있던 `== 0 → "none"` 분기를 제거하고 단위 계약을 추가했다. 이로써 HWPX
charPr id 갭 채움·HML `ShadeColor` 부재 같은 다른 생성 경로와도 같은 sentinel 규칙으로 수렴한다.

#4141 이 `relative_sizes` 를 `CharShape::default()` 한 줄로 고친 것과 같은 모양이다.

## 4. 곁들여 접은 것 — 판정 술어의 분열

"음영 없음" 판정이 7곳에 4가지로 구현돼 있었고 그중 둘은 렌더러와 어긋나 있었다.

| 판정식 | 위치 | 상태 |
| --- | --- | --- |
| `rgb != 0xFFFFFF && rgb != 0` (마스크 후) | renderer 5종 | 정합 |
| `shade_color != 0x00FFFFFF` (마스크 **없음**) | `paint/text_v2.rs` | 불일치 |
| `(shade & 0xFFFFFF) == 0xFFFFFF` | `paint/paint_op.rs` | 불일치 — `0` 을 음영으로 봄 |
| `opaque_rgb` + 흰/검정 | `hidden_text.rs` | 정합, 그러나 private |

[#3546](https://github.com/edwardkim/rhwp/issues/3546)·[#3557](https://github.com/edwardkim/rhwp/issues/3557)·#4141·#4155
가 전부 이 계열이다. `hidden_text.rs` 의 술어가 이미 정답(진짜 COLORREF 규약)이었으므로 그것을
`model::color` 로 승격하고 8곳이 호출하게 했다.

L1 만 해도 IR 값이 `0 → 0xFFFFFFFF` 로 바뀌면 `paint_op.rs` 의 `is_fill_only_glyph_replay` 가
HWP3 에서 뒤집혀 시각 검증 레인이 어차피 필요했다. 같은 레인에서 함께 끝내되 커밋을 분리해
리뷰어가 따로 판단할 수 있게 했다.

## 5. 변경 파일

| 파일 | 변경 | 커밋 |
| --- | --- | --- |
| `src/model/color.rs` (신설) | `NONE`·`opaque_rgb`·`char_shade` + 단위 테스트 | 1·3 |
| `src/model/mod.rs` | `pub mod color;` | 1 |
| `src/model/style.rs` | `CharShape::default().shade_color` → sentinel | 1 |
| `src/parser/hwp3/mod.rs` | `hwp3_char_shade_color` 신설 + 배선 + 단위 테스트 3종 | 1 |
| `src/parser/hml/reader.rs` | `ShadeColor` 부재 → sentinel | 1 |
| `src/document_core/builders/exam_paper.rs` | 명시 `shade_color: 0` 제거 | 1 |
| `src/serializer/hwpx/header.rs` | 실제 검정 음영 `0x00000000`을 `#000000`으로 보존, `none`은 sentinel만 사용 | 메인터너 보정 |
| `tests/issue_4155_hwp3_char_shade_contract.rs` (신설) | 저장 계약 7종 (한컴 오라클 포함) | 2·4 |
| renderer 5종 · paint 2종 · `hidden_text.rs` · `parser/doc_info.rs` | 술어 위임 | 3 |

## 6. 검증

### 6.0 통과 기준

이 이슈는 **값이 틀렸다**는 이슈지 파일이 같아야 한다는 이슈가 아니다. 변환본과 한컴 산출물의
동일성은 기준이 될 수 없다 — §6.4 의 3쪽 대조가 보여주듯 자동 번호·머리말 같은 다른 축의 기존
결함이 남아 있고 그것들은 이 이슈 범위가 아니다. 기준은 두 축이다.

| 축 | 기준 | 오라클 | 결과 |
| --- | --- | --- | --- |
| ① 증상 소멸 | 변환본 본문에 줄 크기 검정 fill 이 없다 | 한컴 PDF (사람 판정) | ✅ §6.4 |
| ② 값 정합 | 우리 `shade_color` ⊆ 한컴이 **같은 문서**를 변환한 값 집합 | `samples/hwp3-sampleN-hwp5.hwp` (CI 자동) | ✅ 8/8 |

②의 정답지가 저장소 안에 있다는 것이 핵심이다. `hwp3-sampleN-hwp5.hwp` 는 한컴이 같은 HWP3
원본을 직접 변환한 산출물이므로, 한컴 없는 CI 에서도 값 정합을 판정할 수 있다. 개수는 맞출 수
없다(한컴은 CHAR_SHAPE 를 중복 제거해 수십 개, 우리는 문단마다 쌓아 수천 개 — 별개 사안).

### 6.1 변이 검증 (red 기준선)

수정 커밋을 되돌리고 계약 테스트를 실행: **6/6 red**. 실패 분포가 이슈 본문과 일치했다.

- HWP3 15표본 전건 검정 — CHAR_SHAPE 68,744개
- `SO-SUEOP.hwp` 2,512 / 2,512 = `0x00000000`
- `hwp3-sample16.hwp` 의 비-sentinel 값이 `[0x00000000, 0x0000ff00]` — 이슈 표의
  "`0x00000000` × 6,516, `0x0000ff00` × 4" 와 같다
- ⑤ HWPX 축은 `SO-SUEOP` `"none"` 전수 절이 **수정 전에도 통과**하고 sample11 음영 보존
  절에서만 실패 — 이슈가 말한 "HWPX 축은 정상"이 확인된 지점이다

### 6.2 게이트

| 게이트 | 결과 |
| --- | --- |
| `--test issue_4155_hwp3_char_shade_contract` | **7 passed / 0 failed** |
| 한컴 자기 변환본 오라클 (8쌍) | **8/8 우리 값 ⊆ 한컴 값** |
| `cargo test --profile release-test --lib` | **3,379 passed / 0 failed** |
| `cargo test --profile release-test --tests` | **5,510 passed / 0 failed** |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 (exit 0, 경고 0) |
| Native Skia `skia --lib` | **58 passed / 0 failed** |
| Native Skia `issue_2225_missing_picture_placeholder` | **2 passed / 0 failed** |
| Native Skia `render_p37_direct_pdf_export` | **4 passed / 0 failed** |
| WASM (`docker compose --env-file .env.docker run --rm --build wasm`) | **통과** — `pkg/` 생성, `wasm-opt` 완주 |

WASM 빌드의 프로젝트 표준은 Docker 경유다(`mydocs/manual/memory/project_wasm_docker_build.md`).
`pkg/` 는 `.gitignore` 대상이라 커밋에 포함되지 않는다.

이 과정에서 **로컬 Docker 이미지가 wasm-pack 버전 고정을 무력화하고 있었음**을 발견했다 —
이미지는 2026-05-03 생성인데 Dockerfile 의 `wasm-pack@0.15.0` 고정은 2026-05-16(`d4a32ab29`)에
들어왔고, `docker compose run` 은 이미지를 다시 굽지 않아 0.14.0 으로 돌고 있었다. `--build` 로
다시 구워 0.15.0 에서 게이트를 닫았다. 상세와 후속 제안은
[stage3](../working/task_m100_4155_stage3.md) §5.1.

### 6.3 시각 증적

수정 전후 `export-svg` 전 페이지 대조([stage3](../working/task_m100_4155_stage3.md) §2).

| 표본 | 달라진 페이지 | 신규 음영 `<rect>` | HWP3 원본 |
| --- | ---: | --- | --- |
| `hwp3-sample16.hwp` | 4 | `#d8d8d8` × 8 | 팔레트 0 × 15% ×8 |
| `hwp3-sample5.hwp` | 1 | `#efefef` × 4 | 팔레트 0 × 6% ×4 |
| `SO-SUEOP.hwp` | 0 | — | 전건 비율 0 |

건수가 이슈의 HWP3 원본 실측과 정확히 일치하고, 수정 전 SVG 에는 이 회색들이 **0건**이었다 —
표 셀 채우기가 아니라 글자 음영으로 새로 그려진 것이 확정된다. L2(술어 통일)는 4표본 전 페이지
**바이트 동일**로 렌더 무영향이 확인됐다.

### 6.4 한컴 판정 — **통과** (2026-08-09, 작업지시자)

이슈가 결함을 잰 것과 같은 계측이다. 변환본을 한컴 2022(12.0.0.535)로 열어 PDF 로 내보내고
원본의 한컴 PDF 와 같은 쪽을 대조했다.

| 자료 | 내용 |
| --- | --- |
| `SO-SUEOP.pdf` | 원본의 한컴 PDF (정답지, 46쪽) |
| `so-sueop-4155.pdf` | **변환본**의 한컴 PDF (47쪽) |

**3쪽 판정: 검정 막대 완전 소멸.** 본문 전체가 정상 판독된다. 이슈가 보고한 "줄 크기 검정
fill 65개, 전부 (0,0,0)"이 0 이 됐다. 결함이 없어졌다는 1차 근거다.

같은 쪽에 **남은 차이**가 있으나 전부 이 이슈 밖의 기존 결함이다 — 음영과 무관하고, 이 변경은
음영 사각형만 건드리므로(SO-SUEOP 자체 SVG 는 수정 전후 바이트 동일) 인과가 없다.

- 자동 번호 글리프 붕괴: 정답지 `(1) 주제`·`가. 줄거리`·`(가) 발단:` → 변환본 `❶ 주제`·
  `. 줄거리`·`) 발단:`
- 머리말: 정답지는 좌우 분리 + 밑줄, 변환본은 붙어 있고 밑줄 없음
- 들여쓰기가 전반적으로 더 깊음, 쪽수 46 vs 47 ([#2151] 계열)

**회색 톤 축**은 `SO-SUEOP` 으로 볼 수 없다(전건 비율 0 이라 음영이 아예 없다). 대신
`hwp3-sample5` 를 쓴다 — 6% 음영 4건이 있고, 한컴 변환본 3종(기본·2018·2024)이 모두
`#EFEFEF` 를 낸다. 우리 변환본도 `#EFEFEF` 로 **값이 정확히 일치**한다(§6.2 오라클 테스트가
CI 에서 이를 고정한다). 시각 확인용 산출물은 `output/issue_4155/sample5-4155.hwp` 다.

[#2151]: https://github.com/edwardkim/rhwp/issues/2151

## 7. 후속으로 넘기는 것

- **`hwp3-sample16` 변환본을 한컴이 열지 못한다** — 이 PR 과 **무관한 기존 결함**이며
  [#4367](https://github.com/edwardkim/rhwp/issues/4367) 로 분리 등록했다. 판정 근거는 아래와 같다.
  - 원본 `samples/hwp3-sample16.hwp` 와 한컴 변환본 `samples/hwp3-sample16-hwp5.hwp` 는
    **둘 다 열린다**. 우리 변환본만 "파일 형식/손상" 계열 대화상자 후 거부된다.
  - 이 변경 **이전** 코드로 만든 변환본도 같이 거부된다. 두 파일의 DocInfo 12,739 레코드 중
    다른 것은 `CHAR_SHAPE` 음영색 6,520건뿐이고 레코드 수·길이·`Section0`·CFB 구조는 전부
    동일하다 — 즉 이 변경은 인과가 없다.
  - [#3676] 이 밝힌 한컴 거부 3계약(구역당 `PAGE_BORDER_FILL`=3, 그림 좌표 비-0, 개체
    `local file version`)을 **전부 통과**한다(구역0: 3 / 그림 7개 중 0 / 개체 16개 중 0).
    네 번째 미발견 계약이거나 다른 계열이다.
  - `convert --verify`·`--verify-pages` 도 통과한다(IR 차이 없음, 64쪽) — rhwp 자신은 이
    결함을 볼 수 없다. #3676 의 계약 테스트는 `hwp3-sample.hwp` 만 덮는다.
- `hwp3_table_cell_shade_color` 의 절상 반올림(15%/6% 에서 ±1). 셀 음영이 있는 HWP3 표본을
  한컴으로 저장해 실측한 뒤 별도 이슈로 글자용과 통일한다.
- `hwp3-sample11` 의 음영 CHAR_SHAPE 가 저장 바이트에는 있으나 렌더 본문 런에서 참조되지
  않는 양상([stage3](../working/task_m100_4155_stage3.md) §2.1). 저장 계약에는 영향이 없어
  이번에 원인을 확인하지 않았다.
- `shade_color: Option<ColorRef>` IR 리팩터링. sentinel 을 아예 없애는 것이 정답이지만
  소비처가 많아 독립 이슈다.
- **검증 환경 두 건** (이 PR 과 무관, 별도 이슈 후보):
  ① `docker compose run` 이 낡은 이미지를 재사용해 Dockerfile 의 wasm-pack 버전 고정(#2233)이
  로컬에서 무력화된다 — 게이트 문서에 `--build` 명시 필요([stage3](../working/task_m100_4155_stage3.md) §5.1).
  ② `dev_environment_guide.md`(WASM canonical)에 Docker 경로 누락 — 2026-07-19 에 이미
  기록된 미해소 문서 불일치([stage3](../working/task_m100_4155_stage3.md) §5.2).

## 8. 부수 정정

`src/model/style.rs` 의 #4141 Default 가드 테스트 주석이 `shade_color = 0` 에 대해 "hidden_text
판정과 **HML preflight** 가 0 에 의존한다"고 적고 있었다. `serializer/hml/preflight.rs` 의
`validate_char_shape` 는 리터럴 0 이 아니라 `CharShape::default()` 와 **상대 비교**하고
`shade_color` 는 검사 목록에 **없다**. `hidden_text` 도 `opaque_rgb` 가 상위 바이트로 먼저
거른다. 기본값 변경을 막는 근거가 아니었으므로 주석을 실측에 맞게 고쳤다.

[#3676]: https://github.com/edwardkim/rhwp/issues/3676
