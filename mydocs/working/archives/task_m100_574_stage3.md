# Task #574 Stage 3 — Fix 적용 (HY견명조 heavy 제거)

**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574

---

## 1. 변경 사항

### 1.1 `src/renderer/style_resolver.rs:601-616`

`is_heavy_display_face` 의 `matches!` 패턴에서 `"HY견명조"` 제거:

```rust
matches!(primary,
    "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
    | "HY견고딕" | "HY견명조B"   // ← "HY견명조" 제거
    | "HY그래픽" | "HY그래픽M"
)
```

doc comment 에 Task #574 변경 사유 추가.

## 2. 단위/통합 테스트 RED → GREEN

### 2.1 단위 테스트

```
$ cargo test --release --lib test_is_heavy_display_face -- --nocapture

test renderer::layout::tests::test_is_heavy_display_face_with_family_chain ... ok
test renderer::layout::tests::test_is_heavy_display_face_matches_known_heavy_faces ... ok
test result: ok. 2 passed; 0 failed
```

### 2.2 통합 테스트

```
$ cargo test --release --lib test_574_page_number_not_force_bold -- --nocapture

test renderer::layout::integration_tests::tests::
    test_574_page_number_not_force_bold_for_hy_kyun_myeongjo ... ok
test result: ok. 1 passed; 0 failed
```

→ 두 테스트 모두 RED → GREEN. 본질 fix 단일 줄 변경으로 해결.

## 3. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/style_resolver.rs:610` | `\| "HY견명조"` 제거 + Task #574 doc 주석 추가 |
| `mydocs/working/task_m100_574_stage3.md` | 본 보고서 |

## 4. 다음 단계

Stage 4 — 광범위 회귀 sweep (7개 샘플 + cargo test --lib + clippy).
