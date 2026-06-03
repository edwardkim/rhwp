# Task #101: 최종 완료보고서 — PartialTable LAYOUT_OVERFLOW 수정

> **이슈**: [#101](https://github.com/edwardkim/rhwp/issues/101)
> **브랜치**: `local/task101`
> **작성일**: 2026-04-11
> **마일스톤**: M100 (v1.0.0)

---

## 요약

`hwpspec.hwp`에서 발생하던 LAYOUT_OVERFLOW 43건을 4건으로 줄였다 (−39건).
3단계에 걸쳐 근본 원인을 확정하고 최소 범위 수정으로 해결했다.
226개 샘플 전체에서 새 회귀 0건.

---

## 단계별 수행 내용

### 1단계: `split_table_rows` spacing_before 차감

**파일**: `src/renderer/pagination/engine.rs`, `src/renderer/layout/height_measurer.rs`

- `height_measurer.rs`: `find_break_row`에서 SPLIT_EPSILON=0.5 제거 (원인 아님)
- `engine.rs`: `split_table_rows`에 `spacing_before_px` 파라미터 추가
  - 첫 분할 시(`!is_continuation && cursor_row==0 && content_offset==0.0`) `avail_for_rows`에서 차감
  - 결과: pi=78 (19페이지) `rows=0..20 → rows=0..19`, LAYOUT_OVERFLOW 제거

**결과**: 43건 → 42건 (−1건)

---

### 2단계: 비-TAC 표 캡션 `current_height` 보정

**파일**: `src/renderer/pagination/engine.rs`

**근본 원인**: 비-TAC 표의 `table_total_height = effective_height + host_spacing`에 캡션 높이가 포함되지 않아
`current_height`가 레이아웃 실제 y_offset보다 작게 누적.

- pi=179: 캡션 누락 20.22px
- pi=180: 캡션 누락 24.03px
- 누적 discrepancy ~36.65px → `page_avail` 과도 계산 → pi=181 rows=0..13 선택 → overflow 28.2px

**수정 내용**:
1. `paginate_table_control`: `caption_extra_for_current` 계산 추가 (비-TAC Top/Bottom 캡션)
2. `place_table_fits`: 파라미터 추가, `current_height += table_total_height + caption_extra_for_current`
3. 피트 판단(`effective_table_height`) 변경 없음

**결과**: 42건 → 20건 (−22건)

---

### 3단계: TAC 표 캡션 `current_height` 보정

**파일**: `src/renderer/pagination/engine.rs`

**근본 원인**: pi=80~85 (TAC 표, 4×4) 6개에 공(空) 캡션이 존재.

- `mt.caption_height = 12.67px`, `caption_spacing = 11.36px`, `caption_extra = 24.03px`

2단계에서 `caption_extra_for_current`를 비-TAC 표에만 적용했기 때문에 TAC 표의 캡션이 누락.
테이블 1개당 discrepancy 15.50px, 6개 누적 ~93px → pi=85 overflow 39.8px.

**수정 내용**: `caption_extra_for_current` 계산에서 `!is_tac_table` 조건 제거

```rust
// 수정 전: TAC 표 제외
let caption_extra_for_current = if !is_tac_table {
    if let Some(mt) = measured_table { ... } else { 0.0 }
} else { 0.0 };

// 수정 후: TAC 및 비-TAC 모두 적용
let caption_extra_for_current = if let Some(mt) = measured_table {
    if mt.caption_height > 0.0 {
        let is_lr = ...; // Left/Right 캡션 제외
        if !is_lr { mt.caption_height + cap_s } else { 0.0 }
    } else { 0.0 }
} else { 0.0 };
```

**결과**: 20건 → 4건 (−16건)

---

## 최종 검증 결과

### LAYOUT_OVERFLOW 건수 변화 (hwpspec.hwp)

| 단계 | 건수 | 변화 |
|------|------|------|
| 수정 전 | 43건 | — |
| 1단계 후 | 42건 | −1건 |
| 2단계 후 | 20건 | −22건 |
| 3단계 후 | **4건** | −16건 |
| **총 감소** | | **−39건** |

### 전체 샘플 회귀 검사 (226개)

| 파일 | 수정 전 | 수정 후 | 변화 |
|------|--------|--------|------|
| `hwpspec.hwp` | 43 | 4 | **−39** ✅ |
| `kps-ai.hwp` | 12 | 12 | 0 |
| `tac-img-02.hwp` | 10 | 10 | 0 |
| `tac-img-02.hwpx` | 9 | 9 | 0 |
| 기타 222개 | (동일) | (동일) | 0 |

**새 회귀: 0건**

### 단위 테스트

```
785 passed; 0 failed; 1 ignored
```

### WASM 빌드 및 시각 회귀 검증

WASM 빌드 성공. 작업지시자 직접 확인 — "기존 복잡한 문서 레이아웃의 렌더링 피델리티 무너짐이 거의 없음" 확인.

---

## 잔존 LAYOUT_OVERFLOW (hwpspec.hwp 4건)

수정 범위 밖의 별도 이슈로 판단, 이번 타스크에서 제외:

| 페이지 | 문단 | 종류 | overflow |
|--------|------|------|---------|
| 5 | 65 | PartialParagraph | 13.2px |
| 9 | 127 | FullParagraph | 13.4px |
| 15 | 170 | Table | 4.8px |
| 40 | 344 | Table | 2.5px |

---

## 수정 파일 목록

- `src/renderer/pagination/engine.rs` — 2단계, 3단계
- `src/renderer/layout/height_measurer.rs` — 1단계 (SPLIT_EPSILON 제거)

---

## 관련 문서

- 수행계획서: `mydocs/plans/task_m100_101.md`
- 구현계획서: `mydocs/plans/task_m100_101_impl.md`
- 1단계 완료보고서: `mydocs/working/task_m100_101_stage1.md`
- 2단계 완료보고서: `mydocs/working/task_m100_101_stage2.md`
- 3단계 완료보고서: `mydocs/working/task_m100_101_stage3.md`
