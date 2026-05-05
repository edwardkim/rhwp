# Task #604 Stage 5 — HWP3 wrap zone 인코딩 무효화 (한컴 변환 메커니즘 모방)

## 본 단계 본질

Stage 4 시각 판정 중 작업지시자 발견: `hwp3-sample4.hwp` (HWP3 native 40p) 와
`hwp3-sample4-hwp5.hwp` (HWP5 변환본 36p) 의 시각/페이지 수 차이.

본질 진단:
- 한컴 자체가 HWP3 → HWP5 변환 시 `cs/sw=0/full` 로 정정 (cf. `hwp3-sample5-hwp5-v2024.hwp`
  모든 LineSeg `cs=0, sw=51024 = full body width`).
- HWP3 spec 의 cs/sw wrap zone 인코딩이 폰트 크기와 호환되지 않는 케이스가 존재
  (`hwp3-sample5.hwp` pi=75 의 `sw=15564HU=207px` 좁은 영역에 13pt 글자 안 들어감).
- 한컴 변환 메커니즘: wrap zone 인코딩 제거 → 본문 full width 자연 흐름, 그림은
  paper-relative absolute layer 로 별도 그려짐 → 시각 정합.

옵션 D (한컴 변환 메커니즘 모방) 채택 — 본 task 의 cs/sw 정정 본질 일부 폐기 + 시각 정합 우선.

## 변경 영역

### `src/parser/hwp3/mod.rs:1399-1417` 정정

이전 (Stage 3):
```rust
let line_cs_sw = current_zone.and_then(|(cs, sw, _pgy_start, pgy_end)| {
    if pic_wrap_zone.is_some() || linfo.pgy < pgy_end {
        Some((cs, sw))
    } else {
        None
    }
});
```

Stage 5 정정:
```rust
let line_cs_sw = if pic_wrap_zone.is_some() {
    // 앵커 문단: 그림 호스트이므로 cs/sw 인코딩 보존 (그림 위치 영향)
    current_zone.map(|(cs, sw, _, _)| (cs, sw))
} else {
    // 후속 wrap text 문단: cs/sw=0 무효화 → 본문 full width 자연 흐름
    None
};
```

본질:
- 앵커 문단 (그림 호스트, `pic_wrap_zone.is_some()`): cs/sw 인코딩 보존 — 그림 자체
  위치 정합에 영향.
- 후속 wrap text 문단: cs/sw=0 — 본문은 full width 흐름. 그림은 absolute layer 로
  별도 그려짐. 한컴 v2024 변환과 시각 정합.

## 검증 결과

### LineSeg 변경

```
--- 문단 0.75 (wrap text) ---
  Before (Stage 3): cs=35460, sw=15564 — 그림 우측 좁은 영역 (글자 안 들어감)
  After  (Stage 5): cs=0,     sw=0     — full width 흐름 (한컴 정합)
```

### 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 정합) |
| `cargo test --test issue_554` | ✅ 12 passed (HWP3 변환본 정합) |
| `cargo test` 통합 31 | ✅ 모두 통과 |

### 회귀 영역

| 영역 | 결과 |
|------|------|
| `exam_science.hwp` | ✅ 4페이지 / 단 0 items=37 (Task #546 정합) |
| `hwp3-sample.hwp` | ✅ 16페이지 (PR #589 baseline 동일) |
| `hwp3-sample5.hwp` | ✅ 64페이지 (PR #589 baseline 동일) |
| `hwp3-sample4.hwp` | ⚠️ 40페이지 (PR #589 baseline 39 → +1, 본 task 의 다른 변경 영향) |

### 시각 판정 자료

- `output/svg/task604_stage5/hwp3-sample5/hwp3-sample5_{004,008,016,022,027}.svg`
- `output/svg/task604_stage5/hwp3-sample4/hwp3-sample4_{025,026}.svg`

본문 `cs=0/sw=full` 로 흐름 → 그림 영역 그대로 유지 + 본문 자연 흐름 → 한컴 v2024 정합.

## LOC 합계

| 파일 | 변경 |
|------|-----|
| `src/parser/hwp3/mod.rs` | -7 / +20 (Stage 5 본질 정정 + 주석) |
| **소스 합계** | **+13 LOC** |

## Stage 3 vs Stage 5 trade-off

| 영역 | Stage 3 (cs/sw 정확 인코딩) | Stage 5 (한컴 변환 모방) |
|------|------------------------|----------------------|
| pi=75 LineSeg | cs=35460, sw=15564 (HWP3 spec 본질) | cs=0, sw=0 (한컴 변환 정합) |
| 시각 — 그림과 텍스트 | 그림 우측 좁은 영역 (글자 안 들어감) | 본문 full width + 그림 absolute |
| 한컴뷰어 정합 | spec 본질 보존 | **시각 정합** |
| 페이지네이션 | 좁은 영역에 못 들어가서 페이지 +N | 자연 흐름 |

본 task 의 본질 정정 가치 (Issue #604 page 4 그림+텍스트 겹침 정정) 는 Stage 5 에서도 보존
— 그림 좌측에 텍스트가 산재하던 결함이 시각 정정. 다른 본질 (cs/sw 정확 인코딩) 은 한컴
호환성 우선으로 폐기.

## 작업지시자 승인 요청

본 Stage 5 (HWP3 wrap zone 인코딩 무효화) 완료 보고. 본 task 종료 진입 승인 요청 — 최종
보고서 갱신 (Stage 5 추가) + commit.

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- Stage 1~3 보고서: `mydocs/working/task_m100_604_stage{1,2,2b,3}.md`
- Stage 4 최종 보고서 (Stage 5 추가 갱신 예정): `mydocs/report/task_m100_604_report.md`
- Issue: #604
