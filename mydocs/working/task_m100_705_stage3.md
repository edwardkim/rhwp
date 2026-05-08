# Task #705 Stage 3 — GREEN 결함 #2 정정 (layout.rs 가드)

## 산출물

- `src/renderer/layout.rs` — `build_page_background()` + `build_page_borders()` 호출에 `hide_fill` + `hide_border` 가드 추가 (4 → 14 line)
- `mydocs/working/task_m100_705_stage3.md` (본 보고서)

## 코드 변경 (layout.rs:411-414 → :411-422)

```rust
// 페이지 배경 (감추기 설정 시 건너뜀)
let hide_fill = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_fill).unwrap_or(false);
if !hide_fill {
    self.build_page_background(&mut tree, layout, page_border_fill, styles, bin_data_content);
}

// 쪽 테두리선 (감추기 설정 시 건너뜀)
let hide_border = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_border).unwrap_or(false);
if !hide_border {
    self.build_page_borders(&mut tree, layout, page_border_fill, styles);
}
```

기존 `hide_master`/`hide_header` 패턴 (layout.rs:417-431) 과 동일.

## 회귀 sweep

```
cargo test --release --lib
test result: ok. 1123 passed; 0 failed; 1 ignored
```

→ **0 fail**. 기존 테스트 영향 없음 — `page.page_hide.hide_fill/hide_border` 가 `true` 인 페이지는 Stage 0 측정 결과 aift.hwp page 2 (셀[167] 6 필드 모두 true) 1건만, 다른 페이지는 가드 진입 안 함.

## SVG smoke check (aift.hwp page 2 vs page 1)

`rhwp export-svg samples/aift.hwp -p 0/1` 실행 후 첫 `<rect>` 노드 비교:

### page 1 (정상 — 표시 페이지)

```
<rect x="75.58" y="75.57" width="646.47" height="971.36"/>     # body area
<rect x="75.58" y="79.35" width="634.99" height="205.76"/>     # header
<rect x="0"     y="0"     width="793.71" height="1122.51" fill="#ffffff"/>  # 페이지 배경
```

→ **`fill="#ffffff"` 페이지 배경 rect 존재** (`build_page_background` 호출됨)

### page 2 (감추기 6 필드 모두 true)

```
<rect x="75.58" y="75.57" width="939.76" height="971.36"/>      # body area (landscape)
<rect x="75.58" y="75.57" width="168.34" height="45.29"/>       # 첫 셀
<rect x="243.93" y="75.57" width="166.13" height="45.29"/>      # 두 번째 셀
```

→ **페이지 배경 rect 없음** (`hide_fill` 가드로 `build_page_background` 건너뜀) ✓

→ Stage 3 가드 본질 정확 동작 ✓

## 종합 영향 (Stage 2 + Stage 3)

aift.hwp page 2 의 6 필드 PageHide 적용 결과:

| 필드 | Stage 2 정정 전 | Stage 2 정정 후 | Stage 3 추가 정정 후 |
|------|----------------|----------------|---------------------|
| `hide_page_num` | 미적용 (표시) | **적용** (미표시) ✓ | (동일) |
| `hide_header` | 미적용 | **적용** (이미 layout.rs:427 가드 있음) ✓ | (동일) |
| `hide_footer` | 미적용 | (header 와 함께 처리) ✓ | (동일) |
| `hide_master_page` | 미적용 | **적용** (layout.rs:417 가드) ✓ | (동일) |
| `hide_border` | 미적용 | (page_hide 채워짐, 가드 X) ✗ | **적용** ✓ |
| `hide_fill` | 미적용 | (page_hide 채워짐, 가드 X) ✗ | **적용** ✓ |

Stage 2 + Stage 3 합으로 **6 필드 모두 한컴 호환 정합**.

## 다른 영향 샘플 예상 (Stage 0 측정)

aift.hwp 외 page_num 만 true 케이스 — fill/border 무관, hide_page_num 만 적용:

| 샘플 | 영향 |
|------|------|
| 2022년 국립국어원 업무계획.hwp | 목차 페이지 page_num 미표시 (Stage 2 효과) |
| KTX.hwp | 동일 |
| kps-ai.hwp | 동일 |
| tac-img-02.hwp/.hwpx | 4 페이지 page_num/header 등 적용 |

Stage 5 의 174 sample sweep 으로 회귀 검증.

## 결함 #3 (dump 인프라) 미정정

`src/main.rs:1644-1667` 의 dump 셀 안 controls 매칭에 `Control::PageHide` 분기 없음 — 디버깅 한계. Stage 4 에서 정정.

## Stage 4 진입 결정

**Stage 4 (dump 인프라 정정 — main.rs)** 진입 가능:

1. `src/main.rs:1644-1667` 의 `match ctrl { Control::Picture(p) ... Control::Shape(s) ... _ => {} }` 에 `Control::PageHide(ph)` 분기 추가
2. 검증: `rhwp dump samples/aift.hwp -s 0 -p 1` → 셀[167]/p[3] PageHide 6 필드 출력 확인

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Stage 0 보고서: `mydocs/working/task_m100_705_stage0.md`
- Stage 1 보고서: `mydocs/working/task_m100_705_stage1.md`
- Stage 2 보고서: `mydocs/working/task_m100_705_stage2.md`
- 본 보고서: `mydocs/working/task_m100_705_stage3.md`
