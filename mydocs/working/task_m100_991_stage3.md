# Task #991 Stage 3 — 회귀 검증

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 선행: [Stage 2 fix 결정](task_m100_991_stage2.md)

## 1. cargo test --release --lib

```
test result: ok. 1297 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

✅ 모든 unit + integration test 통과.

### 시도별 비교
| | cargo test fail | Editor |
|---|---|---|
| Pre-fix (baseline) | 0 | OK |
| F1 (parser 광범위) | 9 | 깨짐 |
| F1-narrow (ch=11/14) | 5 | 일부 깨짐 |
| F2-wide | 5 | OK |
| **F2-narrow (본 fix)** | **0** ✓ | OK |

## 2. 240 sample 페이지 수 회귀

`find samples -maxdepth 2 -name "*.hwp" -o -name "*.hwpx"` 전체 비교:

```
diff baseline pre.txt post.txt:
223a224
> hy-001.hwpx: 2
```

- 240 sample 중 **변동 0 건**
- `hy-001.hwpx: 2` 는 baseline 에 없던 신규 sample (회귀 아님)

### HWP5 sample16 별도 측정
- pre-fix: 62 페이지
- post-fix: 62 페이지 (변동 없음)

## 3. rustfmt

```
$ cargo fmt --check
(no diff)
```

✅ 통과.

## 4. 종합

| 검증 | 결과 |
|------|------|
| cargo test --release --lib | ✅ 1297 / 0 failed |
| 240 sample 페이지 수 | ✅ 변동 0 |
| cargo fmt --check | ✅ 통과 |
| Editor pipeline | ✅ 영향 없음 |
