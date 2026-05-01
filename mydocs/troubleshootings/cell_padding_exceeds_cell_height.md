# 셀 padding 이 cell.height 를 초과하는 비정상 IR 처리

## 날짜

2026-05-01

## 증상

`samples/mel-001.hwp` 2쪽 8x12 인원현황 표 (s0:pi=22) 에서:

- **셀[21] r=2 c=2 "현 원"**: 텍스트가 표시되지 않음 + rhwp-studio 에서 셀 클릭/키보드 진입 안됨
- **합계 행 (rs=2 셀[10]) 의 r=1/r=2 분배 비균등**: r=1 정원=27.45px, r=2 현원=7.99px (정상 17.07/17.07)
- **인덱스 행 (r=0)**: 정상 26.4px → 회귀 12.36~20.35px (50~77%)
- **rhwp v0.7.8 + devel HEAD 모두 결함**, 크롬 확장 v0.2.1 정상

## IR 정합 — 비정상 padding

```
셀[21] r=2,c=2 rs=1,cs=1 h=1280 w=4935 pad=(141,141,1700,1700) aim=false bf=16 paras=1 text="현 원"
셀[43] r=4,c=2 rs=1,cs=1 h=1280 w=4935 pad=(141,141,1700,1700) aim=false bf=16 paras=1 text="현 원"
셀[65] r=6,c=2 rs=1,cs=1 h=1300 w=4935 pad=(141,141,1700,1700) aim=false bf=23 paras=1 text="현 원"
```

- `cell.height = 1280 HU` = 17.07px (1 inch = 7200 HU = 96px)
- `cell.padding.top = 1700 HU` = 22.67px
- `cell.padding.bottom = 1700 HU` = 22.67px
- **pad_top + pad_bottom = 45.33px > cell.height (17.07px)** ★

→ 셀 안 padding 합산이 셀 자체 높이의 **2.66배**.

### HWPX 정합 (samples/hwpx/mel-001.hwpx)

```xml
<hp:tc ... hasMargin="0" ... borderFillIDRef="16">
  <hp:cellSz width="4935" height="1280"/>
  <hp:cellMargin left="141" right="141" top="1700" bottom="1700"/>
</hp:tc>
```

`hasMargin="0"` (= `apply_inner_margin=false`) — HWP 스펙상 **셀 고유 여백 미사용 (표 기본 여백 사용)**.

## 회귀 origin

`src/renderer/layout/table_layout.rs::resolve_cell_padding` (Task #347 추가 가드):

```rust
let prefer_cell_axis = |c: i16, t: i16| -> bool {
    if cell.apply_inner_margin {
        c != 0
    } else {
        // aim=false: cell이 table보다 명백히 큰 경우만 cell 우선 (의도된 비대칭)
        (c as i32) > (t as i32)
    }
};
```

Task #347 가 **KTX 목차 R=1417 HU** 같은 작성자 의도된 비대칭 padding 영역 정정 위해 도입한 가드. **`aim=false 에서도 cell.padding > table.padding 면 cell 우선`** 정책.

→ mel-001 p2 셀[21] 의 1700 HU 도 cell 우선으로 적용 → **셀 안의 padding 합산이 cell.height 자체를 초과** → 회귀:

1. `measure_table_impl` 의 `required_height = content + pad_top + pad_bottom` 가 거대 (66+ px) → row_heights[2] = 66+
2. raw_table_height = 합산 ≈ 327px > common.height (146.13px)
3. TAC 표 비례 축소 (`scale = 146/327 = 0.45`) 적용 → 모든 행 축소 (r=0=26.4 → 12.36)
4. cell rect 좁아짐 → inner_height = 0 → paragraph_layout 의 텍스트 발행 차단

## 한컴의 방어 로직

작업지시자 통찰: **한컴은 비정상 IR (padding > cell.h) 을 자체 방어 로직으로 처리**.

한컴의 시각 출력 정합:
- 셀[21] "현 원" 텍스트 정상 표시 (셀 영역 안)
- 행 분배 균등 (r=1=17.07 / r=2=17.07)
- 표 전체 높이 cell.height 합산 정합

→ 한컴은 padding 이 cell.height 를 초과하면 **padding 을 cell.height 안에 들어가도록 자체 비례 축소** 적용 (또는 padding 을 무시).

## 정정

### 정책 — 한컴 방어 로직 모방

`resolve_cell_padding` 의 끝에 다음 가드 추가 — pad_top + pad_bottom 이 cell.height 를 초과하면 cell.height 의 절반까지로 비례 축소:

```rust
// [Task #501] 한컴 방어 로직 모방 — cell.padding.top + bottom 합산이
// cell.height 자체를 초과하면 (mel-001 p2 셀[21]: pad=1700 HU 두 축, h=1280 HU)
// 한컴은 자체 가드로 cell 안에 콘텐츠가 들어가도록 처리. cell.height 의 절반까지
// 비례 축소 (HWP 스펙 외 한컴 동작 모방).
let (pad_top, pad_bottom) = if cell.height < 0x80000000 {
    let cell_h_px = hwpunit_to_px(cell.height as i32, self.dpi);
    let total_v_pad = pad_top + pad_bottom;
    if cell_h_px > 0.0 && total_v_pad >= cell_h_px {
        let max_v_pad = cell_h_px * 0.5;
        let scale = max_v_pad / total_v_pad;
        (pad_top * scale, pad_bottom * scale)
    } else {
        (pad_top, pad_bottom)
    }
} else {
    (pad_top, pad_bottom)
};
```

### measure_table_impl 보강 (안전망)

`src/renderer/height_measurer.rs::measure_table_impl` 의 1-b단계 (content_height + pad → required_height) 에서도 동일 가드 적용. 셀 padding 이 cell.height 를 초과하면 IR cell.height 권위 우선:

```rust
// [Task #501] cell.padding 이 IR cell.height 의 절반을 초과하는 비정상 케이스 가드
let total_pad = pad_top + pad_bottom;
let cell_h_px = if cell.height < 0x80000000 {
    hwpunit_to_px(cell.height as i32, self.dpi)
} else { 0.0 };
let required_height = if cell_h_px > 0.0
    && total_pad > cell_h_px * 0.5
    && content_height <= cell_h_px
{
    cell_h_px
} else {
    content_height + total_pad
};
```

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | resolve_cell_padding 끝에 한컴 방어 로직 가드 추가 |
| `src/renderer/height_measurer.rs` | measure_table_impl 1-b단계 가드 추가 (안전망) |
| `tests/issue_501.rs` | 신규 통합 테스트 (mel-001 p2 pi=22 행 0 IR cell.h 정합) |

## 검증

| 검증 | 결과 |
|------|------|
| `cargo test --test issue_501` | PASS (Red → Green) |
| `cargo test --lib` | 1086 passed |
| `cargo test --test svg_snapshot` | 5/6 (form_002 부동소수 정밀도 미세 차이 — 시각 영향 없음) |
| `cargo test --test issue_418` | 1/1 |
| `cargo clippy --lib -- -D warnings` | 0건 |
| WASM 빌드 | 4,206,487 bytes |
| 작업지시자 시각 검증 | mel-001 p2 pi=22 정합 ★ |

## 영향 영역

### 정정 효과 (정합)

- mel-001 p2 pi=22 8x12 인원현황 표:
  - 행 0 (헤더 인덱스): 12.36 → 26.4 (정상)
  - 합계 r=1 정원/r=2 현원: 27.45/7.99 → 17.07/17.07 (균등)
  - 셀[21] r=2 c=2 "현 원" 텍스트 정상 표시
  - rhwp-studio 셀 진입 정상

### 회귀 점검 (다른 샘플)

| 샘플 | 영향 |
|------|------|
| KTX (R=1417 HU 비대칭 padding) | 영향 없음 (1417 HU < cell.height, 가드 미발동) |
| 일반 표 (pad < cell.h) | 영향 없음 (가드 미발동) |
| TAC 표 (treat_as_char=true) 다수 | 영향 없음 |
| form_002 | 부동소수 정밀도 미세 차이만 (시각 동일) |

## 메모리 룰 정합

- `feedback_hancom_compat_specific_over_general` — 일반화 휴리스틱이 아닌 **구조 가드** (pad > cell.h 케이스) 적용
- `feedback_pdf_not_authoritative` — 한컴 시각 출력을 정답지로 직접 사용하지 않고 **한컴 방어 로직 모방** 정책으로 일관성 확보

## 관련 task

- Task #279: 셀 padding 정책 초기 (전 축에서 cell 우선)
- Task #347: aim=false 에서 cell > table 비대칭 가드 (KTX 목차 정합)
- **Task #501**: 본 트러블슈팅 — pad > cell.h 비정상 IR 의 한컴 방어 로직 모방

## 후속 작업 가능성

- `1700 HU` 의 비정상 IR 이 한컴 편집기에서 어떻게 입력되는지 (UI 경로) 조사
- HWP 스펙의 `cellMargin` 상한 (cell.height 와의 관계) 명시 영역 점검
