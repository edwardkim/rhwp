# Stage 2 — task_m100_4155 결함 수정과 계약 고정 (L1)

- **이슈**: [#4155](https://github.com/edwardkim/rhwp/issues/4155)
- **계획서**: [`mydocs/plans/task_m100_4155.md`](../plans/task_m100_4155.md)
- **선행**: [stage1](task_m100_4155_stage1.md)
- **브랜치**: `task_m100_4155_hwp3_char_shade`
- **커밋**: `71f607188`(수정) · `604eaf2f8`(계약 테스트)
- **작업 시각**: 2026-08-09 KST

## 1. 수정 내용

### 1.1 `hwp3_char_shade_color` 신설 (`src/parser/hwp3/mod.rs`)

```rust
fn hwp3_char_shade_color(palette_index: u8, shade_ratio: u8) -> Option<crate::model::ColorRef> {
    if shade_ratio == 0 {
        return None;
    }
    let base = hwp3_color_index_to_color_ref(palette_index);
    let ratio = u32::from(shade_ratio.min(100));
    let lerp = |component: u32| (component * ratio + 255 * (100 - ratio)) / 100;
    ...
}
```

흰 바탕에 팔레트 색을 비율만큼 섞는 채널별 lerp 다. 정수 **절하**가 한컴 저장본과 맞는다
([stage1](task_m100_4155_stage1.md) §3). 반환 타입을 `Option` 으로 두어 "음영 없음"이라는
결정을 sentinel 값이 아니라 타입으로 표현하고, sentinel 은 IR 에 쓰는 호출부 한 곳에서만
등장한다.

### 1.2 IR sentinel 통일

- `src/model/color.rs` 신설 — `NONE: ColorRef = 0xFFFF_FFFF` 와 근거 주석.
- `src/model/style.rs` — `impl Default for CharShape` 의 `shade_color: 0` → `color::NONE`.
  #4141 이 `relative_sizes` 를 같은 자리에서 고친 것과 동형이다.
- `src/parser/hml/reader.rs` — `ShadeColor` 속성 부재 시 `unwrap_or(0)` → `unwrap_or(NONE)`.
- `src/document_core/builders/exam_paper.rs` — 명시 `shade_color: 0` 제거(`..Default::default()`
  가 이미 있다).

라이터 3종은 **무수정**이다. `color_hex` 가 이미 `0xFFFFFFFF → "none"` 이고 HWP5 는 통과
저장이면 맞다.

## 2. 기존 테스트 2건의 기대값 갱신

둘 다 옛 계약을 고정하고 있었다. 값만 바꾼 게 아니라 무엇이 왜 바뀌었는지 주석에 남겼다.

| 테스트 | 종전 | 갱신 |
| --- | --- | --- |
| `task2958_convert_char_shape_preserves_shade_color` | 비율 0 으로 팔레트 1(파랑)을 기대 | 비율 100% 에서 팔레트 보존 + 비율 0 은 sentinel |
| `char_shape_default_matches_spec_only_for_relative_sizes` | 어긋나는 필드가 `relative_sizes` 하나 | `shade_color` 포함 둘. 이름도 `..._and_shade` 로 |

#2958 테스트가 고정하던 계약이 곧 결함이었다 — 비율 0 인 팔레트 인덱스를 색으로 읽는 것.
#4141 가드 테스트 주석의 "HML preflight 가 0 에 의존한다"는 단서는 실측과 달랐다
([stage1](task_m100_4155_stage1.md) §6).

## 3. 계약 테스트 — `tests/issue_4155_hwp3_char_shade_contract.rs`

`tests/issue_4141_hwp3_relative_size_contract.rs` 의 헬퍼(`hwp3_samples`,
`char_shape_payloads`, `zip_text_entry`, `convert_to_hwp5_bytes`)를 그대로 옮겨 썼다.

| # | 테스트 | 고정하는 것 |
| --- | --- | --- |
| ① | `hwp3_convert_never_emits_black_char_shade` | HWP3 표본 전수 검정 음영 0건 (스윕 하한 10건) |
| ② | `so_sueop_char_shades_are_all_no_shade` | SO-SUEOP CHAR_SHAPE 전건 `0xFFFFFFFF` |
| ③ | `hwp3_shaded_samples_match_hancom_gray` | 한컴 실측 회색 4케이스 — **반올림 방향** |
| ④ | `public_document_core_export_also_avoids_black_char_shade` | public 저장 경로 |
| ⑤ | `hwp3_export_hwpx_keeps_shade_color_contract` | HWPX `"none"` 전수 + 음영 보존 |
| ⑥ | `hml_roundtrip_without_shadecolor_emits_no_shade_sentinel` | HML `4294967295` |

③ 이 없으면 lerp 를 절상으로 구현해도 ①②④ 는 전부 통과한다 — 검정만 아니면 되기 때문이다.
③ 은 `0xd8d8d8`·`0xefefef`·`0x999999` 를 이름으로 요구하므로 절상 구현에서 red 가 된다.

## 4. 검증

### 4.1 변이 검증 (red 기준선)

수정 커밋을 되돌리고 실행: **6/6 red**. 실패 분포가 이슈 본문과 일치했다
([stage1](task_m100_4155_stage1.md) §2·§4). 특히 ⑤ 는 SO-SUEOP `"none"` 전수 절이 수정 전에도
통과하고 sample11 음영 보존 절에서만 실패해, 이슈가 말한 "HWPX 축은 정상"이 확인됐다.

### 4.2 green

| 명령 | 결과 |
| --- | --- |
| `cargo test --profile release-test --lib` | **3,377 passed / 0 failed** |
| `--test issue_4155_hwp3_char_shade_contract` | **6 passed / 0 failed** |
| `--test hidden_text_contract` | 24 passed |
| `--test issue_4141_hwp3_relative_size_contract` | 37 passed |
| `--test hml_serializer` · `--test hml_parser` | 31 · 50 passed |
| `--test hwpx_to_hwp_adapter` | 5 passed |

`hidden_text_contract` 무회귀가 특히 중요하다 — 그 테스트의 `CLEAN_SAMPLES` 는 HWP3 표본이
다수이고, `shade_color` 를 음영으로 오독하면 31,907건 오탐이 되돌아온다. `opaque_rgb` 가
상위 바이트로 먼저 거르므로 `0xFFFFFFFF` 도 "색 없음"으로 받는다.
