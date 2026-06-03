# Task #404 — 헤딩 문단이 후속 콘텐츠와 다른 페이지로 분리됨 (최종 보고서)

## 요약

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 9쪽 하단에서 발생하던 heading orphan 을 해소했다. pi=83 "(7) 다수 기부자 현황" 이 후속 표 pi=84/85 와 함께 10쪽으로 이동.

`src/renderer/typeset.rs::typeset_section` 메인 루프에 vpos 기반 보정 trigger 추가 — 5개 조건 AND 결합으로 false positive 차단.

## 원인

- HWP 원본은 paragraph 단위 vpos (LineSeg 의 vertical_pos) 가 페이지 본문 영역(body_h_in_HU = 70012) 을 넘으면 다음 페이지로 보내는 구조
- 우리 엔진은 누적 height 기반(`current_height + para_h_px vs available_height`) 으로 fit 결정 → vpos 기준 미세 초과 (886 HU = 0.31mm) 케이스에서 누적 height 는 fit 으로 판정
- 결과: 헤딩 14.7px 만 페이지 9 끝에 잔류, 후속 표(pi=84 190.9px) 는 페이지 9 잔여 12.2px 에 못 들어가 페이지 10 으로 → orphan

## 적용된 수정

### Trigger 조건 (5개 AND, `typeset_section` 메인 루프)

```
A) !current_items.is_empty()                                # 페이지 첫 item 자기참조 회피
B) wrap_around_cs < 0 && col_count == 1                     # 단일 단 + non-wrap
C) current_height + para_h_px <= available_height           # 현재 fit
D) vpos_end > page_bottom_vpos + 283                        # vpos 기준 1mm 초과
E) next_h_px > 30.0                                         # 다음 paragraph substantial
   && current_height + para_h_px + next_h_px > avail        # 다음이 잔여 영역에 fit 안 함
```

발동 시 `st.advance_column_or_new_page()` → 현재 paragraph 가 다음 페이지로 push.

### page_top_vpos 계산

`current_items` 의 첫 item para_index 를 통해 매 iteration 즉시 계산. (`TypesetState` 필드 추적은 `typeset_paragraph` 내부 페이지 flush 시 동기 안 되는 문제 발견 → 방식 변경)

### 변경 파일

- `src/renderer/typeset.rs` — trigger 추가 (+33 / -15 line)

## 검증 결과

### 1. pi=83 페이지 이동 ✓

**Stage 1 (수정 전)**:
| 페이지 9 | 페이지 10 |
|---|---|
| 8 items, used=930.1px | 12 items |
| pi=77~82, **pi=83 heading** (마지막) | pi=84 표, pi=85 표, ... |

**Stage 2 (수정 후)**:
| 페이지 9 | 페이지 10 |
|---|---|
| 7 items, used=906.6px | 13 items |
| pi=77~82 | **pi=83 heading**, pi=84 표, pi=85 표, ... |

### 2. SVG 시각 검증 ✓

페이지 9 첫 본문 라인: "(6) 기부 금액별 기여도(총기부 금액 대비 각 비중)" (pi=77)
페이지 10 첫 본문 라인: **"(7) 다수 기부자 현황"** (pi=83) — 후속 표와 함께 동일 페이지

### 3. 회귀 테스트 — 1073개 모두 통과 ✓

```
1023 lib + 6 svg_snapshot + 25 paragraphs + 14 tables + 2 gugeo + 1 tab_cross_run
+ 그 외 통합 테스트 = 1073 통과
```

### 4. 10개 대표 샘플 LAYOUT_OVERFLOW 회귀 검증

| 샘플 | Stage 1 | Stage 2 | 차이 |
|------|---------|---------|------|
| **2025년 기부·답례품 (타겟)** | 57 | **42** | **-15 ↓** |
| 2022 국립국어원 업무계획 | 0 | 0 | 0 |
| aift | 3 | 3 | 0 |
| exam_eng | 0 | 0 | 0 |
| biz_plan | 0 | 0 | 0 |
| 21_언어_기출 | 14 | 14 | 0 |
| **kps-ai** | 5 | **4** | **-1 ↓** |
| k-water-rfp | 0 | 0 | 0 |
| 20250130-hongbo | 0 | 0 | 0 |

**회귀 0건 + 2개 샘플 개선** (orphan 으로 잘못 배치되던 paragraph 가 적절한 페이지로 이동하면서 overflow 감소).

### 5. 전체 페이지 수

타겟 샘플: 30 → 30 (변화 없음).

## False Positive 차단 분석

Stage 1 진단 로그에서 vpos overflow 가 발생한 paragraph 41건 중 1건만 진짜 orphan. 대표 false positive:

| pi | overflow | 원인 | Stage 2 trigger 결과 |
|----|---------|------|---------------------|
| pi=22 | 338 HU | wrap=TopAndBottom 표(line_segs 부정확) | 발동 (의미 없음 — 어차피 페이지 2 push) |
| pi=62~76 | 932~25572 HU | 페이지 8 의 TAC 그림(pi=57) wrap-around 로 vpos↔px 비율 어긋남 | 조건 E next_substantial 필터로 차단 (다음 paragraph 14.7px) |
| pi=171 | 17053 HU | 위와 동일 | 조건 E 차단 |

조건 E (next_substantial > 30px AND next_doesnt_fit) 가 핵심 필터.

## 비범위 확인

- `engine.rs::paginate_with_measured` (fallback 경로) — 본 task 미적용, 별도 후속 이슈
- HWP 5.x 바이너리 — vpos 메타가 LineSeg 에 있으면 동일 동작. 부정확하면 본 trigger 도 부정확. 본 task 범위 밖.
- ParaShape `keep_with_next` — 본 케이스(ps_id=11, kwn=0) 사용 불가. 향후 활성 케이스 발견 시 별도 보강.

## 단계별 진행 요약

| 단계 | 산출물 | 커밋 |
|------|--------|------|
| 계획 | `task_404.md` + `task_404_impl.md` | 36a457c |
| Stage 1 진단 | `task_404_stage1.md` + 진단 로그 코드 | 25b27b2 |
| Stage 2 구현 | `task_404_stage2.md` + trigger 코드 | 42b5136 |
| Stage 3 검증 | `task_404_report.md` (본 문서) | (이번 커밋) |

## 검증 기준 충족 (수행계획서 §검증 기준)

1. ✅ 9쪽 SVG 에 pi=83 heading 미표시
2. ✅ 10쪽 SVG 가 pi=83 heading + pi=84/85 표 함께 표시
3. ✅ 기존 회귀 테스트 모두 통과
4. ✅ 10개 대표 샘플 LAYOUT_OVERFLOW 카운트 회귀 없음 (+2 샘플 개선)

## 결론

heading orphan 패턴 1건 해소. 5개 조건 AND trigger 로 false positive 차단, 회귀 없음.
