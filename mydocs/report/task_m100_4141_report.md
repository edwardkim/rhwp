# task_m100_4141 최종 보고서 — CharShape 상대크기 0 직렬화

- **Issue**: [#4141](https://github.com/edwardkim/rhwp/issues/4141)
- **계획서**: [`mydocs/plans/task_m100_4141.md`](../plans/task_m100_4141.md)
- **단계 기록**: [stage1](../working/task_m100_4141_stage1.md) · [stage2](../working/task_m100_4141_stage2.md)
- **브랜치**: `task_m100_4141` (분기 기준 `upstream/devel` `0fdac31ba`)
- **작성 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: `src/model/style.rs` 1곳

## 1. 요약

HWP3 문서를 `rhwp convert` 로 HWP5 변환한 뒤 한컴으로 열면 **사실상 백지**였다. 오류·복구
대화상자 없이 열리는데 본문 글자가 전부 ~0.1pt 로 그려진다.

원인은 `CharShape.relative_sizes` 가 **0** 으로 저장되는 것이었다. OWPML 은 `relSz` 를
`xs:positiveInteger` minInclusive=10 / maxInclusive=250, `default="100"` 으로 정의하므로
(`mydocs/manual/OWPML SCHEMA/Header XML schema.xml:716-728`) **0 은 타입 수준에서 이미 불법**이다.
한컴은 실효 크기를 `기준 크기 × 상대크기%` 로 해석한다 — 10pt × 0% ≈ 0.1pt.

이슈는 `convert_char_shape` 한 곳을 지목했지만, 조사 결과 **모델의 파생 `Default` 가 근본
원인**이었다. `src/model/style.rs` 에 수동 `impl Default` 를 써서 한 곳에서 해소했다.

## 2. 왜 지금까지 안 잡혔나

rhwp 는 이 결함을 **구조적으로 볼 수 없다.** 렌더러가 `relative_sizes` 를 아예 읽지 않는다
(`src/renderer/style_resolver.rs:339-361` 은 `base_size`·`spacings`·`ratios` 만 소비.
`src/renderer/`·`src/paint/` 참조 0건). 그 결과:

| 게이트 | 왜 통과했나 |
| --- | --- |
| `export-text`·`info`·자체 렌더 | 크기 계산에 relSz 를 안 쓴다 |
| `convert --verify` / `export-hwpx --verify` | `src/serializer/hwpx/roundtrip.rs` 가 DocInfo char_shapes 를 **개수만**(`:626-631`), 문단은 `(start_pos, char_shape_id)` 쌍만(`:1069-1092`) 비교. **속성값 미비교** |
| `hwp5_roundtrip_baseline` | `out_of_scope`(`:78-81`)가 `detect_format != Hwp` 를 자동 제외 → HWP3 는 범위 밖 |
| `ir_field_sweep_baseline` | 파서→직렬화→재파싱이 대칭으로 0 을 유지하므로 IR 은 정말 같다 |

#3546·#3557·#3676 과 같은 **"자기정합은 유지되고 한컴만 깨지는"** 계열이다.

## 3. 실측 (Stage 1)

수정 전 바이너리로 HWP3 표본 전수를 변환해 **저장 바이트/XML 에서 직접** 읽었다.

| 축 | 결과 |
| --- | --- |
| HWP5 `convert` | 표본 15건, CHAR_SHAPE **68,744개 전건(100%)이 relSz=0** |
| HWPX `export-hwpx` | 동수 전건 `<hh:relSz ...="0">` — 이슈가 "미실측"으로 남긴 축을 확정 |
| HML | HWP3→HML 은 **도달 불가**(`export_hml_native` 가 HML 출처 전용 `hml_metadata` 요구). 대신 `RELSIZE` 자식 없는 HML 왕복이 `RELSIZE="0"` 방출 — 배포 CLI 로 재현 |
| 인덱스 0 CharShape | `ratios=0`·`base_size=0` 이지만 **전 15표본에서 참조 0건** → 2차 원인 아님 |

`samples/SO-SUEOP.hwp` 의 2,512개는 이슈 본문 수치와 정확히 일치한다.

### 정답지 — 오직 `relative_sizes` 만 잘못됐다

`samples/SO-SUEOP.hwpx`(같은 문서의 한컴산 HWPX) charPr 51개 실측:

| 필드 | 한컴 HWPX | rhwp 변환본 | 판정 |
| --- | --- | --- | --- |
| `relSz` | **51/51 = 100** (편차 0) | 0 | **결함** |
| `ratio` | 95×30, 90×11, 100×8, 97×2 | 95 | 정상 — HWP3 레코드의 진짜 데이터 |
| `spacing` | -1×25, 0×11, -2×10, -3×3 | -1 | 정상 — 진짜 데이터 |
| `offset` | 51/51 = 0 | 0 | 정상 — 0 이 유효값 |

`ratio`·`spacing` 은 편차가 있어 진짜 데이터임이 드러난다. 편차가 0 인 `relSz` 만 결함이다.
선행 실측도 같은 방향이다 — HWPX 60개 문서 4,298개 charPr 에서 `relSz != 100` 은 0건
(`src/document_core/queries/hidden_text.rs:281-282`).

## 4. 수정

`src/model/style.rs` 에서 `#[derive(Debug, Clone, Default)]` 의 `Default` 를 빼고 수동 impl 을 썼다.
**`relative_sizes: [100; 7]` 한 줄만** 파생값과 다르고 나머지 30개 필드는 파생값 그대로다.

이 한 곳이 relSz=0 을 만드는 **6개 경로를 전부** 해소한다 — 전부 `CharShape::default()` 또는
`..Default::default()` 를 통과한다:

1. `src/parser/hwp3/mod.rs:526` `convert_char_shape` (HWP3 레코드에 상대크기 개념 자체가 없다)
2. `src/parser/hwp3/mod.rs:3566` 인덱스 0 placeholder
3. `src/parser/hwpx/header.rs:588` `<hh:relSz>` 자식 부재
4. `src/parser/hwpx/header.rs:848-858` charPr id 갭 채움
5. `src/parser/hml/reader.rs:599-605` `RELSIZE` 부재
6. `src/document_core/html_table_import.rs:627`, `src/document_core/commands/html_import.rs:845`

라이터 셋(HWP5 `serializer/char_shape.rs:21`, HWPX `serializer/hwpx/header.rs:638`,
HML `serializer/hml/head.rs:131`)이 모두 가드 없이 IR 을 방출하므로 3축이 **같은 커밋 하나로**
동시에 초록이 됐다.

부수 효과로 **모델 Default 와 HWP5 파서 폴백의 불일치**가 해소됐다 —
`src/parser/doc_info.rs:542-545` 는 이미 100 을 폴백하고 있었다.

### 함께 바꾸지 않은 것

`ratios`·`base_size` 는 **렌더러가 소비하므로**(`style_resolver.rs:341`,`:355`) 그대로 뒀다.
기본값 변경은 렌더 회귀 검증 lane 을 요구하고, #4141 의 한컴 판정 결과 귀속을 흐린다.
`char_offsets`·`spacings` 는 0 이 스펙상 유효값이고, 색상 필드는 sentinel 로 쓰인다.

이 비대칭을 `char_shape_default_matches_spec_only_for_relative_sizes` 가 코드로 고정한다.

### 기각한 대안

- **`convert_char_shape` 만 수정**(이슈 원안) — 6곳 중 1곳만 덮는다. 특히 인덱스 0 placeholder 는
  모든 HWP3 문서에 있어 전수 계약 테스트가 그 자리에서 실패한다.
- **라이터 write-time clamp** — IR ↔ 저장 바이트가 달라져 `ir_field_sweep_baseline` hwpx lane 에
  신규 발산이 생기고, 계약 테스트가 IR 결함을 못 잡게 된다.
  `debug_assert!` 대체도 불가 — `release-test` 는 `inherits = "release"`.
- **읽기 시 0→100 정규화** — HWP5 는 `raw_data` 우선이라 재저장 시 여전히 0 이 나간다.

## 5. 회귀 고정

### 통합 계약 — `tests/issue_4141_hwp3_relative_size_contract.rs` (신규 5건)

한컴이 CI 에 없으므로 **저장 바이트/XML 에서** 검사한다. #3676 파일을 확장하지 않고 새로 만들었다
— 실패 부류(개봉 거부 vs 열리는데 백지), 스트림(BodyText vs DocInfo), 표본 범위가 모두 다르다.

| 테스트 | 축 |
| --- | --- |
| `hwp3_convert_emits_valid_relative_sizes_for_every_sample` | HWP5 DocInfo CHAR_SHAPE 오프셋 28..35, HWP3 표본 **전수** |
| `so_sueop_convert_relative_sizes_are_all_100` | 같음, 재현 표본 이름 고정 |
| `public_document_core_export_also_emits_valid_relative_sizes` | 같음, `export_hwp_with_adapter` 경로 |
| `hwp3_export_hwpx_emits_valid_rel_sz` | HWPX `Contents/header.xml` 의 `<hh:relSz>` |
| `hml_roundtrip_without_relsize_child_emits_valid_relsize` | HML `RELSIZE` — 자식 없는 fixture 왕복 |

HWP3 lane 은 `== 100` **강단언**이다 — `Hwp3CharShape`(`src/parser/hwp3/records.rs:193-203`)에
상대크기 필드가 없으므로 변환본은 예외 없이 스펙 기본값이어야 한다. 범위 검사(10~250)는 별도
헬퍼로 두고 HWPX/HML 축에 쓴다.

전수 스윕에 `assert!(swept >= 10)` 하한을 뒀다 — 표본이 전부 건너뛰어져 조용히 통과하는 것을 막는다.

재사용: `Record::read_all`(`src/parser/record.rs:34`) + `CfbReader::read_doc_info`.
`walk_records` 를 손으로 다시 쓰지 않았다.

### 유닛 3건

- `src/model/style.rs::char_shape_default_matches_spec_only_for_relative_sizes`
- `src/parser/hwpx/header.rs::char_pr_without_rel_sz_child_defaults_to_100_percent`
- `src/parser/hwpx/header.rs::char_pr_id_gap_filler_gets_valid_relative_size`

## 6. 검증

### TDD — 빨강을 먼저 확인했다

수정 없이 5건 전부 실패(`0 passed; 5 failed`), 실패 메시지의 수치가 Stage 1 실측과 일치.
수정 후 `5 passed; 0 failed` (0.61s).

### 게이트 (`local_validation.md` 4.3 — Rust parser/model/CLI)

| 대상 | 결과 |
| --- | --- |
| `issue_4141_hwp3_relative_size_contract` | **5 passed** |
| `issue_3676_hwp3_convert_hancom_openable` | 5 passed |
| `hml_serializer` | 31 passed |
| `hidden_text_contract` | 24 passed |
| `hwp5_roundtrip_baseline` | 3 passed |
| `hwpx_roundtrip_baseline` | 4 passed |
| `--lib model::style` | 6 passed |
| `--lib parser::hwpx::header` | 51 passed |
| `ir_field_sweep_baseline` | 2 passed, **덤프가 baseline 과 SHA-256 일치(무변동)** |
| `cargo fmt --check` | 변경 3파일 위반 해소 |
| `cargo clippy --lib --tests -D warnings` | **exit 0, 경고 0** |

IR sweep 무변동은 계획 단계에서 논증이었고, 덤프를 실제로 떠서 확인했다(줄 수 598=598,
줄바꿈 정규화 후 해시 일치).

renderer lane 은 실행하지 않았다 — `relative_sizes` 는 렌더 경로 참조가 0건이고 렌더러가
소비하는 `ratios`·`base_size` 는 건드리지 않았다.

**전체 `cargo test --profile release-test --tests` 와 `cargo clippy --all-targets` 는
작업지시자 승인 대기다** (`docs_and_git_workflow.md:181-184`).

## 7. 한컴 판정 — 수행 완료 (2026-08-07)

CI 에 한컴이 없으므로 최종 정답지 판정은 수동이다. 번들을 `output/issue_4141/` 에 준비하고
(`.gitignore:15` 로 비커밋) 작업지시자가 한컴으로 열어 PDF 로 출력했다. PyMuPDF 계측:

| | 쪽수 | span | min | max | median | **1pt 미만** |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 원본 `pdf/SO-SUEOP-2024.pdf` | 46 | 14,417 | 7.17 | 50.64 | **9.71** | 0 |
| `before` (수정 전) | 46 | 10,604 | 0.12 | 0.12 | 0.12 | **10,604** |
| `after` (수정 후) | **47** | 7,730 | 6.96 | 50.51 | **9.71** | **0** |

| # | 기준 | 결과 |
| --- | --- | --- |
| A1 | 1pt 미만 span = 0개 | **PASS** — 0개 |
| A2 | 크기 분포 min/max/median ±5% | **PASS** — min −2.9%, max −0.3%, median **0%** |
| A2′ | 최빈 8종 일치 | **FAIL** |
| A3 | 쪽수 46 | **FAIL** — 47쪽 |
| A4 | 1쪽 육안 정상 | **FAIL** — §7.2 |
| A5 | **음성 대조군** — `before` 재현 | **PASS** — 10,604 span 전부 0.12pt, 이슈 수치와 정확히 일치 |

A5 가 정확히 재현됐으므로 판정 절차는 유효하다.

### 7.1 판정 — 이 이슈의 계약은 해소됐다

정의된 결함은 "본문 글자가 전부 ~0.1pt 로 그려진다"이고 그것은 해소됐다:
1pt 미만 span **10,604개 → 0개**, median 이 원본과 소수점까지 일치(9.71).
본문도 온전하다(추출 문자수 원본 50,172자 → `after` 54,219자).

### 7.2 다만 문서는 아직 사용 가능하지 않다 — 별개 결함 둘

A3·A4 실패의 원인은 #4141 과 인과가 없는 결함 둘이다.

**① 글자 음영이 검정 — [#4155](https://github.com/edwardkim/rhwp/issues/4155) (이 판정에서 발견해 등록)**

`after` 는 본문 전체가 검정 막대로 덮인다. 텍스트는 제 위치·제 크기로 그려지고 그 위에 줄 크기
순검정 사각형이 칠해진다. `shade_color` 바이트가 **before/after 동일**(`0x00000000` × 2,512)이므로
#4141 과 인과가 없다 — 글자가 0.12pt 일 때는 음영 사각형도 0×0 으로 찌부러져 보이지 않았을 뿐이다
(`before` 3쪽 검정 fill 3개 크기 0×0 → `after` 65개 줄 크기).

근인은 HWP3 `shade_ratio` 미반영과 HWP5 라이터의 "음영 없음" sentinel 미번역이다.
한컴은 "음영 없음"을 `0xFFFFFFFF` 로 쓴다(`samples/` HWP5 380건에서 22,189건. 검정은 **0건**).
**#4141 merge 이후 별도 작업**한다(작업지시자 판단).

**② 1쪽 글맵시 누락 — [#4097](https://github.com/edwardkim/rhwp/issues/4097) 축**

그 수정은 [PR #4144](https://github.com/edwardkim/rhwp/pull/4144) 에 있고 이 브랜치는
`upstream/devel` 분기라 미포함이다.

**③ A3(쪽수 47 vs 46) 는 원인 미확정.** `before` 는 전 글자가 0.12pt 라 그 46쪽이 레이아웃
기준선이 될 수 없다. #4155·#4097 이 남은 상태에서는 기여를 분리할 수 없으므로 **셋이 해소된 뒤
재측정해야 판단이 선다.** 지금 단정하지 않는다.

### 7.3 번들과 절차 (재현용)

### 인계 전 바이트 사전검증 (완료)

| 파일 | CHAR_SHAPE / relSz | relSz | ratios | base_size 최빈 |
| --- | ---: | --- | --- | --- |
| `before.hwp` | 2,512 | **0 × 2,512** | 100 × 977 | 1000 × 1706 |
| `after.hwp` | 2,512 | **100 × 2,512** | 100 × 977 | 1000 × 1706 |
| `before.hwpx` | 2,512 | **0 × 2,512** | — | — |
| `after.hwpx` | 2,512 | **100 × 2,512** | — | — |

**격리된 변수는 상대크기 하나다.** `.hwpx` 는 12개 ZIP 엔트리 중 `Contents/header.xml` 하나만
다르다. `.hwp` 는 총 크기 99,840 동일에 6,227바이트가 다른데, DocInfo 가 deflate 압축이라
2,512×7바이트 변경이 스트림 전체로 번지기 때문이다.

합격 기준은 **판정 전에 고정**했고(A1~A5, 위 표) A5 를 음성 대조군으로 둬 절차 자체를
검증하게 했다. 절차와 계측 스크립트는 `output/issue_4141/PANJEONG.md` 와 `measure_spans.py` 에 있다.

## 8. 사용자 영향 — 재변환이 필요하다

이 수정은 **재변환해야 적용된다.** rhwp 가 이미 만들어 배포한 relSz=0 파일은 자동 복구되지
않는다. 읽기 시 정규화를 넣지 않은 이유는 §4 의 기각 사유대로 HWP5 가 `raw_data` 를 우선해
재저장 시 여전히 0 이 나가기 때문이다. HWP3 원본에서 다시 변환해야 한다.

## 9. 후속

### 글자 음영 검정 — [#4155](https://github.com/edwardkim/rhwp/issues/4155) (등록 완료)

이 이슈의 한컴 판정에서 발견해 실측 근거와 함께 등록했다(§7.2). **#4141 merge 이후 별도
작업**한다 — 같은 브랜치에 묶으면 한컴 판정 결과의 귀속이 흐려지기 때문이다.

이 문서가 다루는 `relative_sizes` 와 **같은 부류**다: rhwp 가 쓰는 sentinel 을 한컴이 공유하지
않아, 자기정합은 유지되고 한컴만 깨진다. 다만 필드도 근인도 다르다(모델 파생 기본값 vs
`shade_ratio` 미반영 + 라이터 sentinel 미번역).

### `ratios` 기본값 (별도 이슈)

`ratios` 기본값도 `[0;7]` 이고 OWPML `ratio` 는 default=100 / [50,200] 이라 0 은 범위 밖이다.
그런데 **렌더러가 이 값을 읽는다**(`style_resolver.rs:355` → 장평 0 = 글자 폭 0).

- HWP3 축은 안전하다 — `convert_char_shape`(`hwp3/mod.rs:540`)가 레코드에서 채우고, 유일하게
  `ratios=0` 인 인덱스 0 레코드는 참조되지 않는다(전 15표본 0건 계측).
- **HML 축은 실제로 노출된다** — `RELSIZE`/`RATIO` 없는 HML 을 배포 CLI 로 왕복시키면
  `RATIO="0"` 이 나간다(Stage 1 §5 재현). HML 은 렌더 경로이기도 하다.

렌더 가시 변경이라 renderer lane(Native Skia 3종 + wasm-pack + 시각 증적)이 발동하므로
별도 이슈로 분리한다. 근거는 Stage 1 §7 계측표.

### #4097 판정 재개

`pdf/task4097/README.md` 가 "#4141 해소 후 같은 쌍을 다시 만들면 이 축의 **양성 판정**이 비로소
가능해진다"고 적어 뒀다. 이 수정이 그 전제를 푼다 — #4097 의 HWP3 글맵시 축 한컴 판정을
다시 만들 수 있다.

## 10. 커밋

| 커밋 | 내용 |
| --- | --- |
| `252e35224` | 계획서 + Stage 1 실측 (프로덕션 변경 0) |
| `5e0df1871` | `impl Default for CharShape` + 계약 테스트 5건 + 유닛 3건 + 문서 정합 |
| (본 커밋) | Stage 3 — 판정 번들·한컴 판정 결과·최종 보고서 (프로덕션 변경 0) |

프로덕션 코드 변경은 `src/model/style.rs` 한 곳뿐이다. Stage 3 에서 쓴 임시 프로브
(`convert_char_shape` 의 `shade_ratio` 출력)는 측정 후 삭제했고 `git status` 로 확인했다.
