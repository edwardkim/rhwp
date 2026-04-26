# 구현계획서 — Task #356 페이지 분기 오버플로

- **이슈**: #356
- **수행계획서**: `task_m05x_356.md`
- **브랜치**: `local/task356`

## 핵심 변경 위치

| 파일 | 라인 | 역할 |
|------|------|------|
| `src/renderer/pagination/engine.rs` | 215~290 | vpos 기준 `current_height` 보정 (현재 `page_has_block_table` 조건부) |
| `src/renderer/pagination/engine.rs` | 583~642 | `paginate_text_lines()` — 문단 적합성 판정과 forced_breaks 처리 |
| `src/renderer/pagination/engine.rs` | 595~604 | 기존 `--respect-vpos-reset` (문단 *내부* LINE_SEG 리셋만) |
| `src/document_core/queries/rendering.rs` | 1618~1672 | `compute_hwp_used_height()` — 권위값 기반 누적, 분기에는 미사용 |

## 구현 전략

문단 단위 페이지 분기 결정 직전에 **인접 문단 vpos 리셋 신호**를 검사하여, 다음 조건을 모두 만족하면 강제 페이지 분기를 적용한다.

### 분기 트리거 조건

`prev_para` (현 페이지에 마지막으로 배치된 문단) 와 `cur_para` (이번에 배치할 문단) 가 다음을 모두 만족할 때:

1. 두 문단 모두 LINE_SEG 권위값을 보유 (`line_segs` 비어있지 않음)
2. `cur_para` 의 첫 ls.vpos < `prev_para` 의 마지막 ls.vpos_end (= vp + lh + ls)
3. 둘 다 같은 컬럼(column_start) — 다단 환경에서 단 변경과 구분
4. `cur_para` 가 표(Table) / 머리말 / 꼬리말 / 각주 본문이 아님 — 표 분할 정책 영향 회피
5. 페이지 첫 문단이 아님 (`prev_para` 존재)
6. (옵션) 갭이 의미 있는 크기 — `prev_vpos_end - cur_vpos > N HU` (오탐 방지 임계, 초기값 N=0 → 단순 비교)

### 적용 동작

- 위 조건 만족 시: `cur_para` 처리 직전 `st.advance_column_or_new_page()` 호출 (= 강제 페이지/단 분기)
- 그 후 정상적으로 `paginate_text_lines()` 진입

### 게이팅

- 기본 활성화 (`true`). 회귀가 발견되면 단계 4 에서 옵션 플래그로 다운그레이드 검토.
- `respect_vpos_reset` 와 별개의 로직(인접 문단 vs 문단 내부)이므로 기존 옵션과 충돌 없음.

## 단계 구성 (총 5단계)

### 단계 1 — 재현 및 정량 진단 (코드 무수정)

**목적**: fix 전·후 비교 기준 확보.

- `samples/2022년 국립국어원 업무계획.hwp` 로:
  - `rhwp dump-pages -p 2` 출력 캡처 → pi=20..42 배치 확인
  - 페이지 3 의 pi=39 vpos_end vs body_area_hu 계산
  - 전체 SVG 페이지 수 카운트 (현재 35 추정)
- `samples/exam_eng.hwp`, form-002 등 골든 케이스에서 인접 문단 vpos 리셋 발생 빈도 사전 조사 (ad-hoc 스크립트 또는 dump-pages 다중 실행)
- 산출물: `mydocs/working/task_m05x_356_stage1.md` — 측정값 표 + 영향 범위 추정

**승인 후 단계 2 진입.**

### 단계 2 — 인접 문단 vpos 리셋 감지 헬퍼 추가

**목적**: 분기 결정 로직을 작은 함수로 분리, 단위 테스트 부착.

- `src/renderer/pagination/engine.rs` 또는 신규 모듈에 헬퍼 추가:
  ```rust
  fn detect_inter_paragraph_vpos_reset(
      prev_para: &Paragraph,
      cur_para: &Paragraph,
  ) -> bool
  ```
- 위 5개 조건 중 (1)(2)(3) 을 본 함수에서 처리. (4)(5) 는 호출 측에서 게이팅.
- 단위 테스트: `tests/` 또는 모듈 내 `#[cfg(test)]` 로 4~6 케이스
  - 정상 진행 (cur.vpos > prev.vpos_end) → false
  - 명백한 리셋 (cur.vpos = 0, prev.vpos_end = 60000) → true
  - 같은 페이지 위치 (cur.vpos = prev.vpos_end + 줄간격) → false
  - 다른 컬럼 (column_start 다름) → false
  - 빈 line_segs → false
- 산출물: `mydocs/working/task_m05x_356_stage2.md`

**승인 후 단계 3 진입.**

### 단계 3 — 페이지네이션 엔진 통합

**목적**: 실제 분기 결정에 헬퍼 적용.

- `process_paragraph` 또는 동등 위치(문단 처리 진입부, 표/페이지브레이크 컨트롤 처리 후)에서:
  - `prev_pagination_para` 와 `paragraphs[para_idx]` 비교
  - 문단이 표/머리말/꼬리말/각주 등 특수 영역이 아닌 경우에만 검사
  - 페이지 첫 문단(=`st.current_items` 가 비었거나 페이지 시작 직후)이면 skip
  - 트리거 시 `st.advance_column_or_new_page()`
- 디버그 로그(`PAGINATE_TRACE` 또는 신규 env 가드) 추가하여 분기 사례 추적
- 산출물: `mydocs/working/task_m05x_356_stage3.md` + 코드 커밋

**승인 후 단계 4 진입.**

### 단계 4 — 통합 검증 및 회귀 측정

**목적**: 본 샘플 수정 확인 + 골든 SVG 회귀 평가.

- 본 샘플:
  - `dump-pages -p 2` → pi=39 까지만 page 3 에 포함 (pi=40 → page 4)
  - `export-svg` 페이지 수 = 37 (PDF 일치 여부 확인. 차이 있으면 분석)
  - 페이지 3 SVG 의 footer `- 1 -` 가 body_area 내부에 있음
- 회귀:
  - `cargo test` 전체 (단위/통합)
  - `tests/golden_svg/*` diff 검사. 변경 발생 시 픽셀 수준 비교 후 의도된 변경/회귀 분류
  - `samples/exam_eng.hwp` 등 주요 샘플 재현 (페이지 수, LAYOUT_OVERFLOW 메시지 카운트)
- 회귀가 큰 경우: 게이팅을 옵션 플래그로 변경하거나 조건 (4)(6) 강화
- 산출물: `mydocs/working/task_m05x_356_stage4.md` — 검증 결과 표 + 골든 변경 목록

**승인 후 단계 5 진입.**

### 단계 5 — 최종 보고서 및 머지 준비

- `mydocs/report/task_m05x_356_report.md` — 최종 보고서 (배경/원인/해결/검증/잔여과제)
- `mydocs/orders/20260426.md` 상태 갱신 (진행 → 완료)
- 커밋 정리 (단계별 커밋 유지)
- `git status` 확인 후 작업지시자 머지 승인 요청

## 검증 기준 요약

| 항목 | 기대값 |
|------|--------|
| 본 샘플 페이지 3 footer 위치 | body_area 내부 |
| 본 샘플 SVG 페이지 수 | 37 (PDF 일치) |
| LAYOUT_OVERFLOW 경고 (해당 샘플) | 0 또는 현재 대비 감소 |
| `cargo test` | 전체 통과 |
| 골든 SVG 회귀 | 의도된 변경 외 0 |

## 위험과 완화

| 위험 | 완화 |
|------|------|
| HWP vpos 리셋이 의도된 페이지 break 가 아닌 경우 (단 변경, 표 내부) | 조건 (3)(4) 로 같은 컬럼·표 외부 제한 |
| 골든 SVG 다수 변경으로 검증 부담 | 단계 4 에서 사전 측정 후 작업지시자 결정 |
| `prev_pagination_para` 와 페이지 시작 인식 어긋남 | `st.current_items.is_empty()` 또는 페이지 첫 문단 플래그 명시 검사 |
| 신규 디버그 로그가 일반 출력 오염 | env 가드(`RHWP_TRACE_PAGE_BREAK=1`) 사용 |
