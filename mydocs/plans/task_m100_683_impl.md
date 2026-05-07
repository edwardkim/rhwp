# 구현계획서 — Task #683

**이슈**: [edwardkim/rhwp#683](https://github.com/edwardkim/rhwp/issues/683)
**브랜치**: `local/task683`
**수행계획서**: `mydocs/plans/task_m100_683.md`

## 사전 조사 요약

### 레이아웃/페이지네이션 분포

- `src/renderer/height_measurer.rs::measure_paragraph` (L210~):
  - `total_height = spacing_before + lines_total + spacing_after - clickhere_adjustment`
  - 댓글: "그림 높이는 문단 높이에 포함하지 않음 (별도 PageItem::Shape로 처리)" (L340)
  - 즉 빈 문단 + 그림 의 `para_height` 는 line_height + line_spacing (≈ 1600 HU) 만 포함
- `src/renderer/pagination/engine.rs::process_controls` (L962~) → Picture branch (L1069):
  - `st.current_height += pic_h + margin_top + margin_bottom`
  - 즉 빈 문단 + 그림 cluster 의 누적 = `para_height(1600 HU) + pic_h(15696 HU) = 17296 HU`
- `src/renderer/pagination/engine.rs::paginate_text_lines` (L607~):
  - L671 `st.current_height += para_height;` — 문단 전체가 들어가는 경우
- 시각 결과: rhwp 출력 = 17280 HU (≈ 17296), 한글 2022 PDF = 18864 HU. **차이 ≈ 1600 HU = 한 줄.**

### 가설 (확정 전 stage 1에서 검증)

빈 문단(텍스트 없음) + `wrap=TopAndBottom` 그림의 paragraph 영역은 **line_height + image_height** 가 표준이지만, 실제 측정값(17296 HU)은 file-stored vpos 와 맞다. 즉 file 자체는 line/image overlap 모델로 저장되어 있고 한글 2022는 이를 무시하고 추가 1줄을 강제로 더 두는 것으로 추정.

또는 — 파일의 vpos 가 잘못 인코딩되어 있고 한글 2022는 line_segs 의 vpos 에 의존하지 않고 라인 + 이미지 구조를 새로 계산.

stage 1 에서 둘 중 어느 쪽이 맞는지 확정한 후 stage 2 로 진입.

## 구현 단계

### Stage 1 — 진단 및 산식 확정

**목표**: 빈 문단 + TopAndBottom 그림의 정확한 paragraph 높이 산식 확정.

**작업**:
1. rhwp 현재 동작 트레이스
   - `dump-pages` 출력 분석: 각 paragraph 의 h 값과 image vpos 의 의미
   - `current_height` 누적 추적 (디버그 print 또는 `RHWP_LOG` 활용)
   - 실제 image2 가 SVG 출력에서 y=633px 로 가는 경로 확인 (col_area.y + image_v_offset 계산)
2. 한글 2022 PDF 정밀 측정
   - 그림1 top, 그림1 bottom, "회색조:" baseline, 그림2 top 의 정확한 px 위치 측정 (PIL)
   - HWPUNIT 으로 환산하여 각 paragraph 의 효과적 높이 derive
3. 다른 샘플의 동일 패턴 비교
   - `samples/` 에서 빈 문단 + wrap=TopAndBottom 그림 패턴 보유 파일 검색
   - 동일한 1600 HU 차이가 일관되게 발생하는지 확인
4. 결론 도출
   - `(A)` measured.total_height 자체에 line_height 추가 vs `(B)` pagination 에서 picture_height 합산 시 line 추가
   - HWPX 동일 처리 영향 범위 확인

**산출물**: `mydocs/working/task_m100_683_stage1.md` — 진단 결과 + 채택 산식

**완료 기준**: 작업지시자 승인

---

### Stage 2 — 산식 수정 구현

**목표**: stage 1 에서 확정한 산식을 단일 위치에 적용.

**작업**:
1. 수정 위치 (stage 1 결과에 따라 양자택일)
   - **(A)** `height_measurer.rs::measure_paragraph` — 빈 문단 + TopAndBottom 그림 케이스에 line_height 한 줄 추가 후 picture_height 가 그 위에 누적되도록 조정
   - **(B)** `pagination/engine.rs::process_controls` Picture branch — TopAndBottom 그림 처리 시 추가 line_height 가산
2. 조건 가드
   - `pic.common.treat_as_char == false`
   - `pic.common.text_wrap == TextWrap::TopAndBottom`
   - 부모 문단이 텍스트 0 인 경우(또는 `cc=9, text_len=0`)
3. 단위 테스트 추가
   - `tests.rs` 또는 `pagination/tests.rs` 에 빈 문단 + TopAndBottom 그림의 height 검증 케이스
4. HWP3 영향 검토 (HWP3 도 같은 IR 사용 → 자동 적용 예상이나 회귀 검증 대상)

**산출물**:
- 코드 수정 (단일 또는 인접 파일)
- 단위 테스트
- `mydocs/working/task_m100_683_stage2.md` — 수정 내역 + diff 요약

**완료 기준**: 작업지시자 승인

---

### Stage 3 — 시각 검증 및 회귀 테스트

**목표**: `samples/pr-149.hwp` 정합 확인 + 다른 샘플 회귀 없음 확인.

**작업**:
1. `samples/pr-149.hwp` 정합
   - `cargo build --release && rhwp export-svg samples/pr-149.hwp -o output/svg/pr-149/`
   - PIL 측정: 그림1/2/3 top y 위치, 그림 간 거리
   - 검증 기준: 그림 top y ±10 px, 그림 간 거리 18864 HU ± 50 HU
2. 회귀 검증
   - `cargo test` 전체 통과
   - wrap=TopAndBottom 그림 보유 다른 샘플 (예: `samples/basic/` 일부, `samples/hwpx/` 일부) SVG 출력 → 시각 확인
   - `compare_svg_pdf.sh` 가 있는 경우 활용
3. 흑백/회색조 효과 회귀 확인 (이번 작업 범위는 아니지만 동일 SVG 사용)

**산출물**:
- `mydocs/working/task_m100_683_stage3.md` — 측정 결과, 회귀 결과, 시각 비교 첨부

**완료 기준**: 작업지시자 승인

---

### Stage 4 — 마무리 및 보고

**작업**:
1. 최종 보고서 작성 — `mydocs/report/task_m100_683_report.md`
   - 원인, 수정 내역, 검증 결과, 영향 범위, 후속 과제 (BlackWhite 디더링, GrayScale 정합 별개 이슈 분리 제안)
2. 오늘 할일 갱신 — `mydocs/orders/{yyyymmdd}.md` 또는 신규 생성
3. `git status` 확인 — 미커밋 파일 없음 확인
4. local/task683 의 모든 커밋이 작업 단위로 분리되어 있는지 확인

**산출물**:
- `mydocs/report/task_m100_683_report.md`
- `mydocs/orders/` 갱신
- 모든 변경사항 커밋

**완료 기준**: 작업지시자 승인 → local/task683 → local/devel merge 가능 상태

## 작업 외부 영향 평가

| 항목 | 영향 |
|------|------|
| HWP3 파서 | 변경 없음 (공통 IR 사용 → 자동 적용) |
| HWPX 파서 | 변경 없음 (동일 IR) |
| Skia 네이티브 렌더러 | 페이지네이션 결과 사용 → 영향 받음 (stage 3에서 검증) |
| WASM 빌드 | 코드 변경만 — 빌드 자동 반영 |
| 기존 회귀 테스트 | stage 3에서 cargo test 전체 통과 확인 |

## 위험 요소

1. **다른 샘플 회귀**: TopAndBottom 그림이 들어간 다른 문서에서 1600 HU 가 잘못 추가되어 시각 변화. → stage 3 광범위 회귀 검증으로 대응.
2. **HWPX vs HWP5 차이**: HWPX 의 line_segs 처리가 다를 가능성. → stage 1 에서 동일 패턴 HWPX 샘플 확인.
3. **빈 문단 정의 모호성**: text=0 만으로 충분한지, controls 만 있고 text 없는 케이스 모두 같은 처리인지. → stage 1 에서 정의 확정.

## 단계 요약

| Stage | 산출물 | 승인 시점 |
|-------|--------|----------|
| 1. 진단 | `working/task_m100_683_stage1.md` | 산식 확정 후 |
| 2. 구현 | 코드 수정 + 테스트 + `working/task_m100_683_stage2.md` | 빌드/테스트 통과 후 |
| 3. 시각 검증 | `working/task_m100_683_stage3.md` | 정합 + 회귀 없음 확인 후 |
| 4. 마무리 | `report/task_m100_683_report.md` + orders 갱신 | 모든 검증 완료 후 |
