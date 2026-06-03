# Stage 5-2 완료보고서 — #1022: 정합 설계 (Option A 채택)

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5-2 — VPOS_CORR ↔ paginator 정합 설계

## 1. 결정적 발견 — VPOS_CORR 의 `+trailing_ls_hu` 는 obsolete bug

### 1-1. 정책 변천

| 시점 | paragraph_layout body advance 정책 | VPOS_CORR 보정 |
|------|-------------------------------------|---------------|
| Task #332 이전 | trailing_ls 포함 | — |
| Task #332/#479 | trailing_ls 제외 ("lh_sum + (n-1)*ls") | — |
| Task #537 (VPOS_CORR 추가) | trailing_ls 제외(당시 정책) | `y_delta_hu += trailing_ls_hu` 로 IR 절대좌표 보정 |
| **Task #452** ← 결정적 변경 | **trailing_ls 포함으로 회복** ("pagination 과 1 ls drift 발생 → 회복") | **(미갱신)** |

`paragraph_layout.rs:3583-3589` 의 Task #452 코멘트:

> 셀 내 마지막 문단의 마지막 줄: trailing line_spacing 제외 / **그 외 모든 줄(본문 단락의 마지막 줄 포함): trailing line_spacing 가산**. pagination/engine.rs 의 current_height 누적(para_height = sum(lh+ls))과 정합. (Task #452: 이전 #332 의 layout-only trailing 제외 → pagination 과 1 ls drift 발생 → 회복)

### 1-2. 현재 상태 (HEAD 9eba727f)

- `paragraph_layout` body advance: trailing_ls **포함** (Task #452 이후).
- VPOS_CORR `y_delta_hu`: trailing_ls 보정 **여전히 적용** (Task #537 의 stale 보정).

결과: 같은 IR 좌표를 두 번 보정 → fresh-compute 시 `+trailing_ls` 만큼 forward shift. 페이지 22 27.74px drift 의 원천.

### 1-3. 검증

페이지 22 (`RHWP_VPOS_DEBUG=1`):
- pi=223 lazy_base = `prev_vpos_end - (y_delta_hu_natural + trailing_ls_hu)` = `1223725 - (40480 + 1040)` = `1182205`.
- 만약 trailing_ls_hu 제거: `lazy_base = 1223725 - 40480 = 1183245`.
- end_y = `col_y + (1223725 - 1183245)/75` = `105.81 + 539.7` = `645.55` = y_in **변화 없음**.

즉 trailing_ls_hu 가 없으면 VPOS_CORR 가 정상 y_offset 을 그대로 두어 paginator 와 정합.

## 2. Option A — 렌더러 측 수정 (채택)

`layout.rs:2374` 의 `y_delta_hu` 산식에서 `+ trailing_ls_hu` 제거.

### 의미

- VPOS_CORR 가 paragraph_layout 의 실제 advance 와 정합한 base 계산.
- 자연 y_offset 과 IR vpos 가 일치하는 정상 케이스에서 보정 없음.
- 진정으로 어긋난 케이스(예: shape jump, vpos reset)에서만 보정.

### 영향 분석

- paginator 측은 변경 없음 — Stage 3 의 cell_units 정합만 유지.
- 렌더러: VPOS_CORR 의 effective drift `+trailing_ls` 제거. paragraph 사이 위치가 IR 좌표대로 정렬.
- 골든 영향: 기존 골든은 VPOS_CORR drift 가 적용된 상태로 캡처 → trailing_ls 만큼 shifted. Option A 적용 후 미세 shift. PDF 대조로 정정/회귀 판정.

### 위험·완화

- 다수 task 누적 회귀 위험: trailing_ls 보정에 의존하던 케이스 회귀 가능. **단계적 검증**:
  1. 단위 테스트 1302 통과 확인.
  2. svg_snapshot (form-002 등) 회귀 점검.
  3. 비공개 184페이지 LAYOUT_OVERFLOW 측정 (목표 0).
  4. 골든 SVG 광범위 회귀 점검 + 이동분 PDF 대조 판정.

- Task #537 의도가 다른 케이스 보정이었다면 (즉 VPOS_CORR 가 이전 정책 #479 시점에 작성되어 그 케이스를 노렸다면) Option A 는 그 케이스를 회귀시킨다 — 점검 후 필요 시 가드 추가.

## 3. Option B — paginator 미러 (비교용·미채택)

paginator 가 `+trailing_ls_hu` 와 lazy_base lifecycle 을 그대로 미러링. 의미적으론 buggy 동작의 대칭 유지지만, 두 모듈에 동일 bug 가 들어감. Option A 가 본질적 정합.

## 4. Stage 5-3 구현 진행

`src/renderer/layout.rs:2374` 한 줄 수정:

```rust
// BEFORE
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0)
    .round() as i32
    + trailing_ls_hu;

// AFTER (Task #1022)
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0)
    .round() as i32;
// Task #452 이후 paragraph_layout 가 trailing_ls 를 포함 advance 하므로
// Task #537 의 +trailing_ls_hu 보정은 over-correction. 제거.
```

`trailing_ls_hu` 변수 자체는 다른 곳에서 쓰이면 유지, 아니면 제거.

## 5. 검증 절차

- `cargo test --release` 1302+ pass.
- `cargo clippy --release` 무경고.
- `LAYOUT_OVERFLOW` 비공개 측정: 38 → 목표 ≤ 12 (Stage 5 종합보고서 §1 의 fresh-compute 27.74px 분 + 다른 페이지 fresh 분 해소).
- form-002 골든 — PDF 정합 유지 확인.
- 공개 골든 광범위 회귀 → PDF 대조 판정.
