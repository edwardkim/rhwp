# Task #712 구현 계획서

**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**브랜치**: `local/task712`
**수행 계획서**: [`task_m100_712.md`](task_m100_712.md)
**작성일**: 2026-05-08

---

## 1. TDD 전략

### 1.1 RED 테스트 (Stage 1)

**파일**: `tests/regression_task712.rs` (신규)
**의도**: pi=586 12x5 표 외곽 상단 y ≥ pi=585 외곽 하단 y 단언. 현 빌드에서 FAIL.

```rust
//! Task #712 회귀 테스트 — wrap=TopAndBottom 음수 vert offset 표가
//! 직전 inline TAC 표 안쪽으로 침범하지 않는지 검증.

use rhwp::parser::hwp5::parse_hwp;
use rhwp::renderer::layout::Layouter;
// (실제 import 는 기존 회귀 테스트 패턴 참고)

#[test]
fn task712_pi586_table_does_not_invade_pi585_outer_box() {
    let path = "samples/2022년 국립국어원 업무계획.hwp";
    let doc = parse_hwp(path).expect("parse");
    // page 31 (0-indexed 30) 의 RenderTree 빌드
    let tree = build_render_tree(&doc, 30);

    let pi585 = locate_table_outer_box(&tree, 585, 0);  // 1x3 TAC 제목 표
    let pi586 = locate_table_outer_box(&tree, 586, 0);  // 12x5 일정 표

    // pi=585 외곽 하단 y
    let pi585_outer_bottom = pi585.y + pi585.height;
    // pi=586 외곽 상단 y
    let pi586_outer_top = pi586.y;

    assert!(
        pi586_outer_top >= pi585_outer_bottom - 0.5,  // 0.5 px 허용 오차 (rounding)
        "pi=586 12x5 표가 pi=585 1x3 표 안쪽으로 침범. \
         pi585 outer_bottom={:.2} pi586 outer_top={:.2} 침범={:.2} px",
        pi585_outer_bottom, pi586_outer_top,
        pi585_outer_bottom - pi586_outer_top,
    );
}
```

**대안 (간소화)**: 기존 `re_sample_gen.rs` / `regression_*.rs` 의 골든 SVG 비교 패턴 활용 가능. 본 케이스는 좌표 직접 단언이 더 본질적이므로 신규 회귀 테스트로 작성.

### 1.2 GREEN 단계 (Stage 3)

가설 H1/H2/H3 중 분석(Stage 2) 으로 확정된 root cause 를 핀포인트 정정.

---

## 2. 분석 도구 (Stage 2)

### 2.1 디버그 인스트루먼트

기존 `RHWP_VPOS_DEBUG=1` 환경변수가 `layout.rs:1539-1546` 에 존재. PartialTable 경로에서는 출력되지 않으므로 추가 인스트루먼트 필요:

**추가 위치**:
1. `layout.rs:2680` (`layout_partial_table_item` 진입) — `pt_y_start, y_offset, vertical_offset` 출력
2. `layout.rs:1488` (`lazy_base` 산출) — `prev_pi, prev_vpos_end, y_delta_hu, lazy_base` 출력
3. `layout.rs:1504-1525` (VPOS_CORR `vpos_end` 결정) — PartialTable 진입 시 호출 여부 명시

**환경변수**: `RHWP_TASK712_DEBUG=1` (분석 한정, GREEN 후 제거)

### 2.2 가설 검증 절차

1. RHWP_TASK712_DEBUG=1 로 page 31 SVG 출력 → stderr 트레이스 수집
2. y_offset 진행 시퀀스 표 작성:
   - pi=585 진입 / 종료 / pi=586 진입 / pt_y_start / layout_partial_table 진입 / 표 렌더 시작 y
3. **15.94 px 위로 어긋나는 지점** 핀포인트
4. 가설 H1/H2/H3 중 매칭되는 것 또는 신규 가설 H4 도출

---

## 3. 단계별 산출물

| Stage | 파일 / 변경 | 검증 |
|-------|-----------|------|
| 0 | `mydocs/plans/task_m100_712.md` ✅ , `task_m100_712_impl.md` (본 문서) | 작성 + 커밋 |
| 1 (RED) | `tests/regression_task712.rs` 신규 | `cargo test task712 -- --nocapture` FAIL 확인 |
| 2 (분석) | (선택) `RHWP_TASK712_DEBUG` 인스트루먼트 일시 추가 | y_offset 진행 트레이스 수집 + 가설 확정 |
| 3 (GREEN) | `src/renderer/layout.rs` 정정 | RED 테스트 PASS, 트레이스에서 침범 0 확인 |
| 4 (회귀) | `cargo test --release` 1221+ 테스트 + `tests/golden_svg/` | 회귀 0 |
| 5 (광범위) | `samples/` 전수 SVG 비교 (현재 vs 패치 후), 페이지 수 변경 추적 | 의도되지 않은 변경 0 |
| 6 (보고) | `mydocs/working/task_m100_712_stage1.md` 등 단계별 + `mydocs/report/task_m100_712_report.md` | close #712 |

---

## 4. Stage 별 상세

### Stage 1 (RED)

**작업**:
1. `tests/regression_task712.rs` 작성 — 위 1.1 의 단언
2. `cargo test --test regression_task712` 실행 → FAIL (현재 침범 15.94 px)
3. 단계별 보고서 `mydocs/working/task_m100_712_stage1.md` 작성
4. 커밋: `Task #712 Stage 1 (RED): pi=585/586 침범 회귀 테스트 — 현재 FAIL 확인`

**기준**: 테스트가 실제 침범량을 정확히 잡는지 확인 (assert message 출력에서 ~16 px 침범 확인).

### Stage 2 (분석)

**작업**:
1. `layout.rs` 에 `RHWP_TASK712_DEBUG` 인스트루먼트 일시 추가 (Stage 3 정정 후 제거)
2. trace 수집 + 가설 검증
3. root cause 확정
4. 보고서 `mydocs/working/task_m100_712_stage2.md` 작성
5. (Stage 3 직전이므로 단독 커밋 불필요 — 인스트루먼트 코드는 Stage 3 와 함께 정리)

### Stage 3 (GREEN)

**작업**:
1. root cause 핀포인트 정정 (최소 외과적 — 한 함수 내 변경 선호)
2. `cargo test --test regression_task712` PASS 확인
3. SVG 시각 비교: 침범 0 확인
4. 인스트루먼트 정리 (디버그 print 제거)
5. 단계별 보고서 `mydocs/working/task_m100_712_stage3.md`
6. 커밋: `Task #712 Stage 3 (GREEN): {root cause} 정정 — pi=586 침범 0`

### Stage 4 (회귀)

**작업**:
1. `cargo build --release && cargo test --release` 전체
2. 골든 SVG 회귀 (`tests/golden_svg/` 등 기존 회귀 자산)
3. 보고서 `mydocs/working/task_m100_712_stage4.md`
4. 커밋: `Task #712 Stage 4: 회귀 검증 (1221+ 테스트 통과, 골든 SVG 회귀 0)`

### Stage 5 (광범위)

**작업**:
1. `samples/` 디렉터리 SVG 전수 생성 (현재 빌드)
2. 페이지 수 / 의심 케이스 (wrap=TopAndBottom + 음수 vert offset) 횡단 비교
3. 보고서 `mydocs/working/task_m100_712_stage5.md`
4. 커밋: `Task #712 Stage 5: 광범위 회귀 검증`

### Stage 6 (최종)

**작업**:
1. 최종 결과 보고서 `mydocs/report/task_m100_712_report.md`
2. closes #712
3. 커밋: `Task #712 Stage 6: 최종 보고서 + closes #712`
4. (작업지시자 승인 후) `local/task712` → `devel` merge
5. (작업지시자 승인 후) `pr-task712` 브랜치 생성, `stream/devel` 으로 PR

---

## 5. 위험 완화

| 위험 | 완화 |
|------|------|
| VPOS_CORR 정정 시 Task #412/#643/#470 회귀 | Stage 4 1221 테스트 + 골든 SVG 회귀 검증 + Stage 5 횡단 |
| 음수 vert offset 가드 추가가 다른 케이스 침범 | Stage 5 에서 wrap=TopAndBottom + vert<0 케이스 전수 검사 |
| 인스트루먼트 잔존 | Stage 3 종료 시 제거 확인, Stage 4 빌드에서 디버그 출력 0 |

---

## 6. 비범위

- PartialTable split 알고리즘 (rows 분할) 자체 변경
- pi=586 의 raw vpos=69196 인코딩 의미 재해석 (HWP 스펙 차원의 별도 조사)
- pi=585 의 인라인 TAC 표 렌더 (정상 동작)

---

## 7. Out of scope

- HWPX 동일 케이스 검증 — 본 결함은 HWP5 바이너리에서 보고된 것이며, HWPX 변환본은 별도 검증.
- 다른 페이지(p32 의 1x3 + PartialTable 연속 케이스) — 동일 root cause 일 가능성이 높지만, 정정 적용 후 자연 해소되는지 Stage 5 에서 확인.
