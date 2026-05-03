# Task #533 Stage 3 — 코드 적용 + 단위 테스트

**작성일**: 2026-05-02
**이슈**: [#533](https://github.com/edwardkim/rhwp/issues/533)
**브랜치**: `local/task533`

## 1. 결론

> Stage 2 구현계획서 기반 변경 적용. **본 결함 시각 검증 통과** (pi=51 SVG y 285.44 → 298.21, gap 11.73 → 24.51), **lib 1116 / svg_snapshot 6/6 / issue_418/501 회귀 0**. Stage 2 초안 발견 결함 (vpos 가 column 누적 좌표) 즉시 정정 후 Stage 4 광범위 회귀 검증 단계 진입.

## 2. 변경 위치

`src/renderer/layout.rs::layout_table_item` line 2509-2532 (line_segs.last() 가산 직전, +18 라인).

## 3. 1차 시도 결함 발견 + 정정

### 3-1. 1차 코드

```rust
let host_text_bottom = para_y_for_table + hwpunit_to_px(
    seg.vertical_pos + seg.line_height, self.dpi);
```

### 3-2. 결함

좌측 단 회귀 발생 (pi=33 이후 263 px 비정상 advance, pi=35 이후 누적). 원인:

- `seg.vertical_pos` 는 **column 누적 좌표** (HWP IR), paragraph 상대 좌표 아님
- 예: pi=33 ls[0].vpos = 17916, ls[3].vpos = 23430 — column 시작 기준 cumulative
- pi=50 의 경우만 column 1 최상단이라 vpos=0 시작 — 우연한 일치
- para_y_for_table + (cumulative vpos + lh) 는 mid-column paragraph 에 잘못된 advance

### 3-3. 정정

```rust
if let (Some(first), Some(last)) = (para.line_segs.first(), para.line_segs.last()) {
    let para_inner_h = (last.vertical_pos + last.line_height)
        .saturating_sub(first.vertical_pos);
    let host_text_bottom = para_y_for_table + hwpunit_to_px(para_inner_h, self.dpi);
    if host_text_bottom > y_offset {
        y_offset = host_text_bottom;
    }
}
```

`first.vertical_pos` 차감 → paragraph 내부 offset 으로 변환.

| 케이스 | first.vpos | last.vpos+lh | inner_h | 정합 |
|--------|-----------|--------------|---------|------|
| pi=50 (column 1 top) | 0 | 4826 | 4826 = 64.35 px | ✓ |
| pi=33 (mid column 0) | 17916 | 24580 | 6664 = 88.85 px | ✓ |
| pi=37 | 34458 | 38134+1150=39284 | 4826 = 64.35 px | ✓ |
| pi=40 | 51000 | 52838+1150=53988 | 2988 = 39.84 px | ✓ |
| pi=47 | 78837 | 82513+1150=83663 | 4826 = 64.35 px | ✓ |

## 4. 시각 검증

### 4-1. 우측 단 (column 1)

```bash
target/release/rhwp export-svg samples/exam_kor.hwp -p 13 -o /tmp/p14_fix2
```

| line | y (수정 전) | y (수정 후) | gap (이전) | gap (수정 후) |
|------|-----------|-----------|------------|--------------|
| pi=50 line 0 | 224.69 | 224.69 | — | — |
| pi=50 line 1 | 249.20 | 249.20 | 24.51 ✓ | 24.51 ✓ |
| pi=50 line 2 | 273.71 | 273.71 | 24.51 ✓ | 24.51 ✓ |
| **pi=51 line 0** | **285.44** | **298.21** | **11.73 ★** | **24.51 ✓** |
| pi=51 line 1 | 309.95 | 322.72 | 24.51 | 24.51 ✓ |
| pi=53 (after empty pi=52) | 358.96 | 371.73 | (×2) | 49.01 = 2×24.51 ✓ |

### 4-2. 좌측 단 (column 0) — 전 영역 회귀 차단 확인

| 위치 | gap (수정 전) | gap (수정 후) |
|------|--------------|--------------|
| pi=33 → pi=34 | (정상 24.51) | 24.51 ✓ |
| pi=37 → pi=38 | **11.41 ★** | **24.51 ✓** |
| pi=38 → pi=39 | 24.51 | 24.51 ✓ |
| pi=39 → pi=40 | 24.51 | 24.51 ✓ |
| pi=40 → pi=41 | **11.73 ★** | **24.51 ✓** |
| pi=46 → pi=47 | 24.51 | 24.51 ✓ |
| pi=47 → pi=48 | **11.31 ★** | **24.51 ✓** |

→ pi=37/40/47 직후 모두 정상 24.51 px 회복.

### 4-3. VPOS_CORR 측정 비교

수정 전:
```
VPOS_CORR: pi=51 vpos_end=5514 base=958 col_y=211.65 y_in=272.40 end_y=272.40
```

수정 후:
```
VPOS_CORR: pi=51 vpos_end=5514 base=0 col_y=211.65 y_in=285.17 end_y=285.17
```

→ `base` 가 958 → 0 으로 정상화. `y_in` 이 272.40 → 285.17 로 13 px 보정. SVG y = 285.17 + bl 13.04 = 298.21 ✓

## 5. 단위/통합 테스트 게이트

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed**, 0 failed, 1 ignored |
| `cargo test --test svg_snapshot` | **6/6** 통과 |
| `cargo test --test issue_418` | 1/1 통과 |
| `cargo test --test issue_501` | 1/1 통과 |
| `cargo clippy --lib` | 2 pre-existing errors (`object_ops.rs` / `table_ops.rs` `pic.caption.unwrap()`), 본 task 미관여 |

## 6. 변경 요약

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | layout_table_item Square wrap 분기 (+18 라인). para_y_for_table + (last.vpos + lh - first.vpos) 로 호스트 텍스트 영역 max advance |

## 7. 다음 단계

작업지시자 승인 후 Stage 4 (광범위 샘플 회귀 검증).

## 8. 승인 게이트

- [x] 본 결함 fix 시각 검증 (pi=51 + 좌측 단 pi=37/40/47)
- [x] 1차 시도 결함 발견 + 정정 (column 누적 vpos 처리)
- [x] lib / svg_snapshot / issue_418 / issue_501 회귀 0
- [x] clippy 신규 warning 0 (pre-existing 2 건은 본 task 미관여)
- [x] 호스트 텍스트 영역 < 표 영역 케이스 max() 가드 정상 동작 (대형 표 회귀 차단)
