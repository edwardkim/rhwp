# Task #540 구현 계획서

**제목**: 빈 paragraph + 음수 ls 의 advance floor 적용 (가설 H2 채택)
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540
**수행계획서**: `mydocs/plans/task_m100_540.md`
**Stage 0 보고서**: `mydocs/working/task_m100_540_stage0.md`

---

## 1. 가설 H2 채택 근거

작업지시자가 "줄간격이 좁다" 보고 → 한컴이 더 넓게 출력 = rhwp(33.01 px) 보다 큰 값.
가장 직접적인 가설은 **H2: 한컴이 빈 paragraph 의 음수 ls 를 floor (무시) 하여 advance = lh**.

기대 gap (한컴 동작):
- 60% 빈 paragraph: advance = 1100 HU (음수 ls -440 무시) → 페이지 2 [4~6] gap = 1816 + 1100 = **2916 HU = 38.88 px**
- 95% 빈 paragraph: advance = 1100 HU (음수 ls -56 무시) → 동일 38.88 px
- 65% 빈 paragraph: 동일 38.88 px

→ 9곳 모두 gap 통일되어 38.88 px 기대.

## 2. 변경 대상

| 파일 | 변경 | LOC |
|------|------|-----|
| `src/renderer/composer/line_breaking.rs` | 음수 ls floor 가 이미 `.max(0)` 으로 적용됨 → 추가 변경 없음 | 0 |
| `src/renderer/layout.rs` | vpos correction 의 `prev_vpos_end` 산출에서 음수 ls 무시 | +5 |
| `src/renderer/height_measurer.rs` | line advance 식에서 음수 ls 무시 (필요 시) | +5 |
| `src/renderer/layout/integration_tests.rs` | TDD 테스트 추가 (9곳 중 대표) | +60 |
| `mydocs/working/task_m100_540_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_540_report.md` | 최종 보고서 | 신규 |

## 3. 핵심 변경 (가설 H2)

### 3.1 본질

vpos correction 이 IR 의 `LINE_SEG.vertical_pos` 를 따라가는데, IR 의 vpos 자체가 음수 ls 반영 (HWP 파일에 그렇게 저장). 한컴은 렌더링 시 이를 floor 처리.

### 3.2 fix 위치 (Stage 1 진단으로 확정)

**가설 1**: `layout.rs` 의 vpos correction `prev_vpos_end` 산출 시 음수 ls 를 0 으로 floor.
```rust
let prev_vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing.max(0);
```

**가설 2**: 음수 ls 를 가진 빈 paragraph 의 `LINE_SEG.line_height + line_spacing` 을 `line_height` 로 강제 (즉 ls floor).

### 3.3 적용 범위

빈 paragraph 만 한정 (text_len == 0 or cc <= 1) — 일반 paragraph 의 음수 ls 는 보존.
근거: synam-001.hwp 의 음수 ls 57건 중 대다수가 일반 paragraph (셀 내부 등). 빈 paragraph 가 아닌 음수 ls 는 의도적이므로 보존.

## 4. 단계 분할

### Stage 1 — TDD + 진단

1. 페이지 2 [4~6] gap = 38.88 px 기대 통합 테스트 추가 (현재 실패).
2. 9곳 중 ls=-440 (60%), ls=-384 (65%), ls=-56 (95%) 대표 케이스 통합 테스트 추가.
3. fix 위치 정밀 진단 — `layout.rs` vpos correction 의 prev_vpos_end 가 정답인지 확인.
4. Stage 1 보고서 + 커밋.

### Stage 2 — Fix 적용

1. 진단 결과 기반 fix.
2. 빈 paragraph 한정 가드 (text_len 또는 cc 조건).
3. 단위 테스트 통과.
4. Stage 2 보고서 + 커밋.

### Stage 3 — 광범위 회귀 검증

1. `cargo test` 전체 통과.
2. **synam-001.hwp 광범위 회귀 검증** (음수 ls 57건 영향 여부).
3. exam_math, 21_언어_기출 등.
4. 회귀 발견 시:
   - 빈 paragraph 가드 정밀화 (cc <= 1 or text.is_empty())
   - 또는 본 task 정정 보류 + 별도 분석
5. Stage 3 보고서 + 최종 보고서.

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| synam-001 의 음수 ls 57건 중 빈 paragraph 가 아닌 케이스 회귀 | 매우 큼 | 빈 paragraph 한정 가드 (text_len ≤ 1) |
| Task #537/#539 회귀 (vpos correction 변경) | 큼 | 가드 조건 정밀화 + 기존 테스트 회귀 검증 |
| 한컴 측정값과 실제 동작 불일치 (가설 H2 오류) | 큼 | Stage 3 회귀 발견 시 본 task 정정 보류 |

## 6. 검증 명령

```bash
cargo build --release
cargo test --release --lib

./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag540 -p 1
./target/release/rhwp export-svg samples/synam-001.hwp -o /tmp/diag540/synam

RHWP_VPOS_DEBUG=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -o /tmp/diag540 -p 1 2>&1 | grep VPOS_CORR
```

## 7. 커밋 단위

- Stage 1: "Task #540 Stage 1: 가설 H2 TDD 단위테스트 + 진단"
- Stage 2: "Task #540 Stage 2: 빈 paragraph 음수 ls floor 적용 (가설 H2)"
- Stage 3: "Task #540 Stage 3: 광범위 회귀 검증 + 최종 보고서"

`closes #540` 는 Stage 3 마지막.

---

승인 후 Stage 1 부터 시작합니다.
