# Task #471 구현계획서

## 변경 위치

`src/renderer/layout.rs:1670-1699` Task #468 cross-column 검출 블록.

## 변경 내용

### Before

```rust
for g in groups.iter_mut() {
    let bf_id = g.0;
    if bf_id == 0 { continue; }
    let first_pi = g.9;
    let last_pi = g.10;

    if !g.7 && first_pi > 0 {
        let prev_bf = composed.get(first_pi - 1)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id)
            .unwrap_or(0);
        if prev_bf == bf_id {
            g.7 = true;
        }
    }

    if !g.8 {
        let next_bf = composed.get(last_pi + 1)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id)
            .unwrap_or(0);
        if next_bf == bf_id {
            g.8 = true;
        }
    }
}
```

### After

```rust
// [Task #471] bf_id 비교가 아닌 stroke_sig 비교 — 머지가 visual stroke 기준 (Task #321 v6)
// 으로 동작하므로 그룹의 g.0 bf_id 는 첫 range 의 bf_id 만 보존됨. 그룹의 visual sig
// 와 인접 paragraph 의 visual sig 비교가 정확.
for g in groups.iter_mut() {
    let bf_id = g.0;
    if bf_id == 0 { continue; }
    let first_pi = g.9;
    let last_pi = g.10;
    let group_sig = stroke_sig(bf_id);
    if group_sig.is_none() { continue; }

    let para_bf = |pi: usize| -> u16 {
        composed.get(pi)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id)
            .unwrap_or(0)
    };

    if !g.7 && first_pi > 0 {
        let prev_sig = stroke_sig(para_bf(first_pi - 1));
        if prev_sig.is_some() && prev_sig == group_sig {
            g.7 = true;
        }
    }

    if !g.8 {
        let next_sig = stroke_sig(para_bf(last_pi + 1));
        if next_sig.is_some() && next_sig == group_sig {
            g.8 = true;
        }
    }
}
```

## 단계 구성

### Stage 1: 통합 테스트

`src/renderer/layout/integration_tests.rs` 에 `test_471_cross_column_box_no_bottom_line_in_col0` 추가.

`samples/21_언어_기출_편집가능본.hwp` 페이지 1 SVG 의 좌측 단 영역 (x ∈ [128, 542]) 에서 y ≈ 1438 부근의 가로선(stroke_width=0.5)이 없어야 함.

### Stage 2: 코드 수정

L1670-1699 블록 교체.

### Stage 3: 회귀 검증

- 신규 테스트 + 기존 테스트 + svg_snapshot
- 다단 샘플 OVERFLOW 비교 (불변 기대)

## 검증 명령

```bash
cargo build --release
cargo test --release
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 -o /tmp/p21/
# col 0 (x < 540) 영역 y > 1400 의 가로선 부재 확인
```
