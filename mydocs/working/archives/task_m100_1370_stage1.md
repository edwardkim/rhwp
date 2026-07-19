# Task #1370 Stage 1 단계별 완료 보고서 — 진단 카탈로그

- **이슈**: #1370 (M100) / 브랜치 `local/task1370`
- **단계**: Stage 1 (진단 카탈로그, **코드 무수정**)
- **산출물**: `mydocs/tech/investigations/issue-1370/task_m100_1370_divergence_catalog.md`

## 1. 수행 내용

A3 회귀 13건을 `dump_page_items`/`build_page_render_tree`/`RHWP_EN_SSOT_DEBUG` 로 진단(검증 경로
고정, export-svg CLI 미사용). 각 테스트의 hancom 기대치 vs A3 실제값, 발산 모드, 책임 게이트 후보를
카탈로그로 정리.

## 2. 핵심 발견

1. **단일 근본 메커니즘 확정**: 레거시 게이트가 휴리스틱 높이 추정 위에서 hancom 의 "이른 break"
   quirk 를 인코딩했고, A3 가 추정을 정확 sim 으로 대체하면서 break 지점이 어긋남. 대부분 **A3 가
   hancom 보다 늦게 break** → 뒤 제목이 단 하단으로 밀리거나 overflow.
2. **앵커 실증** (2022_nov p17, #4): 문29 제목 `pi=812` 는 acc=18.0(제목 한 줄)에 불과한데 y=1000.7
   (단 하단 1001.6 = overflow). 직전 누적(`pi=802`=185.9, `pi=804`=128.5)이 단을 채우는데도 advance
   게이트가 발화하지 않음. → `a2_overflow_with_para` 발화 임계가 정확 sim 에서 어긋남.
3. **그룹 확정**:
   - 그룹 A(Stage 2): 경계 시프트/advance 미발화 — #2,#4,#8,#10,#13
   - 그룹 B(Stage 3): split/full-para 간격 — #1,#3,#4,#6
   - 그룹 C(Stage 4): tail/late titles/rewind — #5,#7,#9,#11,#12
4. **책임 게이트 후보**: `a2_overflow_with_para`·`split_endnote_to_fit`·
   `large_between_tail_render_overflows`(`*0.85`)·`near_bottom_tail`(`*0.90`)·
   `advance_large_between_single_line_rewind`·`compact_endnote_own_vpos_span_fits`. 정확 책임은
   Stage 2~4 조건 토글로 실증 확정.

## 3. 검증

- 코드 무수정 → 빌드/테스트 영향 없음. 기본(B) 72/72·A3 59/72(13 failed) 현황만 측정(이슈 일치).
- 그룹 매핑은 발산 모드 + typeset.rs 게이트 독해 기반 1차 가설. Stage 2 부터 실증 갱신.

## 4. 다음 단계

Stage 2 — 그룹 A 재보정. `a2_overflow_with_para` 발화/`compact_endnote_own_vpos_span_fits`/
`advance_column_or_new_page` 를 정확 sim bottom 기준으로 교정. 매 수정 전 exam(1082/1139) A3 동시
green 게이트.

## 5. 승인 요청

Stage 1 진단 카탈로그 검토 및 그룹 분할(A/B/C) 승인 요청. 승인 시 Stage 2(그룹 A, 첫 소스 수정) 착수.
