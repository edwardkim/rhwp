---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 표 셀 렌더링의 암묵지 — 한컴 `letter_spacing` 보정, narrow glyph 역진, 자연폭 가드

## 문서 성격

이 문서는 **스펙 문서에 적혀 있지 않은 HWP 조판 엔진의 암묵적 저장 의도**와, 그것을 모르고 rhwp를 수정할 경우 반복적으로 마주치게 될 회귀 패턴을 기록한다.

후발 기여자가 `src/renderer/layout/text_measurement.rs` / `paragraph_layout.rs` 를 수정하려 할 때 **며칠을 날릴 수 있는 함정**이 있으므로, 손대기 전에 반드시 읽는다.

출처: Issue #229 (1차 해결) + Task #229 Fix (회귀 수정, PR #235) / 기여자 @planet6897

## 배경: 왜 표 셀 긴 숫자가 겹쳐 보이는가

금융·통계 HWP 문서에서 `"65,063,026,600"` 같은 긴 숫자 셀을 rhwp로 렌더하면, 한컴 결과와 두 가지 차이가 난다.

1. **셀 폭 미사용** — rhwp 폰트 메트릭으로는 자연 폭이 셀 inner 폭보다 좁게 측정되어, Center 정렬 시 좌우 여백이 크게 남는다.
2. **narrow glyph 역진 겹침** — 콤마·마침표 다음 숫자가 콤마보다 작은 x 좌표에 배치되어 콤마 위에 숫자가 얹힌다.

두 현상의 뿌리는 동일하다. **HWP 편집기가 CharShape의 `letter_spacing` 필드에 음수 자간을 미리 저장해둔다**는 사실에서 출발한다.

## 암묵지 1 — 한컴 `letter_spacing` 저장 의도

### 사실

한컴 편집기는 셀보다 자연 폭이 큰 텍스트를 저장할 때, **자간(letter_spacing)을 음수로 미리 계산해서 파일에 박아둔다**. 예: 12pt 숫자 문자열이 셀에 들어가도록 `letter_spacing = -2.88 px`.

### 의미

- 스펙 문서(`한글 문서 파일 형식.pdf`)의 CharShape 정의에는 `letter_spacing`이 "자간 조정값"으로만 적혀 있다. **저장 측이 렌더 측의 폰트 메트릭 차이를 보정하기 위해 역산해 넣는다**는 맥락은 없다.
- 즉 `letter_spacing`은 **사용자 의도의 자간**(장식적 tightening)일 수도 있고, **편집기가 렌더링 폭을 맞추기 위해 저장한 보정값**일 수도 있다. 파일만 봐서는 둘을 구별할 수 없다.

### 왜 구별해야 하는가

rhwp의 폰트 메트릭은 한컴 내장 폰트 DB와 다르다. 같은 음수 자간을 그대로 적용하면:

- **편집기 보정 케이스**: rhwp에서는 측정이 이미 더 좁은데 거기에 또 음수를 더해서 → 글자가 역진하고 셀에 좌우 여백이 남음
- **장식적 tightening 케이스**: 사용자 의도대로 좁은 조판이 필요한 정상 케이스

rhwp는 **편집기 보정 케이스에서만 반대 방향(양수 자간)으로 재보정**해야 하고, **장식적 tightening은 그대로 존중**해야 한다.

### 판별 기준 (핵심)

```
자연 폭(letter_spacing = 0 기준 측정) > 셀 available_width
```

- **true** → 편집기 보정. `extra_char_spacing`을 양수로 주어 셀 폭에 수렴시킨다.
- **false** → 사용자 의도의 tightening. 기존 동작 유지.

이 가드를 빼거나 약화시키면 `form-002.hwpx` 같은 일반 문서의 기존 레이아웃이 깨진다.

**소스 위치**: [src/renderer/layout/paragraph_layout.rs](../../src/renderer/layout/paragraph_layout.rs) — underflow 자간 확장 분기 (주석 `// 자연 폭(ls=0) > available_width` 근처).

## 암묵지 2 — narrow glyph 역진

### 사실

음수 자간(`letter_spacing + extra_char_spacing < 0`)이 적용된 run에서 콤마·마침표처럼 base advance가 좁은 글리프는 **effective advance가 음수**가 될 수 있다.

```
콤마 base ≈ 2.61 px (12pt)
letter_spacing = -2.88 px
effective = 2.61 + (-2.88) = -0.27 px  ← 음수
```

결과적으로 콤마 다음 글자가 콤마보다 **작은 x 좌표**에 배치되어 역진 겹침이 발생한다.

### 처방

per-char 최소 advance를 **`base_w * ratio * 0.5`** 로 클램프한다.

```rust
if style.letter_spacing + style.extra_char_spacing < 0.0 {
    let min_w = base_w * ratio * 0.5;
    w = w.max(min_w);
}
```

### 클램프를 적용할 5곳

[src/renderer/layout/text_measurement.rs](../../src/renderer/layout/text_measurement.rs) 의 `char_width` 클로저 **5곳 전부에 일관되게** 적용해야 한다. 한두 곳만 고치면 측정 경로와 배치 경로가 어긋나 수렴 반복이 실패한다.

1. `EmbeddedTextMeasurer::estimate_text_width`
2. `EmbeddedTextMeasurer::compute_char_positions`
3. `WasmTextMeasurer::estimate_text_width`
4. `WasmTextMeasurer::compute_char_positions`
5. `estimate_text_width_unrounded`

### 가드 조건의 중요성

**양수 자간 또는 0 케이스에는 절대 클램프를 적용하지 않는다.** 과거에 무조건 클램프했던 커밋(`8c9b366`)을 되돌린 커밋(`21a02ec`)이 있었고, 그 이유는 비-오버플로우 셀의 측정까지 영향을 받아 정상 문서의 레이아웃이 깨졌기 때문이다.

**회귀 방지 테스트**: `test_non_compression_width_unchanged_by_fix` ([src/renderer/layout/text_measurement.rs](../../src/renderer/layout/text_measurement.rs) 의 test 모듈)이 이 가드를 지키는지 확인한다.

## 암묵지 3 — 수렴 반복 3회

### 사실

narrow glyph per-char 클램프가 들어가면, 선형 1회 분배로 계산한 `extra_char_spacing`이 **실제 렌더 폭과 어긋난다**. 클램프가 음수 기여를 상쇄하기 때문.

### 처방

[src/renderer/layout/paragraph_layout.rs](../../src/renderer/layout/paragraph_layout.rs) 의 underflow 확장 분기에서 **최대 3회 수렴 반복**으로 `extra_char_spacing`을 재계산한다.

이 3회는 실험적으로 결정된 값이다. 1회로는 수렴하지 않는 경우가 있고, 4회 이상은 대부분 의미 없음. 수정 시 수렴 기준을 임의로 낮추지 않는다.

## 암묵지 4 — `effective_text_width` 복원

### 사실

Center / Right / Distribute 정렬의 x 시작점 계산은 **실제 렌더 폭을 반영**해야 한다. `total_text_width`만으로 계산하면 자간 확장 후 실제 폭과 달라서 정렬이 어긋난다.

### 처방

```rust
let effective_text_width = if extra_char_sp > 0.0 && cell_ctx.is_some() && ... {
    total_text_width + extra_char_sp * total_char_count as f64
} else {
    total_text_width
};
```

이 계산을 뺐더니 정렬이 밀렸다는 회귀가 과거에 있었음.

## 디버깅 신호 요약

표 셀 렌더링 버그를 만났을 때 다음을 먼저 확인한다:

| 증상 | 의심 지점 |
|------|----------|
| 콤마·마침표 다음 글자가 겹침 | narrow glyph 역진. `letter_spacing + extra_char_spacing < 0` 조건에서 per-char 클램프 5곳 누락 여부 확인 |
| 일반 문서(form-002 등) 레이아웃 깨짐 | 클램프를 가드 없이 적용한 경우. `< 0` 가드 누락 확인 |
| 긴 숫자 셀이 좌우 여백 크게 남음 | underflow 확장 분기에서 `자연폭 > available_width` 가드가 막고 있는지 확인 |
| 장식적 tightening 문서의 자간이 멋대로 확장됨 | 자연폭 가드 누락. 편집기 보정과 사용자 의도를 구별 못 함 |

## 재현 샘플

- [samples/hwpx/table-text.hwpx](../../samples/hwpx/table-text.hwpx) — 긴 숫자 셀 + 음수 자간 (주 재현 샘플)
- [samples/hwpx/form-002.hwpx](../../samples/hwpx/form-002.hwpx) — 장식적 tightening 비회귀 가드 (자연폭 ≤ 셀폭 케이스)

## 회귀 테스트

| 테스트 | 위치 | 역할 |
|--------|------|------|
| `test_overflow_compression_positions_monotonic_comma` | [src/renderer/layout/text_measurement.rs](../../src/renderer/layout/text_measurement.rs) | `extra_char_spacing=-2.88` 콤마 단조성 |
| `test_overflow_compression_positions_monotonic_period` | 동일 | 마침표 단조성 |
| `test_charshape_negative_letter_spacing_no_reverse` | 동일 | 실제 문서 재현 단조성 |
| `test_non_compression_width_unchanged_by_fix` | 동일 | 비-압축 경로 비회귀 가드 |
| `table_text_page_0` | [tests/svg_snapshot.rs](../../tests/svg_snapshot.rs) | 골든 SVG 스냅샷 |
| `form_002_page_0` | 동일 | 장식적 tightening 비회귀 가드 |

## 역사

| 커밋 | 내용 | 결과 |
|------|------|------|
| `8c9b366` (Task #229 1차) | per-char 50% 클램프 5곳 무조건 적용 | 긴 숫자 겹침 해결, but 일반 문서 레이아웃 회귀 |
| `21a02ec` (Task #229 후속) | 5곳 클램프 모두 제거 | 일반 문서 복구, but 긴 숫자 역진 회귀 |
| PR #235 (Task #229 Fix) | 가드 조건부로 클램프 + underflow 확장 복원 | 두 케이스 모두 해결 |

## 관련 문서

- [mydocs/plans/archives/task_m100_229.md](../plans/archives/task_m100_229.md) — 수행 계획서
- [mydocs/plans/archives/task_m100_229_impl.md](../plans/archives/task_m100_229_impl.md) — 구현 계획서
- [mydocs/report/archives/task_m100_229_report.md](../report/archives/task_m100_229_report.md) — 최종 보고서
- [PR #235](https://github.com/edwardkim/rhwp/pull/235)
- [Issue #229](https://github.com/edwardkim/rhwp/issues/229)

## 기여자 크레딧

이 암묵지를 밝혀낸 것은 기여자 [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) 님이다. HWP 스펙 문서에는 없는 **편집기의 저장 의도**를 Visual Diff 검증과 역산을 통해 도출하신 기여로, rhwp 조판 엔진 이해에 큰 이정표가 되었다.
