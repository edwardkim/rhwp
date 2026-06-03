# Task #643: 페이지 분할 드리프트 정정 — 구현계획서

## Stage 0 진단 결과 (선행 완료)

`RHWP_TYPESET_DRIFT=1 rhwp dump-pages "samples/2022년 국립국어원 업무계획.hwp" -p 5` 로그로 정밀 진단 완료.

### 핵심 데이터

```
TYPESET_DRIFT_PI: pi=80 sb=4.0 lines=2 lh_sum=40.0 ls_sum=24.0 trail_ls=12.0
                  fmt_total=68.0 vpos_h=52.0 cur_h=870.7 avail=923.5
PartialParagraph pi=80  lines=0..1  vpos=65943..68343
```

### Root Cause 확정

`pagination/engine.rs:846-852` 의 fit 루프가 `mp.line_advance(li) = lh + ls` 를 누적:

```rust
for li in cursor_line..seg_end {
    let content_h = mp.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line { break; }
    cumulative += mp.line_advance(li);  // ← lh + 트레일링 ls 모두 누적
    end_line = li + 1;
}
```

**버그**: 세그먼트 마지막 줄까지 배치한 후에도 `cumulative` 에 트레일링 `line_spacing` 이 더해진 상태로 남음.
- pi=80 line 0 만 배치: cumulative = 32 (= 20 + 12), 실제 필요 = 20
- pi=80 line 0,1 모두 배치 검사: cumulative + line_height(1) = 32 + 20 = 52 vs avail_for_lines = 48.8 → break

**산술 검증**:
- HWP 본문 높이: 933.5px, pi=80 시작 cur_h=870.7px, sb=4.0
- pi=80 line 1 끝 위치 (정상 산식): 870.7 + 4 + 32 + 20 = **926.7px ≤ 933.5px ✓**
- 그러나 우리 산식 (트레일링 ls 포함): 870.7 + 4 + 32 + 32 = 938.7px > 933.5px ✗
- 게다가 `LAYOUT_DRIFT_SAFETY_PX=10` 안전마진까지 적용 → avail=923.5 → 한 줄도 더 못 들어감

### 두 정정 영역

1. **fit 산식 정정**: 마지막 줄 fit 검사 시 `line_height` 만 사용 (트레일링 ls 제외) — 표준 레이아웃 관행
2. **안전마진 재평가**: `LAYOUT_DRIFT_SAFETY_PX=10` 은 fit 산식 버그의 band-aid 였음. fit 정정 후 축소/제거 검토

## 구현 단계

### Stage 1: TDD — 실패 테스트 먼저

**대상 파일**: `tests/pagination_drift_test.rs` (신규)

**시나리오**:
1. `samples/2022년 국립국어원 업무계획.hwp` 파싱
2. 페이지 6 (global_idx=5) 의 마지막 항목 검사
3. **기대 (정정 후)**: pi=80 가 `FullParagraph` 또는 `PartialParagraph { end_line: 2 }` 로 페이지 6 에 들어감
4. **현재 (버그)**: `PartialParagraph { para_index: 80, start_line: 0, end_line: 1 }` 로 line 1 누락

테스트는 처음에는 RED 상태로 커밋.

### Stage 2: fit 산식 정정 (좁은 수술)

**대상 파일**: `src/renderer/pagination/engine.rs`

**위치**: `paginate_paragraph_with_breaks` 의 줄 단위 배치 루프 (846-852)

**수정 전략**:

```rust
// AS-IS
for li in cursor_line..seg_end {
    let content_h = mp.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line { break; }
    cumulative += mp.line_advance(li);
    end_line = li + 1;
}

// TO-BE: 마지막 줄은 lh 만 차지 (트레일링 ls 제외)
for li in cursor_line..seg_end {
    let content_h = mp.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line { break; }
    cumulative += if li + 1 < seg_end {
        mp.line_advance(li)   // 다음 줄로 advance
    } else {
        mp.line_heights[li]   // 마지막 줄: 자체 높이만
    };
    end_line = li + 1;
}
```

`part_line_height` 도 동일 산식 적용. `part_height = sp_b + part_line_height + part_sp_after` 가 실제 점유 높이가 되도록.

**부수 효과**: `current_height` 누적도 감소 → 다음 페이지 첫 항목 위치도 영향. 그러나 이는 정정 방향 (drift 제거).

### Stage 3: 안전마진 축소

**대상 파일**: `src/renderer/typeset.rs:876`

**수정 전략**:
- `LAYOUT_DRIFT_SAFETY_PX` 을 단순 제거하지 않고, **0 으로 축소** 후 회귀 테스트 결과로 결정
- 안전마진 도입 사유 (Task #359/#361 주석) 케이스 회귀 점검 — k-water-rfp p15 등
- 회귀 발생 시 `LAYOUT_DRIFT_SAFETY_PX = 2.0` 등 보수적 축소

### Stage 4: 회귀 검증

1. **본 케이스 시각 판정**: `rhwp export-svg --debug-overlay samples/2022년...hwp` → 페이지 6 line 1 포함 확인
2. **dump-pages 검증**: `rhwp dump-pages ... -p 5` → pi=80 가 `FullParagraph` 또는 `lines=0..2`
3. **164 fixture 회귀**: `cargo test --release` (1,614 페이지)
4. **시각 회귀 의심 케이스 sweep**: PartialParagraph 발생 페이지 자동 추출 → before/after 비교

**회귀 허용 기준**:
- 의도된 정정 (HWP 정합 회복): 허용
- 다른 페이지에서 새로운 부당 분리 발생: 차단 → Stage 3 안전마진 재조정

### Stage 5: 최종 보고서

`mydocs/report/task_m100_643_report.md` 작성:
- 본 케이스 정정 결과
- 회귀 sweep 결과 (정정/회귀 페이지 카운트)
- LAYOUT_DRIFT_SAFETY_PX 최종 값
- WASM 빌드 사이즈
- 향후 모니터링 항목 (Task #321 drift 재진단 권장)

## 위험 요소

1. **회귀 가능성**: 광범위한 페이지 분할 변경 가능 → 의도된 정정과 회귀 구분 어려움
2. **다단/표 내부**: `paginate_multicolumn_paragraph`, `place_table_fits` 등 유사 패턴 존재 가능 (확장 검토)
3. **footnote/header**: 별도 fit 경로 (`typeset.rs:1048`, `1076`) 도 동일 안전마진 사용 → 동기화 필요

## 단계별 커밋 단위

- Stage 1: `Task #643 Stage 1: 회귀 테스트 (RED) — pi=80 page6 line1 누락 캡처`
- Stage 2: `Task #643 Stage 2: pagination fit 산식 정정 (트레일링 ls 제외)`
- Stage 3: `Task #643 Stage 3: LAYOUT_DRIFT_SAFETY_PX 축소`
- Stage 4: `Task #643 Stage 4: 회귀 검증 + sweep 결과`
- Stage 5: `Task #643 Stage 5: 최종 보고서 + orders 갱신`

## 승인 요청

본 구현계획서 승인 후 Stage 1 (실패 테스트 작성) 부터 진행합니다.
