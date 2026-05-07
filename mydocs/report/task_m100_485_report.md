# 최종 결과 보고서 — Task #485

**제목**: synam-001.hwp 15·20·21 페이지 분할 표 셀 마지막 줄 클립핑 (Task #431 잔여)
**브랜치**: `local/task485` (분기: `local/devel`)
**마일스톤**: v1.0.0 (M100)
**작성일**: 2026-05-07
**관련 이슈**: #485 (본), #431 (선행 정정), #362 (회귀 origin)

---

## 1. 요약

`samples/synam-001.hwp` 15·20·21 페이지의 RowBreak 분할 표(PartialTable)에서 마지막 줄이 본문 영역 하단 경계와 시각적으로 겹치며 글자 descender 가 클립핑되던 결함을 정정.

본질 분석 결과 **두 개의 분리된 버그**가 결합된 것으로 확인:

- **Bug-1 (out-of-order)**: `compute_cell_line_ranges` 의 inner break 가 outer 루프를 빠져나오지 않아, 셀 마지막 단락(line_spacing 제외로 line_h 작아짐)이 abs_limit 안에 fit 하여 시각 순서 역전 + 클립
- **Bug-2 (boundary epsilon)**: `line_end_pos > abs_limit` 의 boundary 케이스에서 ~0~2px 차이로 fit 하면 cell-clip-rect bottom 침범

## 2. 정정 영역

`src/renderer/layout/table_layout.rs` 의 `compute_cell_line_ranges` 단일 함수에서 정정.

### 2.1 변경 요약

```rust
// [Task #485 Bug-2] boundary epsilon 도입
const SPLIT_LIMIT_EPSILON: f64 = 2.0;
let effective_limit = if has_limit { content_offset + content_limit - SPLIT_LIMIT_EPSILON } else { 0.0 };

// [Task #485 Bug-1] limit_reached 플래그 도입
let mut limit_reached = false;

for (pi, ...) in composed_paras.iter().enumerate() {
    // 한도 초과 후 후속 단락 강제 차단 (시각 순서 보존)
    if limit_reached {
        let visible_count = if line_count == 0 { 0 } else { line_count };
        result.push((visible_count, visible_count));
        continue;
    }
    ...

    // atomic 분기
    let exceeds_limit = has_limit && para_end_pos > effective_limit && !bigger_than_page;
    if was_on_prev || exceeds_limit {
        result.push((visible_count, visible_count));
        if exceeds_limit { limit_reached = true; }
    } else { ... }

    // 일반 line 분기
    for (li, line) in comp.lines.iter().enumerate() {
        ...
        if has_limit && line_end_pos > effective_limit {
            limit_reached = true;
            break;
        }
        ...
    }
}
```

### 2.2 변경 파일

- `src/renderer/layout/table_layout.rs` — `compute_cell_line_ranges` 함수 (3 hunk)

기타 파일 변경 없음.

## 3. 검증 결과

### 3.1 본 결함 해소

| 페이지 | 결함 | 정정 후 |
|--------|------|---------|
| `synam-001.hwp` p15 | pi=84 (cell-last, line_h=14.667) slip → 클립 | slip 차단 → 클립 해소 ✓ |
| `synam-001.hwp` p20 | pi=169 (cell-last, line_h=13.333) slip → 클립 | slip 차단 → 클립 해소 ✓ |
| `synam-001.hwp` p21 | pi=108 (gap=0.947 NEAR) → 클립 | epsilon 마진으로 차단 → 클립 해소 ✓ |

### 3.2 회귀 점검

| 회귀 영역 | 결과 |
|-----------|------|
| Task #362 (kps-ai.hwp p56/67/68/69/70/72/73) | 정상 (의도 보존) |
| Task #431 (synam-001 빈 페이지 미발생) | 정상 (의도 보존) |
| Task #398 (rowspan 보호 블록) | 정상 |
| Task #474 (RowBreak 표 분할) | 정상 |
| aift.hwp 분할 표 (p10~p14) | 정상 |

### 3.3 자동 테스트

```
cargo test --release: 1199 passed, 0 failed, 2 ignored
```

### 3.4 전 샘플 export-svg

```
samples/*.hwp (155개): 0 fail
```

## 4. 정정 단계 이력

| 단계 | 내용 | 산출물 | 커밋 |
|------|------|--------|------|
| Stage 1 | 본질 정밀 측정 (trace 기반) — 2개 버그 식별 | `working/task_m100_485_stage1.md` | `ac29d1c5` |
| Stage 2a | Bug-1 정정 (limit_reached 플래그 + outer break) | `working/task_m100_485_stage2a.md` | `ea969cdc` |
| Stage 2b | Bug-2 정정 (epsilon 2.0px) | `working/task_m100_485_stage2b.md` | `2ce1aae3` |
| Stage 3 | 회귀 검증 (1199 cargo test + 시각 + 전 샘플) | `working/task_m100_485_stage3.md` | `82f605f8` |
| Stage 4 | 최종 보고 (본 문서) | `report/task_m100_485_report.md` | (현재 커밋) |

## 5. 후보 A·D 미적용 결정

수행계획서 §5 의 후보 중:

- **후보 A** (typeset `engine.rs:1708/1718/1759/1770` 의 `split_end_limit = avail_content` 에 epsilon 차감): **미적용** — 본질이 layout 측에서 정정됨. typeset 측은 정확한 cell-clip-rect 정합을 유지. 적용 시 이중 마진 위험.
- **후보 D** (layout vpos correction 단계 본문 영역 침범 시 드롭): **미적용** — 선행 collapse 회귀 사례 (`typeset_layout_drift_analysis.md` §회귀 원인 체인 6항). Stage 2a+2b 만으로 충분.

## 6. 잔여 위험 / 미해결 항목

### 6.1 epsilon 임의성

`SPLIT_LIMIT_EPSILON = 2.0px` 는 측정 데이터 (p15·p20·p21 의 gap 0.947~1.973) 기반 휴리스틱. 폰트 크기/메트릭 변화에 따라 적합성이 달라질 수 있음. 본 한국어 폰트(맑은 고딕) 기본 케이스에 적합.

### 6.2 layout drift 본질

`typeset_layout_drift_analysis.md` 의 "단일 모델 통합" (typeset/layout 의 height 측정 모델 일치) 은 본 정정 영역 밖. 본 epsilon 은 그 구조적 본질의 임시 흡수. 향후 별도 이슈로 추적.

### 6.3 PDF 적재량 차이

본 정정으로 일부 페이지에서 마지막 1~2줄이 다음 페이지로 밀림. 한컴 PDF 와 페이지 분할이 정확히 일치하지 않을 수 있으나, 본 결함 해소 측면에서 의도된 동작 (descender 침범 방지).

## 7. 결론

이슈 #485 의 결함 해소 완료. Bug-1 (out-of-order) + Bug-2 (boundary epsilon) 두 본질을 layout 측에서 정정. 회귀 없음, 1199 cargo test 통과, 전 샘플 export 성공.

이슈 #485 close 가능. (작업지시자 최종 승인 후)

---

## 8. 참고 자료

- 수행계획서: `mydocs/plans/task_m100_485.md`
- 구현 계획서: `mydocs/plans/task_m100_485_impl.md`
- Stage 보고서: `mydocs/working/task_m100_485_stage{1,2a,2b,3}.md`
- 관련 트러블슈팅: `mydocs/troubleshootings/typeset_layout_drift_analysis.md`, `typeset_fit_accumulation_drift.md`
- GitHub Issue: https://github.com/edwardkim/rhwp/issues/485
