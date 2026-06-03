# Task #463 최종 결과보고서

## 제목

exam_kor 14p~16p: 본문 외곽선 셀 leakage / 박스 정합 / wrap host 들여쓰기 / 인라인 그림 crop / 확장 바탕쪽 헤더 중복 — 다중 결함 일괄 수정

## 결과

✅ **완료 (8 stage)**. exam_kor.hwp 의 14p~16p 헤더·본문 외곽선·인라인 그림이 PDF 와 일치. 1069 단위테스트 통과, 5종 회귀 0.

## Stage 진행 요약

| Stage | 주제 | 산출물 |
|-------|------|--------|
| 1 (계획) | 수행계획서 + 구현계획서 | `plans/task_m100_463.md`, `plans/task_m100_463_impl.md` |
| 2 | 14p 본문 외곽선 셀 단락 leakage 차단 | `working/task_m100_463_stage2.md` |
| 3 | Stage 2 시각/회귀 검증 | `working/task_m100_463_stage3.md` |
| 4 | 최종 보고서 + orders 갱신 | (기존 `report/task_m100_463_report.md`) |
| 5 | 14p [A][B][C] 마커 박스 정합 (max-extent merge + wrap host override + floating 표 우측 확장) | `working/task_m100_463_stage5.md` |
| 6 | 14p wrap host 텍스트 들여쓰기 이중 적용 수정 (학생3) | `working/task_m100_463_stage6.md` |
| 7 | 15p 헤더 "(A형)" 표시 → 인라인 TAC 그림 crop 누락 수정 | `working/task_m100_463_stage7.md` |
| 8 | 16p/20p 좌측 헤더 "2/4" 겹침 → 확장 바탕쪽 apply_to 동일 시 active 대체 | `working/task_m100_463_stage8.md` |

## 변경 요약

### 코드 변경 (3 파일)

| 파일 | 변경 라인 | 내용 |
|------|----------|------|
| `src/renderer/layout/paragraph_layout.rs` | +31, -1 | 셀 단락 leakage 게이팅 (Stage 2) + 인라인 TAC 그림 emit 3곳에 crop/original_size_hu (Stage 7) |
| `src/renderer/layout.rs` | +60, -2 | wrap_around 영역 보정·max-extent merge·border_box_override·floating 표 우측 헬퍼 (Stage 5/6) |
| `src/document_core/queries/rendering.rs` | +18, -3 | 확장 바탕쪽 overlap=true + 같은 apply_to 시 active 대체 (Stage 8) |

### 문서 산출물

- `mydocs/plans/task_m100_463.md` — 수행계획서
- `mydocs/plans/task_m100_463_impl.md` — 구현계획서
- `mydocs/working/task_m100_463_stage{2,3,5,6,7,8}.md` — 단계별 보고서
- `mydocs/report/task_m100_463_report.md` — 본 최종 보고서

## 결함별 원인·수정

### 1. 14p 본문 외곽선 4 박스 분리 (Stage 2)

**원인**: 표 셀 안 단락(인용 따옴표 ｢｣ 3×2 표) 이 `para_border_ranges` 본문 큐에 leakage. cross-bf_id stroke signature merge 정책과 결합해 본문 그룹이 4개로 쪼개짐.

**수정**: `paragraph_layout.rs:2516` push 호출에 `cell_ctx.is_none()` 게이팅. 셀 단락은 더 이상 본문 큐에 들어가지 않음.

### 2. 14p [A][B][C] 마커 박스 외곽 표시 (Stage 5)

**원인 (3가지)**:
- merge geometry 가 첫 항목 (좁은 PartialParagraph wrap_area) 으로 굳어 후속 wider paragraphs 가 박스 밖으로 튀어나옴.
- wrap host paragraph 의 box geometry 가 좁은 wrap_area 로 굳음.
- floating 표 (어울림 wrap=Square) 우측이 box 밖으로 빠져나감.

**수정**:
- merge 시 `last.x` / `last.x+w` 를 `min(x)` / `max(x+w)` 로 max-extent 업데이트.
- `LayoutEngine` 에 `border_box_override: Cell<Option<(f64, f64)>>` 추가, wrap host text 직전 활성화. `paragraph_layout.rs` 의 push 가 override 가 있으면 그 geometry 사용.
- `compute_square_wrap_tbl_x_right` 헬퍼로 wrap=Square 표 우측 x 계산, override width 를 `max(col_area.width, tbl_x_right - col_area.x)` 로 확장. override 활성 시 `margin_right` 미차감.

### 3. 14p 학생3 (wrap host) 들여쓰기 (Stage 6)

**원인**: `layout_wrap_around_paras` 의 `wrap_area` 가 `wrap_text_x = col_area.x + LINE_SEG.column_start` 로 설정되는데 `column_start` 는 paragraph margin_left 를 이미 포함. 이후 `layout_composed_paragraph` 가 col_area (= wrap_area) 에 `margin_left + inner_pad` 를 추가로 더해 wrap host 만 margin_left 가 이중 적용.

**수정**: `wrap_area.x = wrap_text_x - host_margin_left`, `width += host_margin_left + host_margin_right`. text x 가 일반 paragraph 와 동일.

### 4. 15p 헤더 "국어 영역(A형)" 표시 (Stage 7)

**원인**: 바탕쪽 헤더 셀 인라인 그림 (bin_id=27, original 174000×26580 HU, crop r=102473) 의 좌측 58.9% 만 표시되어야 "국어 영역" 만 노출. 그러나 인라인 TAC 그림은 `picture_footnote.rs::layout_picture` 가 아닌 `paragraph_layout.rs` 의 텍스트 흐름에서 직접 emit — 3곳 모두 `crop` / `original_size_hu` 필드 미설정 → SVG 렌더에서 비트맵 전체 표시.

**수정**: `paragraph_layout.rs` 의 ImageNode emit 3곳 (1741, 2001, 2086 라인) 에 `picture_footnote.rs:84-101` 와 동일한 crop / original_size_hu 추출 코드 추가.

### 5. 16p/20p 좌측 헤더 "2/4" 겹침 (Stage 8)

**원인**: master[0] (Both, "2") 와 master[2] (Both, is_ext=true, **overlap=true**, "4") 가 같은 위치 헤더 셀에 그려짐. `rendering.rs:996+` 의 확장 바탕쪽 적용 로직이 `overlap=true` 를 항상 extra 로 추가 → 둘 다 렌더. 한컴 PDF 출력은 "대체" 로 동작 (작성자가 마지막 쪽 전용 헤더로 의도).

**수정**: `overlap=true` 확장이 active master 와 같은 `apply_to` 일 때 active 대체. 다른 `apply_to` 는 기존대로 extra.

## 검증

### 시각 비교 (page 14, 15, 16, 20)

| 페이지 | 변경 전 | 변경 후 | PDF |
|--------|---------|---------|-----|
| 14p (가) 박스 | 4개 분리 | **단일 큰 박스** ✓ | 단일 |
| 14p [A][B][C] 마커 | 박스 밖 | **박스 안** ✓ | 박스 안 |
| 14p 학생3 라벨 x | 151.17 (들여쓰기) | **139.84** ✓ | 정렬 |
| 15p 헤더 | 국어 영역**(A형)**(화법과 작문) | **국어 영역**(화법과 작문) ✓ | 국어 영역(화법과 작문) |
| 16p 좌측 | **2** + **4** 겹침 | **4** ✓ | 4 |
| 20p 좌측 | **2** + **4** 겹침 | **4** ✓ | 4 |

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed; 1 ignored; 0 measured
```

### 회귀 (5종 샘플)

- `2010-01-06.hwp` (6p) ✓
- `biz_plan.hwp` (6p) ✓
- `21_언어_기출_편집가능본.hwp` (15p) ✓
- `exam_eng.hwp` (8p) ✓
- `2022년 국립국어원 업무계획.hwp` (40p) ✓

## 잔존 한계 (별도 추적)

- **15p 우측 단 본문 overflow**: pi=103-105 의 LINE_SEG vpos-reset 이 col 1 → 다음 페이지 전환 시점에 미인식 (Task #459 와 동일 메커니즘이지만 col 1 경계). 본 task 범위 외, 별도 이슈로 분리 권장.
- **16p PDF 우측 "홀수형" 뱃지**: master 내용이 아닌 본문 별도 요소. 본 fix 와 무관, 별도 조사 필요.
- **`compute_square_wrap_tbl_x_right` 는 `horz_rel_to=Column` 케이스만 정확** (Stage 5 한계): `Paper`/`Page` 케이스 floating 표 위치는 다른 ref_x/ref_w 사용 — 별도 케이스 발견 시 보강 필요.
- **다른 `apply_to` 조합 확장 바탕쪽** (예: active=Odd + ext=Both, Stage 8 한계): 본 샘플에 없어 PDF 동작 미확인. 발견 시 별도 분석 필요.

## 커밋 이력

```
c500a07 Task #463 Stage 8: 확장 바탕쪽 헤더 중복 렌더링 수정
eedc395 Task #463 Stage 7: 인라인 TAC 그림 crop 누락 수정
0e81119 Task #463 Stage 6: wrap host 텍스트 들여쓰기 이중 적용 수정
58270f8 Task #463 Stage 5: 박스 geometry max-extent + wrap host override + floating 표 둘러싸기
7c46be0 Task #463 Stage 4: 최종 결과 보고서 + orders 갱신
6a6bbf8 Task #463 Stage 2: 셀 단락이 본문 외곽선 큐에 leakage 하지 않도록 게이팅
2a074be Task #463: 수행계획서 + 구현계획서
```

(Stage 8 최종 보고서 + orders 갱신은 본 stage9 커밋에 포함)

## 머지 절차

1. `local/task463` → `local/devel` no-ff merge
2. `local/devel` → `devel` push (별도 시점, 메인테이너 판단)
3. 이슈 #463 close

## 참조

- GitHub Issue: [#463](https://github.com/edwardkim/rhwp/issues/463)
- 관련 이전 이슈: Task #321 v6 (stroke signature merge), Task #430 (그림 crop)
- 관련 코드:
  - `src/renderer/layout/paragraph_layout.rs:2516` (Stage 2), :1741/:2001/:2086 (Stage 7)
  - `src/renderer/layout.rs:1604+` (Stage 5 merge), :2837+ (Stage 5 wrap_around override), :2880+ (Stage 6 wrap_area)
  - `src/document_core/queries/rendering.rs:996+` (Stage 8)
- 샘플: `samples/exam_kor.hwp`, `samples/exam_kor.pdf`
