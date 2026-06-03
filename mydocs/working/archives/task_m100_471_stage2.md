# Task #471 Stage 2 완료 보고서

## 단계 목표

`src/renderer/layout.rs:1670-1699` Task #468 cross-column 검출의 비교 기준을 bf_id → stroke_sig 로 변경.

## 변경 내용

```rust
let group_sig = stroke_sig(bf_id);
if group_sig.is_none() { continue; }
let para_bf = |pi: usize| -> u16 { ... };

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
```

머지된 그룹의 첫 range bf_id 가 다른 paragraph (예: pi=6 bf=7) 의 bf_id 라도, stroke_sig 가 같으면 cross-column 인접 paragraph (예: pi=10 bf=4) 와 정확히 매칭.

## 결과

Stage 1 의 red 테스트가 green 으로 전환:
- 좌측 단 (가) 박스 4면 stroke rect 부재 ✓
- partial_end=true → skip_bottom=true → bottom 가로선 미렌더링

## 다음 단계

Stage 3 — 전체 cargo test + 다단 샘플 OVERFLOW 회귀 검증.
