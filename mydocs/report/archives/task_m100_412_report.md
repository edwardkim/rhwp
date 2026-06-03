# Task #412 최종 결과 보고서

**제목**: 다단 우측 단 단행 문단 줄간격 누락 — vpos 보정 공식의 base 차감 무력화

**Issue**: https://github.com/edwardkim/rhwp/issues/412

**브랜치**: `local/task412`

**작업 기간**: 2026-04-28

---

## 1. 요약

`samples/exam_eng.hwp` 1페이지 우측 단 7번 문항 선택지 5개의 줄 간격이 좁게 렌더링되던 버그를 수정. 추가로 좌측 단의 첫 선택지에서 발생하던 catch-up 간격 확대 문제와 페이지 2 item 20 의 동일 문제를 함께 해결.

근본 원인은 `layout.rs:1392-1430` 의 vpos 보정 공식이 `col_area.y` 와 `vpos_page_base` 의 좌표 의미를 혼동한 것. body_wide_reserved (페이지 너비 글상자/표) 푸시가 있는 다단 레이아웃에서 첫 항목이 `col_area.y` 가 아닌 푸시 적용 후 위치에서 시작하는데, 보정 공식이 이를 반영하지 않아 보정값이 first_item_vpos × scale 만큼 위로 어긋나 다단 우측 단처럼 base 가 큰 경우 조건 검사를 통과하지 못하고 스킵됨.

## 2. 변경 요약

### 2.1 코드 변경 (`src/renderer/layout.rs`)

1. **anchor 도입**: `build_single_column` 진입 시 body_wide_reserved 푸시 적용 직후의 y_offset 을 `col_anchor_y` 로 캡처. 첫 PageItem 의 실제 렌더 y = vpos_page_base 좌표.

2. **vpos_end 정밀화**: `prev_seg.vpos + lh + ls` 대신 현재 paragraph 의 first seg vpos 를 우선 사용. HWP 가 paragraph spacing_after 를 다음 paragraph 의 first vpos 에 인코딩하므로 더 정확. vpos reset(0) 또는 prev 보다 작아진 경우 prev 기반 fallback.

3. **page_path/lazy_path 분리**:
   - page_path: `col_anchor_y + (vpos_end - base) * scale`
   - lazy_path: `col_area.y + (vpos_end - base) * scale` (기존 유지)

### 2.2 변경 hunk 수

- 핵심 로직: 1 hunk (`vpos 보정 분기` + anchor 캡처)
- 진단 코드: 1 hunk (env-gated `RHWP_VPOS_DEBUG`)

## 3. 검증 결과

### 3.1 원래 보고된 버그 (exam_eng p1 우측 단 item 7)

| 옵션 간 | Pre-fix Δy | Post-fix Δy | 기대 |
|--------|-----------|-----------|------|
| ①→② | 15.34 ❌ | 22.54 | 22.55 |
| ②→⑤ | 15.33 ❌ | 22.55 균일 | 22.55 |

### 3.2 추가 수정된 케이스

| 케이스 | Pre-fix | Post-fix |
|--------|---------|----------|
| p1 좌측 단 item 1 ①→② | 28.56 (catch-up) | 21.89 ✓ |
| p2 우측 단 item 20 ①→② | 26.59 (catch-up) | 19.92 ✓ |

### 3.3 회귀 검증 (7개 샘플 합계 268페이지)

- cargo test 6/6 통과
- 단단 문서 (k-water-rfp, aift, kps-ai 등): 영향 없음
- 다단 문서 (exam_eng/kor/math): page_path 보정 정상화
- snapshot 1건 갱신: aift-page3 약 3.68 px shift — issue #147 본래 검증 목적과 무관한 위치 미세 조정이며 HWP vpos 의도에 더 부합

### 3.4 분포 변화 (exam_eng)

| 분류 | Pre-fix | Post-fix |
|------|---------|----------|
| page_true (보정 적용) | 127 | 127 |
| page_false (스킵) | 15 | **0** |
| lazy_true | 131 | 131 |
| lazy_false | 3 | 3 |

→ 모든 page_path 보정이 정상 발동.

## 4. 잔존 이슈 (별도 task 권고)

### 4.1 exam_eng p2 우측 단 item 18 ①→② (Δ=15.33, 기대 19.93)

**원인**: pi=104 (Picture InFrontOfText + Table TopAndBottom tac=true) 가 다음 paragraph (pi=105) 의 vpos 보정 분기를 두 가드 (`prev_has_overlay_shape`, `prev_tac_seg_applied`) 로 차단. 결과적으로 lazy_base 초기화가 pi=106 으로 미뤄지면서 pi=105→106 한 nudge 누락.

**왜 본 task 에서 해결 안 함**:
- overlay shape bypass 의 의도(개체 높이 포함된 vpos 사용 시 과대 보정 방지)와 lazy_base 초기화 타이밍이 얽혀 있어 단순 분리가 위험
- 본 task 의 주 목적(다단 우측 단 base 차감 문제) 과 다른 root cause
- 별도 회귀 검증 필요

→ 신규 issue 생성 후 별도 task 로 추적 권고.

## 5. 단계별 산출물

| Stage | 산출물 |
|-------|-------|
| 1 (분석) | `mydocs/plans/task_m100_412.md`, `mydocs/plans/task_m100_412_impl.md`, `mydocs/working/task_m100_412_stage1.md` |
| 2 (구현) | `mydocs/working/task_m100_412_stage2.md` |
| 3 (회귀) | `mydocs/working/task_m100_412_stage3.md` |
| 4 (보고) | `mydocs/report/task_m100_412_report.md` (본 문서) |

## 6. 커밋 이력

```
3d395e2 Task #412 Stage 3: 다중 샘플 회귀 검증
cdeea6b Task #412 Stage 2: vpos 보정 anchor (col_anchor_y) 도입 + curr_first_vpos 사용
144ff53 Task #412 Stage 1: vpos 보정 진단 + 영향 범위 분석
```

## 7. 핵심 학습 사항

1. **HWP vpos 의 좌표 의미**: vpos 는 column-relative (column body top = vpos 0). body_wide_reserved 푸시가 있어도 column 내 vpos 좌표계는 동일.

2. **col_area.y 와 first_item_vpos 의 관계**: 일반적으로 일치하지 않음. body_wide_reserved 푸시 후 y_offset 이 vpos_page_base 에 대응. 따라서 vpos→y 변환은 anchor (= 푸시 후 y_offset) 를 사용해야 정확.

3. **HWP paragraph spacing_after 인코딩**: `paragraph[N+1].first_seg.vpos = paragraph[N].last_seg.vpos + lh + ls + spacing_after`. 즉 다음 paragraph 의 first vpos 가 spacing_after 까지 포함된 정확한 위치.

4. **page_path / lazy_path 의 좌표 anchor 차이**:
   - page_path: 첫 항목 vpos 가 명확. anchor = col_anchor_y (post body_wide_reserved 푸시).
   - lazy_path: 첫 항목 vpos 신뢰 불가 (Shape/PartialTable). anchor = col_area.y (lazy_base 가 sequential y_offset 에서 역산).

## 8. 결론

본 task 의 목표인 다단 우측 단 옵션 줄 간격 버그는 해결됨. 부수적으로 좌측 단 / 페이지 2 의 유사 catch-up 케이스도 함께 해결됨. 회귀 영향 미미 (snapshot 1건 미세 조정, 단단 문서 영향 없음). 잔존 item 18 케이스는 별개 root cause 이므로 별도 task 로 추적.

local/devel merge → devel push 진행 가능.
