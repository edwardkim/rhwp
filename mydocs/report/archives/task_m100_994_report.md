# Task #994 최종 보고서 — HWP5 line_segs 누락 paragraph 겹침 해소

- 이슈: [#994](https://github.com/edwardkim/rhwp/issues/994)
- 브랜치: `local/task994`
- 일자: 2026-05-18

## 1. 작업 결과

`samples/hwp3-sample16-hwp5.hwp` 페이지 19~24 의 paragraph 겹침 해소.

### 변경 파일
- `src/renderer/composer.rs` (+33 lines) — `compose_lines` fallback 의 word wrap 합성

### 효과
- HWP5 sample16 page 19~24 시각: **겹침 해소 + 자연스러운 정렬** (작업지시자 판정 통과)
- cargo test --release --lib: **1297 passed, 0 failed** ✓
- cargo fmt --check: 통과 ✓
- 240 sample 페이지 수: 타깃 1 건 (62→67), 회귀 0 ✓
- Editor 기능 (parser 미변경): 영향 없음 ✓

## 2. Root cause

HWP3 → HWP5 변환 시 일부 paragraph (`󰏅 ...` PUA bullet long text) 의 `PARA_LINE_SEG` 누락.

- HWP3: `ls[0] lh=1300 ls=780` 있음 → composer 가 line_segs 기반 처리
- HWP5: `line_segs.is_empty()` → composer fallback 의 단일 ComposedLine 생성 → layout 이 wrap 없이 한 y 좌표에 모든 chars → **시각 겹침**

영향 paragraph: HWP5 sample16 에서 **59 개**.

## 3. Fix (G4-final, 3차 반복)

### 변경 위치
[src/renderer/composer.rs:321-368](../../src/renderer/composer.rs#L321-L368) — `compose_lines` 의 `line_segs.is_empty()` fallback.

### 핵심 개선
1. **Word boundary 분할**: ~35 chars/line 한도 내 가장 가까운 공백 후로 break — mid-word 분할 회피
2. **`has_line_break=true` marking**: non-last synth line — Justify 정렬 비활성화 (chars spacing 부풀림 회피)

### 결과
- composer 가 paragraph 를 다수 ComposedLine 으로 분할
- layout 이 line 별로 정상 wrap + 자연스러운 chars 정렬

## 4. 회귀 영향

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | ✅ 1297 / 0 failed |
| cargo fmt --check | ✅ |
| 240 sample 페이지 수 | ✅ 타깃 1 건 만 (62→67) |
| 시각 회귀 | 없음 (타깃 sample 만 변경) |
| Editor 기능 | 영향 없음 (parser 미변경) |

## 5. 단계별 진행

| Stage | 내용 | 산출물 |
|-------|------|--------|
| 1. 진단 정밀화 | line_segs 누락 paragraph 식별 (59건) | working/stage1.md |
| 2. G4 구현 | word wrap + has_line_break | working/stage2.md |
| 3. 회귀 + 시각 검증 | cargo test + 240 sample + 시각 판정 | working/stage3.md |
| 4. 시각 승인 | 작업지시자 판정 통과 | 본 commit |
| 5. PR | 본 보고서 + PR 생성 | TBD |

## 6. 잔존 / 후속

### Page count 차이 (별도 후속 이슈 예정)
| 변종 | 페이지 수 |
|------|---------|
| HWP3 sample16.hwp | 64 (reference) |
| HWP5 sample16-hwp5.hwp (post G4) | 67 (+3) |
| HWPX 자동보정 후 | 69 (+5) |

→ paragraph height 누적 차이가 다른 root cause. 별도 task 분리 예정.

### HWPX 변종 영향
HWPX (`sample16-hwp5.hwpx`) 도 동일 path 통과 — G4 적용 시 영향 있을 수 있으나 본 task scope 외.

## 7. 산출물

- `mydocs/plans/task_m100_994.md` (수행 계획서)
- `mydocs/plans/task_m100_994_impl.md` (구현 계획서)
- `mydocs/working/task_m100_994_stage1.md` (진단)
- `mydocs/working/task_m100_994_stage2.md` (G4 구현)
- `mydocs/working/task_m100_994_stage3.md` (회귀 + 시각 검증)
- 본 보고서
- 소스 변경: `src/renderer/composer.rs` +33 lines
