# Stage 5 (v3) — 옵션 3: 전-단 1회 순차 scratch 렌더 (overflow→0 입증, gate 재보정 잔여)

Part A 의 per-para 고립 측정이 컨텍스트 의존·순차 상호작용으로 구조적 벽에 막힌 뒤(stage4),
**전-단 1회 순차 scratch 렌더**로 전환했다. sim==render 를 구조적으로 보장해 **overflow→0** 을
달성(접근 입증)했으나, break/split 게이트가 신(full-render) sim 값에 미보정되어 **sparse 페이지**
회귀가 남았다(Part B 게이트 재보정 과제).

## 1. 구현
- **`LayoutEngine::measure_endnote_column_bottom`**(layout.rs): 단 items 를 scratch 로 1회
  `build_single_column` 렌더해 단 bottom(px) 반환. 렌더 코드 자체로 측정 → vpos forward-jump·
  trailing·text_start_line 등 dispatch 가 네이티브 처리.
- **typeset A3 분기**(simulate_endnote_column_bottom_y): current_items(+extra)를 **로컬 +1 오프셋
  재색인**(0 더미)해 호출. 매 호출 새 scratch 엔진(격리).

## 2. 해결한 blocker 2건
1. **미주 vpos 정규화 비활성**: scratch `endnote_para_base=usize::MAX` 라 `endnote_line_vpos_base`
   (para>=base) 가 꺼져 절대 vpos 누수 → `endnote_para_base=0` 설정.
2. **`para_index==0` column-top vpos fallback 오발동**: 0-기반 재색인이 "섹션 첫 문단" fallback
   (절대 vpos 가산)을 잘못 발동(수식 para 35px→**13721px**→158쪽 폭발) → **로컬 인덱스 +1
   오프셋**(0 더미, 미참조)으로 회피.

## 3. 결과 (정정 — export-svg 158쪽은 CLI 아티팩트, 실제 렌더 경로는 건전)

**중요 정정**: 초기 export-svg 158쪽(sparse) 은 **CLI export-svg 경로 아티팩트**였다. 권위 검증
경로(`render_page_svg_native`, 테스트·studio·WASM 사용)와 페이지네이션(`dump-pages`)은 건전:

| 측정 경로 | A3(옵션3) 결과 |
|-----------|---------------|
| `dump-pages`(TypesetEngine 페이지네이션) | **23쪽** (건전) |
| `issue_1082`(render_page_svg_native, 5 exam overflow) | **5/5 pass** — sep20/20 23.5px→**0**, 두 배치 해결 |
| `export-svg` CLI | 158쪽 (별개 CLI 경로 폭발 — 실제 렌더 무관, 별도 조사 대상) |

- **overflow→0 + issue_1082 5/5 달성**: 옵션3 의 sim==render 가 1차 타겟(두 배치·overflow)을 실제
  해결. v2 A2 의 sep20/20 23.5px 회귀를 제거.

## 4. 잔여 — 13건 hancom 배치 회귀 (비단조 cascade)
전 endnote suite A3: `issue_1082` 5/5 이나 **`issue_1139` 13 fail**(issue_1139/1189/1209/1284 의
hancom-PDF 배치 테스트: "question30 우단 시작", "endnote boundary matches pdf" 등).

- 원인: **정확 sim ≠ hancom 의 (특이) 실제 배치**. 옵션3 의 정확한 fit 결정이 hancom 과 다른
  단/쪽 경계를 만든다. 튜닝된 게이트가 hancom 의 quirk 를 인코딩하고 있었음.
- 즉 옵션3 은 overflow-정확성은 얻었으나 hancom-배치-튜닝을 잃음 → **per-document 재보정** 필요.

## 5. 검증
- **기본 B 무회귀**: 옵션3 = A3 전용 게이트, B 무영향(전 suite 기본 green 유지).
- 격리 단위 테스트 유지(1 pass). 빌드 정상.
- 계측: `EN_COLSIM`·`EN_RENDER` 상주(RHWP_EN_SSOT_DEBUG).

## 6. 상태 + 다음
- **입증·달성**: 옵션3 = 구조적으로 옳은 측정(sim==render). overflow→0 + issue_1082(두 배치)
  실제 해결. blocker 2건(vpos 정규화·para0 fallback) 해소. Part A 발산 네이티브 해소.
- **잔여(다회차 재튜닝)**: 정확 sim 위에서 13건 hancom 배치를 재현하도록 게이트를 per-document
  재보정. 비단조 cascade 라 전 exam **동시 green** 게이트로([[tech_endnote_overflow_nonmonotonic_gate]]).
  v3 성공 기준(7 재튜닝 회귀 해소)은 이 재보정 완료 시점.
- **별개 조사**: A3 에서 `export-svg` CLI 가 render_page_svg_native 와 다른 페이지 수(158 vs 23)
  → export-svg 경로의 A3 페이지네이션 재실행/상태 차이 의심. 실제 렌더·테스트 무관하나 추적 필요.
- groundwork(measure_endnote_column_bottom, blocker 해소) 견고. A3 실험 opt-in.
