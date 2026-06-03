# Task #404 Stage 1 — vpos 진단 + 가설 확정

## 작업 내용

`src/renderer/typeset.rs::TypesetState`에 페이지 첫 문단의 vpos를 추적할 필드를 추가하고, `typeset_section` 메인 루프 진입부에 진단 로그를 삽입하여 vpos 기준 overflow가 발생하는 paragraph를 모두 출력했다.

### 변경 사항

1. `TypesetState.page_first_vpos: Option<i32>` 필드 추가
2. `reset_for_new_page()` 에 `page_first_vpos = None` reset 추가 (페이지 전환 시 기준점 무효화)
3. `typeset_section` 메인 루프에 진단 로그 (Stage 2에서 제거 예정):
   - 현재 단의 첫 item의 para_index → first_seg.vpos를 `page_top_vpos`로 사용
   - 각 paragraph의 `vpos_end = first_seg.vpos + sum(line_segs.lh+ls)` 계산
   - `vpos_end > page_top_vpos + body_h_in_hu` 인 경우 모든 값 출력

빌드: `cargo build --release` 통과.

## 진단 결과 (타겟 샘플)

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`

### pi=83 (orphan 대상) — 가설 확정 ✓

| 항목 | 값 |
|------|----|
| page_top_vpos (pi=77 first_vpos) | 532187 |
| body_h_hu | 70012 |
| page_bottom_vpos | 602199 |
| pi=83 first_vpos | 601325 |
| pi=83 para_h_hu | 1760 |
| pi=83 vpos_end | 603085 |
| **overflow** | **886 HU (≈ 0.31 mm)** |
| 누적 height | curr_h=906.6 / avail=933.5 → 26.9px 여유 |
| pi=83 px 높이 | 14.7px |

`906.6 + 14.7 = 921.3 < 933.5` → 누적 기준으로는 fit. 그러나 vpos 기준 886 HU 초과. **HWP 원본은 vpos 기준 페이지 한계 = 602199를 넘는 paragraph를 다음 페이지로 보냄** → pi=83은 10쪽으로 가야 정상.

또한 다음 paragraph pi=84는 Table 7×3 (h=190.9px). pi=83 배치 후 잔여 = 933.5 - 921.3 = 12.2px. 표는 12.2px에 들어갈 수 없으므로 다음 페이지로 push되어 **heading orphan** 발생.

### dump-pages 확인

- 페이지 9: items=8, last item = `pi=83 "(7) 다수 기부자 현황"` (heading 단독)
- 페이지 10: first item = `pi=84 7x3 Table`, 이어서 pi=85 3x3 Table

heading + 두 후속 표가 분리됨이 dump-pages 출력으로 확인됨.

## False Positive 관찰 — Stage 2 전략 재검토 필요

진단 로그에서 pi=83 외에도 **41개 paragraph**가 `vpos overflow > 0` 으로 보고됨. 그러나 대부분은 정상 배치(orphan 아님)이며, 단순히 "vpos overflow → push" 규칙을 적용하면 다수 회귀가 발생한다.

### 대표 false positive

**페이지 8 (pi=57~76)** — TAC 그림(pi=57) + 표(pi=59) + 빈 문단 19개 (pi=58~76):

| paragraph | first_vpos | vpos_end | overflow | 실제 상태 |
|-----------|-----------|----------|----------|-----------|
| pi=62 | 505787 | 507547 | 932 | 페이지 8 정상 배치 |
| pi=63 | 507547 | 509307 | 2692 | 페이지 8 정상 배치 |
| ... | ... | ... | (증가) | ... |
| pi=76 | 530427 | 532187 | 25572 | 페이지 8 정상 배치 |

**원인**: 페이지 8은 TAC 그림(pi=57, vpos 436603→462486+α)과 wrap-around로 인해 vpos↔px 비율이 크게 어긋난다. `hwp_used≈7087.0px` vs 실제 `used=921.7px` (diff=-6165.3px). 그림이 vpos로는 큰 영역을 차지하지만 px로는 그림 자체 높이(269px 정도)만 차지하고 옆/아래 텍스트가 흐른다. 따라서 후속 19줄이 모두 vpos 기준 overflow지만 px 기준 정상 배치.

이 19줄을 모두 다음 페이지로 push하면 페이지 8 자체가 잘못 잘림.

### 다른 케이스도 비슷 (vpos↔px 비율 어긋남)

| paragraph | overflow | curr_h | avail | 다음 paragraph | 비고 |
|-----------|---------|--------|-------|---------------|------|
| pi=22 | 338 | 906.9 | 933.5 | pi=23 그림 (h=465.1) | 페이지 1 마지막 line |
| pi=62 | 932 | 569.7 | 933.5 | pi=63 단순 텍스트 | 정상 |
| **pi=83** | **886** | **906.6** | **933.5** | **pi=84 Table (h=190.9, fit 불가)** | **orphan** |
| pi=171 | 17053 | 904.4 | 933.5 | pi=172 단순 텍스트 | 정상 |

**관찰**: pi=83만 "**현재는 fit, 다음 block은 fit 불가**" 패턴. 다른 false positive들은 다음 paragraph가 작아서 같은 페이지에 함께 들어감.

## Stage 2 전략 재정의

원래 impl 계획(`task_404_impl.md`)은 `vpos_end > page_bottom_vpos + 283 HU` 단일 조건으로 push했지만, false positive 41건 중 1건만 진짜 orphan이라 무차별 적용 시 회귀 위험.

**refined trigger (heading-orphan 패턴)**:

```
조건 A: 현재 paragraph가 누적 height 기준으로 fit
       (current_height + para_h_px <= available_height)

조건 B: vpos 기준으로는 overflow
       (first_vpos + para_h_hu > page_top_vpos + body_h_hu + 283 HU)

조건 C: 다음 non-empty block(Table/Shape/큰 paragraph)이 잔여 영역에 fit 불가
       (current_height + para_h_px + next_block_h_px > available_height)

조건 D: col_count == 1, wrap_around_cs < 0
       (multi-column / wrap-around zone 회피)

A AND B AND C AND D → advance_column_or_new_page()
```

### refined trigger 검증 (진단 로그 기반)

- pi=83: A=fit, B=overflow=886, C=잔여 12.2px < pi=84 표 190.9px, D=충족 → **push ✓**
- pi=62: A=fit, B=overflow=932, C=잔여 363px > pi=63 14.7px (fit) → skip ✓
- pi=22: A=fit, B=overflow=338, C=잔여 26.6px < pi=23 그림 465.1px → push (?)
- pi=171: A=fit, B=overflow=17053, C=잔여 29.1px > pi=172 14.7px (fit) → skip ✓

pi=22의 경우 trigger 발동. 페이지 1 마지막 줄을 페이지 2로 보내고 pi=22+pi=23을 함께 배치. 이게 의도된 동작인지 확인 필요. (pi=22는 본문 텍스트 1줄, pi=23은 큰 그림. 함께 다음 페이지로 가는 게 한글/PDF와 일치한다면 OK.)

## Stage 2 진행 결정

다음 진행 전 작업지시자 승인 필요:

1. **refined trigger 채택 여부** (heading-orphan 패턴 4 조건)
2. **next-block 정의 범위** — Table/Shape만 vs 모든 다음 paragraph 포함
3. **pi=22 케이스 처리** — refined trigger 발동을 허용할 것인지 (페이지 1 변화 발생)

## 산출물

- 코드 변경: `src/renderer/typeset.rs` (TypesetState 필드 + reset 코드 + 진단 로그). 진단 로그는 Stage 2에서 제거.
- 보고서: `mydocs/working/task_404_stage1.md` (본 문서)

## 다음 단계

Stage 2 — refined trigger 구현 + pi=83 push 확인 + 회귀 테스트.
