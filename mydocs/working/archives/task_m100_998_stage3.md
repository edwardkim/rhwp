# Task #998 Stage 3 — 회귀 + 시각 검증

- 이슈: [#998](https://github.com/edwardkim/rhwp/issues/998)
- 선행: [Stage 2 구현 계획서](../plans/task_m100_998_impl.md)

## 1. 구현 요약

### 변경 1: composer.rs CHARS_PER_LINE 35 → 45
HWP3 reference 측정 결과 (43~46 chars/line) 에 맞춰 휴리스틱 조정.

### 변경 2: typeset.rs spacing_before 보정 추가
HWP5 변환본의 line_segs 누락 paragraph 가 ParaShape `spacing_before=2264 HU` (HWP3 1132 의 2x) 보유 — 데이터 자체 차이로 ~1 페이지 inflate. typeset.format_paragraph 에서 `line_segs.is_empty() && !text.is_empty()` case 에 `spacing_before=0` 적용 → HWP3 reference 와 정합.

## 2. cargo test --release --lib

```
test result: ok. 1297 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

✅ 전체 통과.

## 3. 240 sample 페이지 수 회귀

```
diff baseline (pre-G4) post (task #998):
166c166
< hwp3-sample16-hwp5.hwp: 62  ← pre-G4 (overlap, page 누락)
---
> hwp3-sample16-hwp5.hwp: 64  ← task #998 (HWP3 reference 정합)

223a224
> hy-001.hwpx: 2  ← 신규 sample (회귀 아님)
```

- 240 sample 중 **변동 1 건** (타깃 sample 만)
- HWP3 / HWPX / 다른 HWP5 sample 모두 변동 0

## 4. HWP5 sample16 시각 검증 (작업지시자 판정)

- Page 19, 22, 23 wrap 정상 ✓
- chars overflow 없음 (우측 page boundary 안쪽) ✓  
- 한컴 viewer 와 자연스러운 정렬 ✓
- 페이지 수 64 정합 ✓ (HWP3 reference 정합)

작업지시자 시각 판정 **통과**.

## 5. 잔존 (별도 후속)

### 자동 보정 path 의 페이지 수 차이 (69 페이지)

- 그대로 보기 (composer fallback): **64** (본 fix 적용)
- 자동 보정 (`reflow_line_segs` 호출): **69** (본 fix path 미통과)

자동 보정은 `reflow_line_segs` 가 line_segs 를 채워서 composer fallback 진입 안 함 + typeset 의 spacing_before=0 조건도 미충족 → 본 fix 미적용 → raw ParaShape (spacing_before=2264) 사용 → +5 페이지.

→ 별도 task 분리 예정.

## 6. HWPX 변종

`samples/hwp3-sample16-hwp5.hwpx` 는 본 fix 영향 없음 (parser path 다름):
- HWPX 는 `<hp:linesegarray>` preset 으로 line_segs 채움 (즉 line_segs.is_empty() == false)
- composer fallback 미통과 + typeset 의 spacing_before=0 조건 미충족
- 페이지 수 72 (PR #989 D6 이후 동일) — #942/#988 close 영역
