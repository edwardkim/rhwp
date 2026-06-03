# Task #994 Stage 3 — 회귀 검증 + 시각 검증

- 이슈: [#994](https://github.com/edwardkim/rhwp/issues/994)
- 선행: [Stage 2 구현](task_m100_994_stage2.md)

## 1. cargo test --release --lib

```
test result: ok. 1297 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

✅ 전체 통과.

## 2. cargo fmt --check

✅ 통과.

## 3. 240 sample 페이지 수 회귀

```
diff baseline post.txt:
166c166
< hwp3-sample16-hwp5.hwp: 62
---
> hwp3-sample16-hwp5.hwp: 67   ← 타깃 sample fix 효과 (+5)
223a224
> hy-001.hwpx: 2  ← baseline 에 없던 신규 sample (회귀 아님)
```

- 240 sample 중 **변동 1 건** (타깃 sample 만)
- HWP3 / HWPX / 다른 HWP5 sample 모두 변동 0

## 4. HWP5 sample16 시각 검증 (작업지시자 판정)

### Pre-fix (page 19~24)
- 겹침 발생 (`󰏅 ...` paragraph 들이 한 y 에 모든 chars 그려짐)

### Post-fix
- 정상 wrap ✓
- 자연스러운 chars 정렬 (Justify 부풀림 없음) ✓
- 한컴 viewer 와 유사 ✓

작업지시자 시각 판정 **통과**.

## 5. Editor 기능 영향

- composer 내부만 변경 — parser 미변경
- insert_text / save / cursor / logical_offset 영향 없음
- editor pipeline 보존

## 6. 잔존 (후속 issue 필요)

### Page count 차이 분석
| 변종 | 페이지 수 | 비고 |
|------|---------|------|
| HWP3 sample16.hwp | 64 | reference |
| HWP5 sample16-hwp5.hwp (pre G4) | 62 | -2 |
| HWP5 sample16-hwp5.hwp (post G4) | 67 | +3 (HWP3 대비) |
| HWPX sample16-hwp5.hwpx | (별도 측정 필요) | |
| HWPX 자동보정 | 69 | studio "자동 보정" 후 |

→ 후속 issue 등록 예정 (별도 root cause: paragraph height 누적 차이)
