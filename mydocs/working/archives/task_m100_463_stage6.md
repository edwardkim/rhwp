# Task #463 Stage 6 완료보고서

## 발견 (Stage 5 머지 후 사용자 검토)

작업지시자가 14페이지 시각 검토에서 발견:
> "학생3 이 학생1 보다 들여쓰여짐"

## 원인

`layout_wrap_around_paras` 가 `wrap_area` 를 `wrap_text_x = col_area.x + LINE_SEG.column_start` 로 설정. `column_start` 는 paragraph margin_left 를 이미 포함한 값(예: 850 HU = 11.33 px = ps_id=40 margin_left).

이후 `layout_composed_paragraph` 가 col_area (= wrap_area) 에 `margin_left + inner_pad` 를 추가로 더해 텍스트 x 를 계산:

| paragraph | text x 계산 | 결과 |
|-----------|-------------|------|
| 학생1 (pi=30, regular) | `col_area.x + margin_left + inner_pad` = 117.2 + 11.33 + 11.33 | **139.86** |
| 학생3 (pi=33, wrap host) | `wrap_area.x + margin_left + inner_pad` = (117.2+11.33) + 11.33 + 11.33 | **151.19** |

→ wrap host 가 `margin_left = 11.33 px` 만큼 우측으로 더 밀림 (margin_left 이중 적용).

## 수정

`layout_wrap_around_paras` 의 `wrap_area` 구성 시 host paragraph 의 `margin_left` 를 좌측으로 보정 (width 도 그만큼 확장):

```rust
let host_para_style = composed.get(table_para_index)
    .and_then(|c| styles.para_styles.get(c.para_style_id as usize));
let host_margin_left = host_para_style.map(|s| s.margin_left).unwrap_or(0.0);
let host_margin_right = host_para_style.map(|s| s.margin_right).unwrap_or(0.0);
let wrap_area = LayoutRect {
    x: wrap_text_x - host_margin_left,
    y: col_area.y,
    width: wrap_text_width + host_margin_left + host_margin_right,
    height: col_area.height,
};
```

이렇게 하면:
- text x = `wrap_area.x + margin_left + inner_pad` = `(wrap_text_x - margin_left) + margin_left + inner_pad` = `wrap_text_x + inner_pad`
- 정상 paragraph text x = `col_area.x + margin_left + inner_pad` = `col_area.x + 11.33 + 11.33` = `col_area.x + 22.66`
- wrap_text_x = col_area.x + wrap_cs = col_area.x + 11.33
- text x (wrap host) = wrap_text_x + inner_pad = col_area.x + 11.33 + 11.33 = col_area.x + 22.66 ✓

> 주의: `inner_pad` 는 paragraph border 안쪽 여백(stroke 가 있을 때 적용)으로 wrap_cs 와 무관 — 보정 대상 아님.

## 검증

### 14p 좌측 단 학생 라벨 x 좌표

| paragraph | y | x 좌표 (Stage 5) | x 좌표 (Stage 6) |
|-----------|---|------------------|------------------|
| pi=30 학생1 | 316.53 | 139.84 | 139.84 |
| pi=31 학생2 | 365.55 | 139.84 | 139.84 |
| pi=32 학생1 | 390.05 | 139.84 | 139.84 |
| pi=33 학생3 (wrap host) | 463.57 | **151.17** | **139.84** ✓ |
| pi=34 학생2 | 548.51 | 139.84 | 139.84 |
| pi=35 학생3 | 597.52 | 139.84 | 139.84 |
| ... | ... | ... | ... |

전체 학생 라벨이 동일 x=139.84 로 정렬. wrap host 들여쓰기 차이 해소.

### 박스 우측 (영향 없음 확인)

좌측 단 박스: x=128.5 w=421.39 → right=549.89 (변동 없음, [A][B][C] 마커 박스 안 유지)

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed
```

### 회귀 (3종 샘플)

- exam_kor.hwp (20p) ✓
- 21_언어_기출_편집가능본.hwp (15p) ✓
- exam_eng.hwp (8p) ✓

## 변경 파일

| 파일 | 변경 라인 |
|------|----------|
| `src/renderer/layout.rs` | +9 (wrap_area 보정) |
