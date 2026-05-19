# Task #1007 Stage 4 — 회귀 검증 + Tuning

이슈: [#1007](https://github.com/edwardkim/rhwp/issues/1007)
Stage 1/2/3: [`stage1`](task_m100_1007_stage1.md), [`stage2`](task_m100_1007_stage2.md), [`stage3`](task_m100_1007_stage3.md)

## 1. Stage 3 이후 추가 fix

| 영역 | 변경 | 효과 |
|------|------|------|
| `composer.rs` CHARS_PER_LINE 35/45 → **50** | 변환본 CharShape spacing=-12% 보정 | pi=400 50 chars 가 2 line → 1 line. h 63→35 |
| `typeset.rs` / `engine.rs` low_threshold 절대값 1500 HU (variant), 4000 HU (curr empty line_segs) | column wrap (first_v 3000+) 과 page reset (first_v <1500) 구별 | 페이지 단위 split alignment 향상 |
| `prev_end_vpos = vpos + line_height` | wrap shape/Table 의 lh 큰 값 반영 | pi=413 wrap shape 후 break 감지 (페이지 19/20) |
| prev/curr walk-back/forward (empty line_segs skip) | 변환본 PARA_LINE_SEG 누락 paragraph 대응 | pi=426 (empty line_segs) break 감지 |
| aux_trigger: empty-paragraph-bridge 감지 | 다수 empty 가 page-bottom 까지 채우는 경우 page break 신호 신뢰 | pi=440 break (페이지 22) |

## 2. 페이지 alignment 검증

`hwp3-sample16-hwp5.hwp` vs `hwp3-sample16.hwp` (HWP3 reference, 64 페이지):

| 항목 | 결과 |
|------|------|
| 페이지 수 | **64 ↔ 64** (한컴 정합) ✓ |
| 페이지별 첫 단락 (pi) 일치 | **63/64 = 98.4% perfect** |
| 불일치 | 페이지 33 (1 empty paragraph "(빈)" 차이 — h=13.3px, 시각 무영향) |

## 3. 다른 sample 회귀 sweep

| 파일 | 페이지 수 | 회귀 |
|------|----------|------|
| hwp3-sample10.hwp | 763 | 없음 |
| hwp3-sample14.hwp | 11 | 없음 |
| hwp3-sample16.hwp (HWP3 원본) | 64 | 없음 |
| exam_kor.hwp (일반 HWP5) | 20 | 없음 |
| aift.hwp (일반 HWP5) | 74 | 없음 |
| biz_plan.hwp (일반 HWP5) | 6 | 없음 |

## 4. 빌드/테스트 검증

- `cargo build --release`: 0 errors ✓
- `cargo test --release --lib`: **1303 passed**, 0 failed ✓
- `cargo clippy --release -- -D warnings`: 0 warnings ✓
- WASM 빌드: ✓ (`wasm-pack build --target web --out-dir pkg`)

## 5. 잔존 / Follow-up

### 5-1. 페이지 33 alignment (minor)

HWP3 page 33 ends with pi=535 ("(빈)" h=13.3px). 변환본은 pi=534 (Table 30×4 30x923px) 가 페이지 32 를 거의 가득 채워 pi=535 가 페이지 33 으로 밀림. 변환본의 Table 측정 미세 차이 (HWP3 pi=533 = FullParagraph h=33.7, 변환본 pi=533 = Table h=30.0) 가 누적. 시각 무영향.

### 5-2. variant paragraph 외곽선 overflow (follow-up issue 권장)

변환본 일부 paragraph (pi=441/445/449/450/460/461 등 59 개) 가 PARA_LINE_SEG 를 정상 저장하지 않아 composer fallback 의 CHARS_PER_LINE heuristic 적용 → render 의 누적 y 가 measure 와 미세 drift → 페이지 마지막 paragraph 가 외곽선 9~45px overflow.

시도/검증:
1. **widow rule** (1 line peek 시 paragraph 전체 next page push) — pi=450 도 함께 push → pi=460 cascade overflow → revert
2. **render line_height max_fs 사용** (line_spacing 분리) — 줄간격 압축 시각 회귀 → revert
3. **현재 상태** — 줄간격 정합 우선, 미세 overflow 잔존

권장: paragraph LINE_SEG synthesizing 또는 widow cascade 방지 로직을 별도 task 로 분리 해결.

## 6. Stage 5 진입

- 최종 보고서 작성
- PR 생성
