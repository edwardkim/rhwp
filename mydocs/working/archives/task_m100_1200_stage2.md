# Stage 2 완료보고서 — Task M100 #1200

**단계**: 회귀 테스트
**브랜치**: `local/task1200`

## 추가 테스트 (`src/parser/hwpx/section.rs` tests)

`test_parse_curve_seg_populates_points`:
- `<hp:curve>` + `<hp:seg>` 3개(LINE chain, 폐곡선) 파싱.
- `CurveShape.points` 가 `[(10,10),(90,10),(90,90),(10,10)]` (첫 seg 시작점 + 각 seg 끝점) 로 채워지는지 검증.

## 결과

```
cargo test --release --lib test_parse_curve_seg
test result: ok. 1 passed; 0 failed
```

전체 스위트(`cargo test --release`) 통과, 회귀 0건.
