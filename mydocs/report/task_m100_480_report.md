# Task M100 #480 최종 결과 보고서

| 항목 | 내용 |
|------|------|
| 이슈 | [#480](https://github.com/edwardkim/rhwp/issues/480) |
| 마일스톤 | M100 (v1.0.0) |
| 브랜치 | `local/task480` |

## 1. 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 14 단 1의 paragraph 299의 표("[A]" 박스, wrap=Square)가 단 1 본문이 아닌 **단 0과 단 1 사이의 갭 영역**에 그려졌다.

| 항목 | x 좌표 |
|------|--------|
| 단 1 시작 (col_area.x) | 580.16 |
| paragraph 본문 시작 | 602.86 |
| **수정 전 표 x** | 580.16 (단 사이 갭) |
| **수정 후 표 x** | 605.5 (paragraph 영역 + horz_offset) |

## 2. 근본 원인

`src/renderer/layout.rs:2256-2270` Square wrap 표 분기가 `col_area.x`만 사용하고 paragraph `effective_margin`(=margin_left + indent if indent > 0)을 미반영. TAC 표 분기(2270-2298)는 동일 패턴에서 `effective_margin`을 적용했으나 Square wrap 분기는 누락.

## 3. 수정

`layout.rs`의 Square wrap 분기에 `effective_margin` + `margin_right`를 적용:

```rust
} else if !is_tac && tbl_is_square {
    let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
    let area_x = col_area.x + effective_margin;
    let area_w = (col_area.width - effective_margin - margin_right).max(0.0);
    let x = match t.common.horz_align {
        HorzAlign::Right | HorzAlign::Outside => area_x + (area_w - tbl_w).max(0.0),
        HorzAlign::Center => area_x + (area_w - tbl_w).max(0.0) / 2.0,
        _ => area_x,
    };
    Some(x)
}
```

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | Square wrap 표 분기에 effective_margin/margin_right 반영 |

## 5. 검증

### 5-1. 핵심 시각 검증

페이지 14 [A] 박스:
- 수정 전: x=580.9, 단 사이 갭 영역
- 수정 후: x=605.5, paragraph 본문 영역 안 (= col_area.x 580.16 + effective_margin 24.59 + horz_offset 0.76)

### 5-2. 단위/통합 테스트

```
cargo test --release
test result: ok. 1078 passed; 0 failed; (lib)
test result: ok. 14, 25, 1, 6, ... (모든 통합 테스트 통과)
```

### 5-3. 골든 SVG

```
cargo test --release --test svg_snapshot
test result: ok. 6 passed; 0 failed;
```

기존 시각 회귀 없음.

### 5-4. Task #295 회귀 점검

halign=Right/Center 처리도 area_x/area_w 기반으로 동일 동작 보장:
- 이전: `col_area.x + col_area.width - tbl_w`
- 수정: `area_x + (area_w - tbl_w).max(0.0) = (col_area.x + ml) + (col_area.w - ml - mr - tbl_w)`

paragraph margin이 0인 케이스(Task #295의 일반적 시나리오)에서는 동작 동일.

## 6. 영향 범위

| 케이스 | 영향 |
|--------|------|
| Square wrap + paragraph margin/indent ≠ 0 | x가 paragraph 영역으로 이동 (수정 의도) |
| Square wrap + paragraph margin = 0 | 변경 없음 |
| TAC 표 | 분기 미변경 |
| InFrontOfText/BehindText/TopAndBottom | 분기 미변경 |

## 7. 잔여 / 후속 작업

본 수정은 표 x 좌표만 정정. PDF에서 본문 텍스트가 표 양쪽으로 흐르는 wrap 동작이 완전히 같은지는 별도 검증 영역 (현재 SVG에서도 본문이 표 우측에 흐름).

## 8. 요약

- 페이지 14 [A] 박스가 단 사이 갭 → paragraph 본문 영역으로 이동 ✓
- 회귀 없음 (단위 1078건 + 통합 + 스냅샷 6건 모두 통과) ✓
- Task #295의 halign=Right 처리 호환 ✓
