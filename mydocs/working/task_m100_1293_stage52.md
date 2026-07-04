# Task 1293 Stage 52: 최신 코드 기준 full sweep all 재검증

## 목적

Stage51에서 focused 회귀를 복구한 뒤 `typeset.rs`가 다시 변경되었다. Stage50의 full sweep all
결과를 그대로 완료 근거로 쓰면 stale evidence가 되므로, 최신 커밋 `57d1d719` 기준 native CLI로
전체 visual sweep을 다시 수행한다.

## 실행

Stage51 검증 중 `cargo build --bin rhwp`로 `target/debug/rhwp`를 최신 코드로 갱신한 뒤 실행했다.

```bash
rm -rf output/task1293_stage51_full_sweep_final
python3 scripts/task1274_visual_sweep.py \
  --target all \
  --out output/task1293_stage51_full_sweep_final \
  --rhwp-bin target/debug/rhwp
```

## 결과 요약

- 결과 파일: `output/task1293_stage51_full_sweep_final/summary.json`
- target 수: 15
- SVG/PDF/render tree page count mismatch: 0
- renderer `LAYOUT_OVERFLOW`: 0
- frame overflow 후보: 0
- question title/text overlap 후보: 0
- equation/text overlap 후보: 0
- line order overlap 후보: 0
- endnote separator gap drift 후보: 0

| target | page count | overflow | frame | title | equation | order | separator drift |
|---|---:|---:|---:|---:|---:|---:|---:|
| `2022-09` | 23/23/23 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2023-09` | 20/20/20 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-between20` | 24/24/24 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-below20-above20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2022-10` | 18/18/18 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2022-11-practice` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 | 0 |

## 참고 후보

자동 sweep의 visual drift 후보는 아직 남아 있다.

- red marker drift 후보: 121
- line band drift 후보: 144
- large ink region drift 후보: 187
- content bottom drift 후보: 113

이 후보들은 PDF/PNG 직접 확인 대상이다. Stage52의 완료 근거는 page count, renderer overflow,
frame/title/equation/order overlap, separator gap drift의 핵심 게이트가 모두 0이라는 점이다.

## 판단

구현 계획서의 visual 검증 항목인 `--target all`은 최신 코드 기준 핵심 게이트를 통과했다. 남은
red/line/large/content bottom drift 후보는 정합성 후보로 추적하되, 현재 자동 게이트상 미주 기능
완료 판단을 막는 overflow/겹침/구분선 gap 회귀는 없다.
