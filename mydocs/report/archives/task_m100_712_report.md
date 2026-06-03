# Task #712 최종 결과 보고서

**제목**: 2022 국립국어원 업무계획 p31 — wrap=TopAndBottom 음수 vert offset 12x5 표가 직전 inline TAC 1x3 제목 표 안쪽으로 침범 (~16 px)
**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**브랜치**: `local/task712`
**작업 기간**: 2026-05-08 (단일 세션)
**최종 상태**: ✅ closes #712

---

## 1. 결함 요약

`samples/2022년 국립국어원 업무계획.hwp` 36 페이지(page_num=34) 상단:
- `pi=585` 1x3 인라인 TAC 제목 표("붙임 / / 과제별 추진일정") 와
- `pi=586` 12x5 일정 표(`wrap=TopAndBottom`, `vert=문단(-1796 HU)`) 가
- 시각적으로 **~12-16 px 겹침** (12x5 표가 1x3 표 안으로 침범)

PDF 권위 자료(`pdf/2022년 국립국어원 업무계획-2022.pdf`) 와 시각 불일치.

## 2. Root cause

### 핵심 메커니즘

`HwpUnit = u32` (`src/model/mod.rs:21`) 라서 음수 `vertical_offset` 값(예: -1796 HU) 이 비트표현 그대로 unsigned 양수(0xFFFFF8FC = 4294965500u32)로 저장됨.

`src/renderer/layout/table_partial.rs:62-71` 의 게이트:
```rust
&& table.common.vertical_offset > 0          // u32 비교 → 음수도 통과
{
    y_start + hwpunit_to_px(table.common.vertical_offset as i32, ...)
    // ↑ as i32 캐스트에서 음수 -1796 → -23.95 px → 표가 위로 점프
}
```

→ 게이트가 음수 차단 의도였으나 unsigned 비교로 무효화. PartialTable 표가 음수만큼 위로 이동하여 직전 인라인 TAC 표 영역 침범.

### 비-Partial 경로와의 차이

`src/renderer/layout/table_layout.rs:1069+` 의 동일 분기에는 `pushed = raw_y.max(y_start)` 클램프가 있어 음수 offset이 무력화됨. PartialTable 경로에는 클램프가 없어 본 결함 표면화.

## 3. 정정

`src/renderer/layout/table_partial.rs:62-71` 와 `src/renderer/layout.rs:2673-2685` 두 곳의 게이트를 **signed i32 비교**로 변경:

```rust
// Before (u32 비교, 음수 비트표현 통과)
&& table.common.vertical_offset > 0

// After (i32 비교, 양수만 통과)
let vert_off_signed = table.common.vertical_offset as i32;
&& vert_off_signed > 0
```

게이트 통과 시 `hwpunit_to_px(vert_off_signed, ...)` 도 동일 signed 변수 사용.

**효과**: 양수 offset → 변경 없음 (그대로 적용), 0/음수 → y_start 유지 (비-Partial 경로의 클램프 효과와 동등).

## 4. 검증

### 회귀 테스트 (TDD)

`tests/issue_712.rs` 신규: pi=586 외곽 상단 y ≥ pi=585 외곽 하단 y - 0.5 px 단언

| 단계 | 결과 |
|------|------|
| Stage 1 RED | FAIL — 침범 12.17 px |
| Stage 3 GREEN | PASS — 침범 0 |

### 측정 비교 (page index 35)

| 항목 | Before | After |
|------|--------|-------|
| pi=585 1x3 cell | [98.25..137.11] | 동일 |
| pi=586 12x5 표 | [124.93..1004.31] | **[148.88..1028.25]** |
| pi=585 cell 하단 → pi=586 시작 간격 | -12.17 px (침범) | **+11.77 px** (line_spacing 8.0 + outer_margin_bottom 3.77 정확 일치) |

### 회귀 (Stage 4)

```
$ cargo test --release
test result: ok. 1252 passed; 0 failed; 5 ignored
```

### 광범위 회귀 (Stage 5)

181 샘플 페이지 수 비교:
```
$ diff /tmp/task712_pagecount_before.txt /tmp/task712_pagecount_after.txt
(0 lines)
```

→ **페이지 수 회귀 0/181**.

분할 연결 페이지(37, page_idx 36) `is_continuation=true` 분기는 가드(`!is_continuation`) 로 인해 무영향 확인.

## 5. 변경 파일

| 파일 | 추가 | 삭제 | 비고 |
|------|------|------|------|
| `src/renderer/layout.rs` | +5 | -2 | `pt_y_start` 게이트 signed |
| `src/renderer/layout/table_partial.rs` | +9 | -3 | gate signed + 주석 |
| `tests/issue_712.rs` | +96 | 0 | 회귀 테스트 |
| `mydocs/plans/task_m100_712.md` | +202 | 0 | 수행 계획서 |
| `mydocs/plans/task_m100_712_impl.md` | +174 | 0 | 구현 계획서 |
| `mydocs/working/task_m100_712_stage1.md` | +52 | 0 | Stage 1 보고서 |
| `mydocs/working/task_m100_712_stage2.md` | +153 | 0 | Stage 2 보고서 |
| `mydocs/working/task_m100_712_stage3.md` | +66 | 0 | Stage 3 보고서 |
| `mydocs/working/task_m100_712_stage4.md` | +99 | 0 | Stage 4-5 보고서 |
| `mydocs/report/task_m100_712_report.md` | (본 문서) | | 최종 보고서 |

순수 코드 변경: **14 라인** (주석 포함). 기능 변경: **2 게이트의 비교 연산자 변환**.

## 6. 단계별 진행 (Stage timeline)

| Stage | 산출물 | 결과 |
|-------|--------|------|
| 0 | 수행 + 구현 계획서 | 6 stage 정의 |
| 1 (RED) | `tests/issue_712.rs` | FAIL 확인 (침범 12.17 px) |
| 2 (분석) | `RHWP_TASK712_DEBUG` 트레이스 | Root cause 확정 (가설 H3) |
| 3 (GREEN) | 게이트 signed 정정 | PASS (침범 0) |
| 4 (회귀) | `cargo test --release` | 1252/1252 pass |
| 5 (광범위) | 181 샘플 페이지 수 횡단 | 회귀 0 |
| 6 (보고) | 본 문서 | closes #712 |

## 7. 후속

### 자연 해소된 부수 결함

페이지 37 (page_num=35) 의 `pi=586 ci=1 1x3 inline TAC 표` 도 동일 paragraph 의 LINE_SEG vpos-reset 패턴으로 영향 받을 가능성이 있었으나, `is_continuation=true` 분기는 가드로 무영향 → 별도 정정 불필요.

### 향후 모니터링

- 신규 샘플 추가 시 음수 `vertical_offset` 케이스 확장 모니터링
- HwpUnit 타입 자체의 signed/unsigned 통합 정리 검토 (별도 광범위 리팩토링 — 본 타스크 비범위)

## 8. 결론

`HwpUnit=u32` 와 `vertical_offset > 0` unsigned 비교의 결합으로 발생한 음수 offset 표 위치 결함을 signed 비교 게이트 정정으로 해소. 회귀 0, 패치 라인 14 (기능 2 라인). PDF 권위 자료와 시각 정합 회복.

**closes #712**
