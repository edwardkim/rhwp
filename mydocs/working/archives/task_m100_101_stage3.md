# Task #101: 3단계 완료 보고서

> **작성일**: 2026-04-11
> **단계**: 3단계 — TAC 표 캡션 `current_height` 보정

---

## 수행 내용

### 근본 원인 (3단계 확정)

섹션4 pi=80~85 (TAC 표, 4×4)에 공(空) 캡션(`table.caption=Some(...)`)이 존재한다.

- `mt.caption_height = 12.67px`
- `caption_spacing = 11.36px`
- `caption_extra = 24.03px`

2단계 수정에서 `caption_extra_for_current`를 **비-TAC 표에만** 적용했기 때문에, TAC 표의 캡션이 `current_height`에서 누락되었다.

#### layout vs pagination 차이 (pi=80~85)

| 항목 | layout (실제) | pagination (2단계) |
|------|-------------|-----------------|
| `layout_table` 반환 | `y_start + 68.27 + 24.03 = y_start + 92.29px` | — |
| `line_spacing` | +8.0px | +8.0px |
| **총 delta** | **100.29px** | **84.80px** |
| **차이** | | **−15.49px** (6개 × = −92.9px) |

이 누적 discrepancy로 인해 pi=85 배치 시 pagination의 `current_height`가 실제보다 작게 계산되어 overflow가 발생했다.

### 수정 내용 (`src/renderer/pagination/engine.rs`)

`caption_extra_for_current` 계산에서 `!is_tac_table` 조건 제거:

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

---

## 검증 결과

### LAYOUT_OVERFLOW 총 건수 (전체 샘플)

| 파일 | 수정 전 | 수정 후 | 변화 |
|------|--------|--------|------|
| `hwpspec.hwp` | 43 | 4 | **−39** ✅ |
| `kps-ai.hwp` | 12 | 12 | 0 |
| `tac-img-02.hwp` | 10 | 10 | 0 |
| `tac-img-02.hwpx` | 9 | 9 | 0 |
| 기타 | (동일) | (동일) | 0 |

**새 회귀: 0건** — hwpspec.hwp에서만 39건 해결, 다른 샘플 무변경.

### hwpspec.hwp LAYOUT_OVERFLOW 상세

```
수정 전: 43개
수정 후: 4개
해결: 39개 (2단계 23건 + 3단계 16건)
회귀: 0건
```

잔존 4건:
- page=5, para=65, PartialParagraph, 13.2px
- page=9, para=127, FullParagraph, 13.4px
- page=15, para=170, Table, 4.8px
- page=40, para=344, Table, 2.5px

### 단위 테스트

```
785 passed; 0 failed; 1 ignored
```

### WASM 빌드

빌드 성공. 작업지시자 직접 회귀 검증 완료 — "기존 복잡한 문서 레이아웃의 렌더링 피델리티 무너짐이 거의 없음" 확인.

---

## 수정 파일

- `src/renderer/pagination/engine.rs`
