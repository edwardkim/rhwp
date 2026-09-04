# Task M100 #2278 Stage 3 완료보고서 — ofPie 보조플롯 + 팔레트 #5 실측 교정

- 이슈: #2278 "C2b: 3D 입체·ofPie 보조플롯 렌더"
- 브랜치: `local/task2278`
- 계획: `task_m100_2278_impl.md` 3단계 설계 (v2에서 무변경 승계 — 2D, 투영 무관)
- 선행: Stage 1R(+v2)·Stage 2 (시각판정 통과)
- 작성일: 2026-07-19

## 1. 팔레트 #5 실측 (계획 (a))

`pdf/chart/원형/원형대원형-2022.pdf`·`원형대가로막대형-2022.pdf` 임베드 비트맵
(2702×1577) 초록계 픽셀 히스토그램:

- 최빈값 **#27A172** (39,161,114) — 원형대원형 23,293표본 / 가로막대형 33,880표본,
  **두 파일 교차 일치** (2·3위는 전부 동일 색의 AA 변이)
- `DEFAULT_PALETTE[4]`: 0xFF5B9BD5(하늘, 유추) → **0xFF27A172(실측)** 교체,
  강등된 하늘은 [5]로 스왑(기존 [5] 유추 초록 0xFF70AD47 자리 — 실측 초록과
  중복 방지), [6][7] 유지. scheme_color 명시색 파싱(accent1=70AD47 등)은 무변경.
- `test_default_palette_hancom_order`(앞 3색 검사) 무수정 통과,
  신규 핀 `test_palette_index4_measured`.

## 2. 모델 + 파서 (계획 (b))

- `mod.rs`: `OfPieInfo { of_pie_type, split_pos, second_pie_size(기본 75),
  has_ser_lines }` + `OfPieType { Pie, Bar }` 신설, `OoxmlChart.of_pie` 필드.
  chart_type은 Pie 유지(#1453 라우팅 앵커) — of_pie 유무로 분기. 모듈 doc의
  ofPie/3D 항목을 지원으로 갱신.
- `parser.rs`: `ofPieChart` arm에 `of_pie = Some(default)` + 신규 arm 4개
  (`ofPieType`/`splitPos`/`secondPieSize`/`serLines`). serLines는 barChart(누적
  계열선)에도 오는 요소라 **Pie plot + of_pie 이중 게이트** (hiLowLines의 Stock
  게이트 선례). `splitType`은 범위 외(기본 last-k 동작).

## 3. 렌더러 (계획 (c))

`render_of_pie` 신설 + Pie 단독 경로 선두 분기 (2D `render_pie`·3D
`render_pie_3d` 무접촉):

- 분할: k = split_pos(반올림, 1..=n−1 클램프) 없으면 2. n<3 → 일반 원형 폴백.
  코퍼스 [10, 3.5, 1.5, 1.2] → 주 원 [10, 3.5] + 결합 2.7.
- 주 원: 앞 n−k 카테고리(palette(ci) — 범례 정순과 일치) + **결합 슬라이스
  palette(n)=[4] 실측 초록계**. 결합 슬라이스 중앙이 보조 플롯(3시)을 향하도록
  회전 (`start = −sweep_c/2 − (total−combined)/total·τ`) — 정답지 실측
  초록 경계 −38°..+34° (중심 ≈0° ✓ 규칙 검증).
- 보조 플롯: Pie=원 (r2 = r1×secondPieSize/100 — **실측 r2/r1=0.754 =
  secondPieSize 75 ✓ 스키마 의미 검증**; **시작각 = +sweep_c/2**, 결합 슬라이스
  아래 모서리 각도 정렬 — 실측 경계 26°≈유도 30°, 12시 시작 아님) /
  Bar=세로 누적 막대 (bar_h=2r2, 폭 0.45×, **첫 분할 카테고리 맨 위**).
- 레이아웃(1차 산출 대조 후 실측 캘리브레이션): 주 원 중심 x=0.23pw, 보조
  x=0.80pw, r1 = min(0.46pw, 0.76ph)/2 — 임베드(2702×1577) 실측 cx1=568/
  cy=915/r1=456/cx2=1831/r2=344 환산.
- serLines: 결합 슬라이스 양 모서리 → Pie는 보조 원 **접선점**(α±acos(r2/d)
  유도 — 실측: 원 상/하단 아님), Bar는 막대 좌변 상/하단. **검정**(실측 코어
  (8,8,8)) 0.75pt. combined ≤ 0 가드.
- 범례 무변경: legend_items Pie 분기(카테고리 n개 정순 + palette(i))가 "결합
  슬라이스 제외·정순" 요구를 현행 충족 — 플롯 색 매핑을 이에 일치.

## 4. TDD 증적

파서 4종 + 렌더러 8종 선작성 → RED 확인(팔레트 핀·ofpie 어휘 부재) → 구현 →
GREEN:

| 테스트 | 검증 |
|---|---|
| `test_parse_ofpie_info` | secondPieSize/serLines → Some{Pie,None,75,true} + Pie 앵커 |
| `test_parse_ofpie_bar_type` / `_split_pos` | val="bar" / splitPos 3.0 |
| `test_parse_serlines_not_leaked_to_barchart` | barChart serLines 이중 게이트 |
| `test_palette_index4_measured` | palette(4) == 0xFF27A172 |
| `test_ofpie_pie_secondary_and_serlines` | 주 3 + 보조 2 + serline 2 / false→0 |
| `test_ofpie_combined_slice_uses_palette4` | 결합 fill = palette(4) 참조 |
| `test_ofpie_bar_secondary_first_split_cat_on_top` | rect 2, [2]색 y < [3]색 y |
| `test_ofpie_split_pos_respected` | split_pos 3 → 주 2·보조 3 |
| `test_ofpie_legend_categories_in_order_no_combined` | 범례 4 정순·[4] 부재 |
| `test_ofpie_two_values_plain_pie_fallback` | n=2 → ofpie 어휘 부재 |

통합(`issue_2278_chart_3d_ofpie.rs` stage3 파트): 원형대원형·원형대가로막대형 ×
{hwpx,hwp} — 주 3·보조 2·serline 2·#27a172 존재·가로막대형 보조 rect·placeholder
부재. 기존 `test_parse_ofpie`(#1453 Pie 앵커) 무수정 통과.

## 5. 검증 결과

- ooxml_chart 유닛: **131 passed / 0 failed** (Stage2 120 + 파서 4 + 렌더러 7)
- 차트 통합·앵커 8스위트(1453 ofPie 2종 포함): **전부 통과**
- `cargo clippy --all-targets -- -D warnings`: 무경고
- `cargo test` 전체(v3 최종): **exit 0, 269 스위트 전부 ok, 3,259 passed / 0 failed**
  (로그: `$TMPDIR/c2b_stage3v3_full_test.log`; v1 3,256 → v2 밀착 가드 +1 → v3 explosion +2)
- fmt: 수정 파일 한정 (mod/parser/renderer/issue_2278 테스트)
- 시각판정 산출물: ofPie 2종 + 3D 5종 재산출 → `output/poc/chart_c2b/` +
  대조 합성 `output/poc/chart_c2b/compare/`

## 6. 시각판정 보정 v2 (2026-07-19)

작업지시자 판정 중 슬라이스 경계 논의 → 정답지 원주 전수 스캔 실측(2D 원형·
3차원원형 타원·ofPie 주/보조 원, 0.5r/0.6r/0.9r 호에서 흰 run **0건**)으로
**한컴은 원형 계열 슬라이스 밀착(흰 경계 없음)** 확정, 작업지시자 밀착 확정.

- 원형 계열 슬라이스 방출 6개소(2D render_pie / ofpie main·second path·rect /
  pie3d wall·top)의 `stroke="#ffffff"` 제거 — 마커(C1d)·라인 할로는 보존.
- 신규 가드 `test_pie_slices_butt_joined_no_white_border`(2D/3D/ofPie 2종 —
  RED 확인 후 제거) — 흰 테두리 재유입 차단.
- 2D 원형(2차원원형·쪼개진원형)도 동일 적용 — C1c 기승인 영역이나 당시 경계는
  쟁점이 아니었고 실측·작업지시자 확정에 따름.

## 7. 시각판정 보정 v3 — 쪼개진원형 explosion (2026-07-19, 작업지시자 편입 승인)

작업지시자 쪼개진원형 재검 요청 → 대조에서 **explosion 미구현** 확정: 한컴은
샘플 XML의 계열 레벨 `<c:explosion val="25"/>`(dPt 0개)를 반영해 전 슬라이스가
중심각 방향으로 벌어지는데 rhwp는 일반 원형으로 렌더. (앞선 "흰색 경계 분리"
논의의 실체 — 테두리가 아닌 explode.)

- 모델: `OoxmlSeries.explosion: Option<f64>` / 파서: 계열 컨텍스트 `explosion` arm
- 렌더(`render_pie`): 슬라이스 꼭짓점·호를 중심각 방향으로 `r×e/100` 이동,
  벌어진 extent가 기존 fit과 같도록 반지름 `1/(1+e/100)` 축소.
  **explosion 부재 시 산식·출력 불변** (2D 원형 무회귀)
- dPt 단위 explosion·3D/ofPie explosion: 코퍼스 부재 — 범위 외 기록
- TDD: `test_parse_pie_explosion` / `test_pie_exploded_slices_offset`
  (오프셋 = r×0.25·반지름 축소·슬라이스별 방향 분리 — RED→GREEN)

## 8. 위험·잔여

- 레이아웃 상수(cx1 0.30/cx2 0.78/r1 0.85 등)는 정답지 근사 초기값 — 시각판정
  보정 여지 (보정은 레이아웃 상수로만, 슬라이스별 상수 금지)
- splitType pos 외 값(val/percent/custom)은 범위 외 — 기본 last-k 동작
- 3D ofPie(3차원 원형대원형)는 코퍼스 부재 — 미도입
