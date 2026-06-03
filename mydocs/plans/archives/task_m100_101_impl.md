# Task #101: PartialTable LAYOUT_OVERFLOW — 구현계획서 (v4)

> **이슈**: [#101](https://github.com/edwardkim/rhwp/issues/101)
> **브랜치**: `local/task101`
> **작성일**: 2026-04-10
> **수정일**: 2026-04-11 (v4 — 30페이지 pi=181 근본 원인 확정)

---

## 현황

### 해결된 케이스

- **19페이지 pi=78**: `split_table_rows`에서 `avail_for_rows -= spacing_before_px` 적용으로 해소 ✅
  - `rows=0..20` → `rows=0..19`, LAYOUT_OVERFLOW 제거 확인

### 미해결 케이스

- **30페이지 pi=181**: LAYOUT_OVERFLOW 28.2px 잔존 ❌
  - `PartialTable pi=181 ci=0 rows=0..13 cont=false 33x4`
  - spacing_before_px 차감만으로는 해결 불가

---

## 30페이지 pi=181 근본 원인 (v4 확정)

### 원인: 비-TAC 표 캡션 높이가 `current_height`에 누락

`pagination`에서 비-TAC 표의 `table_total_height`:
```
table_total_height = effective_height + host_spacing
effective_height = mt.total_height - cap_h - cap_s   ← 캡션 제외
```

그러나 `place_table_fits`에서 `current_height += table_total_height`로 누적 시 캡션이 빠집니다.

`layout_table`은 `table_bottom = table_y + table_height + caption_extra`를 반환하므로
레이아웃의 y_offset 전진에는 캡션이 포함됩니다.

### 수치 근거 (디버그 출력으로 확인)

```
pi=179 (17×3 표): mt.total_height=291.91 / mt.rh_sum=271.69 / mt.cap_h=12.67
  → 캡션 + spacing = 291.91 - 271.69 = 20.22px 누락
  → layout y 전진: 306.57px / pagination 전진: 286.36px → 차이 20.21px

pi=180 (8×2 표): mt.total_height=153.88 / mt.rh_sum=129.85 / mt.cap_h=12.67
  → 캡션 + spacing = 153.88 - 129.85 = 24.03px 누락
  → layout y 전진: 168.55px / pagination 전진: 144.52px → 차이 24.03px

pi=179 시작 전 역방향 오차: -7.55px (pagination이 layout보다 큰 경우)
누적 discrepancy: 20.22 + 24.03 - 7.55 = 36.70px ≈ 36.65px (측정값)
```

이 discrepancy로 인해 pi=181의 `page_avail`이 실제보다 크게 계산되어 rows=0..13 선택,
레이아웃에서 overflow 발생.

### 실패한 수정 시도 (참고)

시도 1: `table_total_height`에 캡션 추가 (피트 판단 포함)
- 피트 판단이 더 엄격해져 기존 표들이 다음 페이지로 밀림 → 다수 회귀

---

## 수정 전략 (v4)

### 핵심 원칙

**`current_height`에 캡션을 추가하되, 피트 판단에서는 제외한다.**

`table_total_height`는 현재 `current_height` 누적에 사용됩니다.
캡션 포함 여부를 분리하려면:

1. `table_total_height` = `effective_height + host_spacing` (변경 없음 — 피트 판단 기준)
2. 별도 `caption_extra_for_current` = 비-TAC Top/Bottom 캡션 높이 + spacing
3. `place_table_fits` 내에서 `current_height += table_total_height + caption_extra_for_current`

### 구체적 수정 방법

`paginate_table_control`에서 `caption_extra_for_current` 계산:
```rust
let caption_extra_for_current = if !is_tac_table {
    if let Some(mt) = measured_table {
        if mt.caption_height > 0.0 {
            let is_lr = table.caption.as_ref().map_or(false, |c| {
                matches!(c.direction, CaptionDirection::Left | CaptionDirection::Right)
            });
            if !is_lr {
                let cap_s = table.caption.as_ref()
                    .map(|c| hwpunit_to_px(c.spacing, dpi))
                    .unwrap_or(0.0);
                mt.caption_height + cap_s
            } else { 0.0 }
        } else { 0.0 }
    } else { 0.0 }
} else { 0.0 };
```

`place_table_fits` 시그니처에 `caption_extra_for_current: f64` 추가:
```rust
fn place_table_fits(..., caption_extra_for_current: f64) { ... }
```

`place_table_fits` 내 `current_height` 업데이트:
```rust
// 기존 (비-독립 플로트 분기):
st.current_height += table_total_height;

// 수정:
st.current_height += table_total_height + caption_extra_for_current;
```

### 피트 판단에서 캡션 제외

`effective_table_height`는 `table_total_height` 기반이므로 캡션이 없습니다 (변경 없음).
`place_table_fits` 호출 조건도 `effective_table_height` 기반이므로 피트 판단은 기존 그대로입니다.

### 영향 범위

- `paginate_table_control`: `caption_extra_for_current` 계산 추가
- `place_table_fits`: 시그니처 파라미터 1개 추가, `current_height` 업데이트 수정
- 피트 판단(`effective_table_height`) 변경 없음
- `split_table_rows` 직접 변경 없음 (결과적으로 `page_avail`이 정확해짐)
- 다른 모든 경로 무변경

### is_independent_float 분기 처리

`place_table_fits`의 `is_independent_float` 분기에서는:
```rust
if float_bottom > st.current_height {
    st.current_height = float_bottom;
}
```
이 경우에는 `caption_extra_for_current`를 적용하지 않습니다.
(`float_bottom = para_start_height + v_off + effective_height`, 캡션이 없는 vert_offset 표)

---

## 단계별 구현

### 1단계 (완료): spacing_before_px 차감 ✅

이미 적용 완료.

### 2단계: 비-TAC 표 캡션 `current_height` 보정

**파일**: `src/renderer/pagination/engine.rs`

1. `paginate_table_control`에서 `caption_extra_for_current` 계산 (비-TAC Top/Bottom 표만)
2. `place_table_fits` 시그니처에 파라미터 추가
3. `place_table_fits` 내 비-독립-플로트 분기 `current_height += table_total_height + caption_extra_for_current`
4. 호출부 2곳(`place_table_fits` 호출) 파라미터 전달

**검증**:
- `dump-pages -p 29`: `PartialTable pi=181 rows=0..12` (또는 overflow 없이 Table로 배치)
- `export-svg -p 29`: LAYOUT_OVERFLOW 제거 확인
- LAYOUT_OVERFLOW 총 건수 43 → 42 이하

### 3단계: 회귀 테스트

- 226개 샘플 전체 SVG 내보내기 (기존 43개 overflow → 42 이하, 새 항목 없음)

---

## 제약 조건

- 수정 범위 최소화 (engine.rs `place_table_fits` 관련만)
- 피트 판단 로직 변경 금지 (기존 표 배치 유지)
- 전역 적용 금지
- 226개 샘플에서 새 회귀 0건 유지
