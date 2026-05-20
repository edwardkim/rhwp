# Stage 5 종합보고서 — #1022: VPOS_CORR 정합 도전 결과

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5 — `MeasuredParagraph` ↔ VPOS_CORR 정합 도전 (작업지시자 결정 "방향 1 도전")

## 1. 추가 분석 — lazy_base lifecycle

페이지 22 의 VPOS_CORR 발동 패턴 (`RHWP_VPOS_DEBUG=1` 로그):

| 항목 | prev_pi | VPOS_CORR 발동 | base 경로 | delta |
|------|---------|----------------|-----------|-------|
| pi=222 | 221 PartialTable | **skip** (`prev_item_was_partial_table` gate) | — | 0 |
| pi=223 | 222 Full | **fresh compute** | lazy_base 새로 계산·캐시 | +13.87 |
| pi=224 | 223 Table | **skip** (로그 없음 — Table item 경로 분기) | — | 0 |
| pi=225 | 224 Full | **fresh recompute** (Table 후 lazy_base 리셋된 듯) | lazy_base 재계산 | +13.87 |
| pi=226 | 225 Full | **cached** | pi=225 의 lazy_base 재사용 | 0 |

**핵심**: fresh-compute 시 `+trailing_ls_prev` 가 더해지고, cached 시 추가 보정 없음. 각종 reset trigger 가 fresh vs cached 를 결정.

## 2. paginator 미러링 요구

paginator 가 VPOS_CORR 와 비트 정합하려면:

1. `vpos_lazy_base: Option<i32>` 상태 추적.
2. VPOS_CORR 의 발동·차단 게이트 미러 (`prev_item_was_partial_table`, `prev_has_overlay_shape`, `seg.vertical_pos == 0 && prev_pi > 0`, `MAX_BACKWARD_PX`, `stale_table_host_vpos`).
3. lazy_base 리셋 trigger 미러 (`layout.rs:2186`(Shape/Table anchor 시 set), `2387`(fresh compute 시 set), `2625`(?) 등 모든 set/reset 경로).
4. `paragraph_layout` 의 실제 advance 정책 — `is_cell_last_line && cell_ctx.is_some()` (trailing_ls 제외), `skip_advance_empty_wrap` (advance 안 함) 분기를 paginator 가 인지.
5. fresh vs cached 결정 후 적절한 delta(=`+trailing_ls_prev` 또는 0) 가산.

각 분기는 다수 task 누적 보정(#332/#412/#474/#479/#485/#537/#539/#643/#716/#874 등)으로 보강된 상태 — 변경 시 모두 검증 필요.

## 3. 시도 결과 (Stage 5-3 방향 3 + 추가 분석)

| 시도 | 결과 |
|------|------|
| 단순 vpos gap 가산 (방향 3) | 38→37, page 22 미해결, test_task76 회귀, page count +6 |
| 일률적 trailing_ls 가산 (5 transition × 13.87 = 69px) | 과대 — VPOS_CORR 실측 27.74 보다 41 더. 다른 페이지 회귀 확실 |

fresh/cached 구분 없이 단순 미러는 회귀가 큼.

## 4. 평가

세션 한계 안에서 다음을 안전하게 완료하기는 어렵다:

1. layout.rs:2152~2625 의 VPOS_CORR 모든 게이트·base set/reset 경로 정밀 감사 (수십 task 누적).
2. paragraph_layout 의 `layout_composed_paragraph`/`layout_raw_paragraph`/`layout_partial_paragraph` 의 advance 정책 정밀 감사 (셀 컨텍스트·wrap zone·is_cell_last_line 분기).
3. paginator 에 두 모듈의 동작을 비트 정합하게 미러 — TypesetState 확장.
4. 광범위 골든·테스트 회귀 점검.

방향 3 (단순) 의 회귀(`test_task76`)가 보여주듯, 정밀 미러 없이 부분 적용은 회귀 위험만 키운다.

## 5. 정정 권고

본 #1022 의 명시 범위(`HeightMeasurer ↔ cell_units`)는 Stage 3 에서 완료
(42→38 events, cell_units 가 HeightMeasurer 와 동일 산식). 페이지 22
잔여 18.3px 의 원인은 paragraph layout 위치 정합(`MeasuredParagraph` ↔
`layout_composed_paragraph` ↔ VPOS_CORR) 부류로, 이 작업은 본 타스크
보다 broader scope 의 **multi-week effort** 가 필요한 별도 과제다.

**별도 후속 이슈로 분리 권고**:

새 M100-NNN "VPOS_CORR ↔ MeasuredParagraph 정합" (가칭). 범위:
- paragraph_layout advance 정책 감사·문서화.
- VPOS_CORR lazy_base lifecycle 감사·문서화.
- 두 모듈의 정합 설계 (기존 정합 함수 추출 또는 통일).
- paginator 미러.
- 광범위 회귀 검증.

본 #1022 는 Stage 3 deliverable 로 마무리, 최종 보고서 작성.
