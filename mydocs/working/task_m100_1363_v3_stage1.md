# Stage 1 (v3) — 측정 호출 배선 (scratch `layout_partial_paragraph` 실측)

A2 시뮬의 per-para **휴리스틱 높이 추정**(saved-vpos span / total_height / line_advances_sum)을
**scratch `LayoutEngine::layout_partial_paragraph` 실측**(렌더 권위)으로 대체하는 측정 경로를
배선하고, POC 로 휴리스틱↔측정 divergence 를 정량화했다.

## 1. 구현

### 1.1 신규 SSOT 레벨 `A3` (opt-in)
`EnSsotLevel` 에 `A3` 추가(`A2` 다음, ord 상위). `RHWP_EN_SSOT=A3`/`a3` 로 진입. 미설정 시
기본 `B` — **A3 코드는 `ssot_level >= EnSsotLevel::A3` 게이트 안에서만 동작**(순수 opt-in).

### 1.2 측정 전용 메서드 `measure_endnote_para_advance`
`src/renderer/typeset.rs`. scratch `LayoutEngine::new(self.dpi)` + 버리는
`PageRenderTree`/`RenderNode(Column)` 로 미주 para 를 실제 레이아웃해 **정확한 렌더
advance(px)** 를 반환한다.

- **좌표 프레임**: 시뮬과 동일한 컬럼 top=0 상대 프레임(`col_area = {x:0, y:0, width:en_col_w,
  height:available}`, `y_start`=상대 y). advance=`(y_after - y_start)` 는 프레임 평행이동
  불변이므로 렌더 절대 좌표와 정합.
- **렌더 인자 정합**: `multi_col_width_hu=None`(렌더 미주 body-flow 경로 `layout.rs:4275` 와
  동일), `start/end_line` = PartialParagraph 항목 지정 그대로(Full 은 `usize::MAX`→내부
  `lines.len()` 클램프), `section_index`=`st.section_index`, `para_index`=글로벌 미주 인덱스.
- **노드 폐기**: scratch tree/col_node 로 버려 실제 렌더 무영향. 매 호출 `new()` 라
  numbering/overflow 상태도 격리(Stage 2 에서 실증 예정).

### 1.3 시뮬 루프 배선
`simulate_endnote_column_bottom_y` 의 per-item advance 계산을 `heuristic_advance` 블록으로
래핑(기존 로직 보존). 그 뒤 `ssot_level >= A3` 이면 `measure_endnote_para_advance` 실측으로
대체, `RHWP_EN_SSOT_DEBUG=1` 시 `EN_MEASURE pi=.. y_top=.. heuristic=.. measured=.. diff=..` 로그.

## 2. POC 결과 — 휴리스틱↔측정 divergence 정량 (sep20/20)

`RHWP_EN_SSOT=A3 RHWP_EN_SSOT_DEBUG=1`, `3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`.

| 패턴 | 예시 | diff | 해석 |
|------|------|------|------|
| **다줄 텍스트/수식** | pi=466 224.3↔224.3, pi=1126 197.8↔237.1 | **~0** (일부 +39) | 저장 span 과 측정 정합(핵심: reflow 아님 검증) |
| **단줄 텍스트 +6px** | pi=1026 30.0↔36.1, pi=481 40.0↔46.1 | **+6.0 일정** | 렌더가 **trailing line-spacing** 포함, 휴리스틱(saved-vpos)은 누락 |
| **TAC 그림/도형** | pi=1131 0.0↔315.2, pi=1115 0.0↔259.2, pi=1175 0.0↔531.5 | 대폭(+) | 휴리스틱 0 → **측정이 정확**(개체 높이 반영) |
| **내부 vpos rewind** | pi=518 25.7↔6.0 | -19.7 | 측정이 단일줄로 과소(rewind 적층 미반영) |

**핵심 발견**: 측정은 **per-para 단독 렌더로는 정확**(다줄 텍스트 diff~0, TAC 그림 정확화).
그러나 미주 단 적층에서 한컴은 노트 간 trailing-ls 를 saved line_segs 처럼 **누락**하는데,
`layout_partial_paragraph` 는 매 para 에 trailing-ls 를 **포함** → **단줄마다 +6px 과대**.
이 누적 +6px 가 A3 에서 튜닝된 overflow 테스트 3건을 깬다(아래 §3).

> 이는 메모리 `trailing_model_no_ssot`(typeset 1984HU 가정 ↔ render 다줄 trailing 불일치)의
> 구조적 문제와 동일 — 측정 자체가 아니라 **노트 간 trailing-ls 조정**이 잔여 과제.

## 3. 검증

### 3.1 기본(B) 무회귀 — A3 순수 opt-in ✓
`cargo test`(전체) 기본 레벨: **exit 0, 123/123 test 바이너리 ok, FAILED/error 0**.
A3 게이트 밖 동작은 종전과 동일(heuristic 블록은 기존 로직 보존).

### 3.2 A3 측정 정합도 ✓ / overflow fit ✗(예상)
`issue_1082`(미주 단 overflow): B/A2 = 5 pass, **A3 = 3 fail**
(`exam_3_09_2022`, `exam_3_11_2022`, `exam_3_09_2024_sep2020`). 원인은 §2 의 단줄 +6px
trailing-ls 누적 → 단 과충전. **측정 배선·per-para 정합은 성공**, fit 정합은 trailing-ls
조정(Stage 2/3) 후 달성.

## 4. 상태 + 다음 (Stage 2)

- **배선 완료**: A3 경로로 scratch 측정 호출 동작. 좌표 프레임·렌더 인자 정합 확인.
- **정량 확보**: divergence 4패턴 분리(trailing +6px / TAC 정확화 / rewind 과소 / 다줄 정합).
- **Stage 2(부작용 격리)**: scratch 엔진 numbering/overflow/last_item_content_bottom 변이가
  측정에만 머무는지 실증. 더불어 **단줄 trailing-ls +6px 조정**(노트 간 trailing 제외 규칙)을
  측정값에 적용해 overflow 3건 재정합 검토 — `trailing_model_no_ssot` 제약 준수(전면 통일 금지).
- **fidelity 한계(기록)**: `bin_data_content=None`(명시 크기 그림 무관), `endnote_para_base`
  미설정(overflow tolerance 만 상이, advance 무영향).
