# Task #555 Stage 2 — TDD RED 보고서

**날짜**: 2026-05-04
**브랜치**: `pr-task555` (devel `f807378a`)
**선행**: Stage 1 진단 완료 (`mydocs/working/task_m100_555_stage1.md`)

## 1. 추가 산출물

### 1.1 `effective_text_for_metrics` 헬퍼 STUB

`src/renderer/composer.rs` (line 924+, +13 LOC):

```rust
/// [Task #555] 폰트 매트릭스 (글자폭/줄간격) 계산용 effective text 반환.
///
/// PUA 옛한글 변환 (Task #528) 후 `run.display_text` 가 자모 시퀀스를 보유하면
/// 본 함수는 그 자모 시퀀스를 반환한다. 그렇지 않으면 `run.text` (PUA char 1글자
/// 또는 일반 텍스트) 를 그대로 반환.
///
/// 사용처: `estimate_text_width` / `estimate_composed_line_width` 등 폰트 매트릭스
/// 측정 함수의 caller. visual 출력 (svg/web_canvas) 은 이미 `display_text` 사용.
///
/// 단일 룰 (분기/허용오차 없음): 비-PUA 텍스트는 fallback 으로 동일 동작.
pub fn effective_text_for_metrics(run: &ComposedTextRun) -> &str {
    // STAGE 2 STUB (TDD RED) — Stage 3 에서 display_text 우선 사용으로 정정.
    // 현재는 text 만 반환 (현 devel 의 결함 동작 보존).
    &run.text
}
```

### 1.2 `estimate_composed_line_width` 호출처 헬퍼 적용

`composer.rs:920` 의 caller 만 헬퍼 사용으로 변경 (RED 확인 후 Stage 3 에서 헬퍼 fix 시 자동 GREEN 전환).

```rust
- estimate_text_width(&run.text, &ts)
+ estimate_text_width(effective_text_for_metrics(run), &ts)
```

### 1.3 단위 테스트 3건 추가 (`composer/tests.rs`)

| 테스트 | 본질 | 결과 |
|--------|------|------|
| `test_555_effective_text_for_metrics_uses_display_text_when_present` | PUA char + display_text="《" 시 helper 가 "《" 반환 | **RED** ❌ (STUB 은 PUA char 반환) |
| `test_555_effective_text_for_metrics_multi_jamo_cluster` | PUA 합자 + display_text="ᄃᆞᄫᆡ" (4 jamo) 시 helper 가 4-char 반환 | **RED** ❌ (STUB 은 1 char 반환) |
| `test_555_effective_text_for_metrics_no_display_text_falls_back_to_text` | display_text=None 시 text 반환 (fallback 회귀 가드) | **GREEN** ✅ |

## 2. 검증 (RED 상태 확인)

```
cargo test --lib --release test_555
test renderer::composer::tests::test_555_effective_text_for_metrics_no_display_text_falls_back_to_text ... ok
test renderer::composer::tests::test_555_effective_text_for_metrics_uses_display_text_when_present ... FAILED
test renderer::composer::tests::test_555_effective_text_for_metrics_multi_jamo_cluster ... FAILED

failures:
    test_555_effective_text_for_metrics_multi_jamo_cluster
    test_555_effective_text_for_metrics_uses_display_text_when_present

test result: FAILED. 1 passed; 2 failed
```

→ 2 RED + 1 GREEN, 의도된 TDD RED 상태.

## 3. 무회귀 검증

```
cargo test --lib --release
test result: FAILED. 1121 passed; 2 failed; 3 ignored
```

- baseline 1121 (Task #548 적용 후) + 1 GREEN 신규 = 1121 passed (TDD RED 2건 제외)
- 비-Task #555 테스트 회귀 0건
- ignored: 3건 (test_552 + test_548 v3 등 기존 ignored, 변동 없음)

## 4. Stage 3 진행 권장

- helper 의 STUB 본문 정정: `&run.text` → `run.display_text.as_deref().unwrap_or(&run.text)`
- 호출처 9건 추가 적용 (composer.rs:920 는 Stage 2 에서 이미 적용):
  - `layout.rs:3444` (Square wrap host est_x — char-by-char)
  - `layout.rs:3510/3516/3522` (`compute_tac_leading_width`)
  - `table_layout.rs:860` (셀 컨텐츠 max width)
  - `table_layout.rs:1657/1814/1840/1922` (셀 inline shape text_before)

## 5. 작업지시자 결정 사항

1. **Stage 2 (TDD RED) 결과 승인** — 2 RED + 1 GREEN, 의도된 상태
2. Stage 3 (본질 정정) 진행 승인
