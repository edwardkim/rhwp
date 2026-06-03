# Task #321 v4 구현 계획서 — col 0 drift 진단

## 전제

- 21_언어 page 1 col 0 drift = +85.8 px (vs HWP LINE_SEG vpos 누적)
- 본 stage 는 **진단·정량화 전용**. 동작 변경 코드 추가 금지.
- 모든 진단 출력은 `RHWP_TYPESET_DRIFT` 환경변수 가드 하에만 활성화.

## Stage 5a — 진단 로그 강화

**대상 파일**: `src/renderer/typeset.rs`

**현행** (`typeset_paragraph` 진입부, line 552-571):

```rust
if std::env::var("RHWP_TYPESET_DRIFT").is_ok() {
    // pi, col, cur_h, avail, fmt_total, vpos_h, diff, first_vpos, last_vpos 출력
}
```

**추가 항목**:

1. **per-paragraph 분해**:
   - `spacing_before_px`: ParaShape spacing_before → px
   - `spacing_after_px`: ParaShape spacing_after → px
   - `border_padding_px`: border_fill_id != 0 일 때 추가되는 padding (top/bottom 합)
   - `line_count`, `line_total_px` (lines 합)
   - `controls_top_bottom_h`: TopAndBottom wrap 인 표/도형 의 reserve 기여

2. **per-line 분해** (조건부, `RHWP_TYPESET_DRIFT_LINES=1` 일 때만):
   - line_idx, line_height_px, line_spacing_px, vpos_step (LINE_SEG 의 vpos 차분)

3. **출력 포맷**: 1 줄에 압축, grep 친화 (key=value 공백 구분).

```
TYPESET_DRIFT_PI: pi=7 col=0 sb=7.6 sa=0.0 bp=0.0 lines=8 line_sum=146.7 ctrl_h=0.0 fmt_total=146.7 vpos_h=146.7 diff=+0.0
```

**CR 포인트**: 함수형 helper 1 개 추가 (`fn drift_breakdown(...)`) — 기존 로직 변경 없음.

**검증**:
- `cargo build --release` 성공
- `RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 2>&1 | grep TYPESET_DRIFT_PI | head` 으로 pi 0..21 출력 확인
- `RHWP_TYPESET_DRIFT` 미설정 시 출력 없음 확인

## Stage 5b — 측정 수집

**스크립트 / 명령** (커밋 안 함, 작업 메모용):

```bash
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -p 0 -o /tmp/drift/ \
  2> /tmp/drift_log.txt
grep "TYPESET_DRIFT_PI" /tmp/drift_log.txt > /tmp/drift_pi.txt
```

**작성 항목** (`mydocs/working/task_m100_321_stage5.md` 부분):

| pi | sb | sa | bp | line_sum | ctrl_h | fmt_total | vpos_h | diff | 누적 diff |
|----|-----|-----|-----|---------|--------|-----------|--------|------|----------|
| 0  | … | … | … | … | … | … | … | … | … |
| …  |   |   |   |   |   |   |   |   |   |
| 9  |   |   |   |   |   |   |   |   | **≈ +85.8** |

**카테고리 기여 합계**:
- Σ(sb+sa diff): 스페이싱
- Σ(bp diff): border padding
- Σ(line_sum - vpos_h, where ctrl=0): 라인 메트릭
- Σ(ctrl_h diff): 표/도형 reserve

**검증**:
- diff 합계 ≈ +85.8 px (±2 px)
- dump-pages 의 col 0 used (1233.5) - hwp_used (1147.7) 와 일치

## Stage 5c — 가설 정리 + 보고서

**산출**: `mydocs/working/task_m100_321_stage5.md`

구조:
1. 측정 환경
2. 측정 결과 표 (5b 결과)
3. 카테고리별 기여 분해
4. **dominant 항목** 식별 (가장 큰 기여)
5. 후속 수정 stage 가설 후보 (수정 안 함, 가설만)

**검증**:
- 보고서 작성 후 `git status` 로 의도 외 수정 없음 확인
- `cargo test` 통과 (5a 의 진단 로그가 test 출력에 영향 없음 — env-gated)

## 회피 사항

- ✋ Layout 동작 수정 금지 — 진단만
- ✋ default 출력 변경 금지 — env-gate 필수
- ✋ 다른 샘플 영향 검증은 stage 5 종료 시점 한 번 (cargo test + 4 개 sample 페이지 수)

## 승인 요청

본 구현 계획서 승인 후 Stage 5a 부터 순차 진행.
