# task_m100_1882 구현 계획서 — C1c 차트 스타일 4갭 보정

- 이슈: #1882 (C1c, #1431 Track C 하위)
- 브랜치: `local/task1882` (from `local/devel` = a433b0d8)
- 마일스톤: M100
- 수행계획서: `mydocs/plans/task_m100_1882.md` (승인 완료 2026-07-04)

## 1. 배경 / 설계 요지

샘플 XML에 스타일 값이 없어(범례 제외) 4갭은 **한컴 기본 스타일 재현**으로 보정한다.
수정 대상은 `src/ooxml_chart/{mod,parser,renderer}.rs`로 봉인됨(축/범례/팔레트 함수 전부
renderer.rs private, parse/render_svg 소비자는 shape_layout.rs:1703,1724 두 곳뿐).
라인 번호는 a433b0d8 기준.

설계 결정(수행계획서 §4 승인 사항):

- **② 팔레트**: `DEFAULT_PALETTE` 교체(앞 4색 정답지 PDF 픽셀 실측). `scheme_color` 의도적 무변경.
- **④ 축**: `nice_range` → `nice_axis`로 교체, `(min, max, step)` 반환. **경계 headroom + 조건부 step 재계산**.
  격자 전용 함수 분리 금지(막대 기하 renderer.rs:399 `t=(v-vmin)/(vmax-vmin)`가 격자와 같은 범위 공유 —
  분리 시 막대가 플롯 초과).
- **① 자동 제목**: 모델 플래그 방식(`chart.title`은 명시 텍스트 전용 유지 → parser.rs:85
  `series.is_empty() && title.is_none()` 조기 반환 가드 무변경).
- **③ 범례**: `LegendPos` enum(default=Bottom — 모델 직접 구성하는 기존 단위 테스트 보호),
  Right만 우측 세로 신설, 그 외 현행 하단 가로 폴백.

## 2. 단계 (5단계, stage-gated — 기능별 분해)

기능별 분해 사유: 갭 4개가 상호 독립(②④는 렌더러 단독, ①③만 모델+파서+렌더러 관통)이라
레이어별로 묶으면 렌더러 단계가 4갭을 한 번에 담는 거대 단계가 됨. 기능별이 단계당
시각 대조(PDF 실측 앵커)·회귀 귀속·롤백에 유리. 리스크 낮은 ②를 먼저 두어 시각 대조 기반을
확보하고, 최고 리스크 ④를 조기 배치한다.

### Stage 1 — 갭② 팔레트 (renderer.rs 단독, 최소 리스크)

- `renderer.rs:10-12` `DEFAULT_PALETTE` 교체:
  ```rust
  const DEFAULT_PALETTE: &[u32] = &[
      0xFF6183D7, // 파랑   — 한컴 2022 실측 (pdf/chart 픽셀 히스토그램)
      0xFFFE813B, // 주황   — 실측
      0xFFB0B0B0, // 회색   — 실측
      0xFFFCD801, // 노랑   — 실측 (원형 4슬라이스)
      0xFF5B9BD5, // 하늘   — 유추 (코퍼스 최대 4시리즈라 5번째+ 실측 불가)
      0xFF70AD47, // 초록   — 유추
      0xFF9013FE, 0xFF50E3C2, // 기존 유지
  ];
  ```
- `scheme_color`(parser.rs:438-452) **무변경** — schemeClr는 문서 테마 참조 의미(코퍼스 미사용),
  변경 시 `test_parse_combo_dual_axis`(parser.rs:516,521)가 근거 없이 깨짐.
- 테스트(신규 renderer 1): 무색 3시리즈 렌더 → `#6183d7`·`#fe813b`·`#b0b0b0` 포함 + `#70ad47` 미포함.
- 검증: `cargo test ooxml_chart` 전부 green(팔레트 hex assert하는 기존 테스트 없음 — grep 확인).

### Stage 2 — 갭④ 축 스케일 (renderer.rs 단독, 최고 리스크)

- `nice_range`(234-254) → `nice_axis(min, max) -> (f64, f64, f64)`:
  ```text
  step0 = floor-nice((max-min)/5)          # 현행 임계(1.5/3/7) 유지
  max1  = ceil(max/step0)*step0
  if |max1-max| < eps:                     # 데이터 max가 step 경계에 정확히 걸림
      max1 += step0                        # headroom +1 step
      step  = ceil-nice((max1-min)/5)      # norm≤1→1, ≤2→2, ≤5→5, else 10
      max1  = ceil(max1/step)*step
  else: step = step0                       # 확장 없으면 유지 (X축 0.5 간격 보존 필수)
  min' 도 동일하게 floor 정렬
  ```
  실측 앵커 3점 재현(수치 검증 완료): 막대 5.0→(0,6,2) 라벨 0,2,4,6 / scatter Y 4.0→(0,5,1) / X 2.6→(0,3,0.5).
- 반환 triple화 연쇄 이관: `value_range_for`(199, 소비 189/230/671/676) · `value_range`(229, 소비
  325/446/669) · `scatter_range`(260, 소비 518-519) — 전부 private, 컴파일러가 누락 검출.
- `render_bars`(317-326): percentStacked는 `(0.0, 100.0, 20.0)` 명시 전달(현행 6등분 출력과 동일 = 무회귀),
  stacked는 `nice_axis(0, max_sum)`.
- `render_value_grid`(809-870): `grid_lines=5` 고정 등분 → `step` 파라미터 + 정수 루프
  `n=((vmax-vmin)/step).round(); for i in 0..=n`(부동소수 누적 드리프트 방지). 비정수 step이면
  자동 `format_axis_num`(decimal) — `format_num` 정수 반올림이 0.5 step 라벨을 손상시키는 잠재 결함 차단.
- 테스트: 기존 1 갱신 — `test_render_scatter_decimal_axis_labels`(1221) `">2.4<"`→`">2.5<"`
  (의도된 스펙 변경, 주석 정정). 신규 3 — 막대 max 5.0→`>6<` 포함·`>5<`/`>3<` 미포함(성긴 라벨),
  scatter Y max 4.0→`>5<`, scatter X max 2.6→`>0.5<`·`>2.5<`·`>3<`.
- 알려진 편차(문서화): 콤보 보조축 max 10→축 0~15(50% headroom, 실측 앵커 없음). 보조축·기본축
  눈금 수 불일치 가능(보조축은 라벨만이라 시각 영향 미미, assert하는 테스트 없음).
- 검증: `cargo test ooxml_chart` + `cargo test --test issue_1453_chart_3d_ofpie_routing --test issue_1431_scatter`
  (percent `100%` 가드 포함).

### Stage 3 — 갭① 자동 제목 (모델+파서+렌더러)

- `mod.rs`: `OoxmlChart`에 `has_title_elem: bool`, `auto_title_deleted: bool` 추가.
- `parser.rs`: `handle_start` `b"title"`(263)에서 `chart.has_title_elem = true` 병기,
  `b"autoTitleDeleted"` arm 신설(`attr_val(e,"val")=="1"`). **:85 조기 반환 가드 무변경.**
- `renderer.rs`(91, 110-117): `effective_title = chart.title.clone()
  .or_else(|| (chart.has_title_elem && !chart.auto_title_deleted).then(|| "차트 제목".into()))`.
  `title_h` 분기·텍스트 렌더 모두 effective_title 기준. `font-weight` 600→400(한컴 regular weight,
  weight assert하는 테스트 없음).
- 테스트: parser 3(텍스트 없는 `c:title`→`has_title_elem=true`+`title=None` / `autoTitleDeleted=1` /
  기존 BAR_XML 명시 title 무회귀) + renderer 2(자동 "차트 제목" 출력 / `font-weight="600"` 미포함).
- 검증: `cargo test ooxml_chart` + 통합 2종(제목 텍스트는 클래스 assert에 무영향 확인).

### Stage 4 — 갭③ 범례 우측 (모델+파서+렌더러)

- `mod.rs`: `enum LegendPos { Bottom, Right, Left, Top }` + `OoxmlChart.legend_pos: LegendPos`
  (**default = Bottom**).
- `parser.rs`: `b"legendPos"` arm — r→Right, l→Left, t→Top, b→Bottom (legendPos는 c:legend 안에서만
  등장하므로 상태 플래그 불요).
- `renderer.rs`:
  - `render_legend`(909-977)를 `legend_items(chart) -> Vec<(String, u32, OoxmlChartType)>`
    (914-949 pie/일반 분기 추출) + 기존 가로 배치 + 신규 `render_legend_vertical`(행 높이 16px,
    수직 중앙 `y = plot_y + (plot_h - n*16)/2`)로 분해. 미사용 `_h` 파라미터를 수직 변형에서 사용.
  - `render_chart_svg`(90-160) 레이아웃 분기: Right이면 `legend_h=0`,
    `legend_w = clamp(최장라벨문자수*10.0 + 26.0, 50.0, w*0.30)`(CJK 10px/char),
    `plot_w = (w - left_pad - right_pad - legend_w).max(10.0)`. pie 경로(120-131) 동일 분기.
    `estimate_axis_label_width`와 충돌 없음(legend_w는 독립 가산항).
  - 범례를 `<g class="hwp-chart-legend">`로 래핑(통합 테스트 위치 검증용).
  - Right 외 값(Bottom/Left/Top)은 현행 하단 가로 폴백(코퍼스 27종 전부 r — Left 확장은 후속 저비용).
- 테스트: parser 1(r→Right, 미존재→Bottom) + renderer 2(Right: 범례 text x > plot 우측 경계 —
  data_bar_xs식 좌표 파싱 헬퍼 / Bottom default: 기존 하단 출력 무회귀).
- 검증: `cargo test ooxml_chart` + 원형 샘플 시각 확인(pie 범례 카테고리 우측 스택).

### Stage 5 — 통합 테스트 + 전체 회귀 + 시각검증

- 신규 `tests/issue_1882_chart_style_gaps.rs`(issue_1431_scatter.rs 관례 미러 — 샘플 로드 →
  `render_page_svg(0)` → substring assert, hwp+hwpx 각각):
  - `chart_auto_title_rendered` — `차트 제목` 포함 + `font-weight="600"` 미포함.
  - `chart_hancom_palette_applied` — 묶은세로막대형: `#6183d7`·`#fe813b`·`#b0b0b0` 포함, `#70ad47` 미포함.
  - `chart_axis_headroom_and_sparse_ticks` — 묶은세로막대형 `>6<` 포함·`>5<`/`>3<` 미포함,
    표식만있는분산형 `>2.5<`·`>5<` 포함.
  - `chart_legend_on_right` — `hwp-chart-legend` `<g>`의 첫 `<text>` x가 데이터 막대 최대 x보다 큼.
- `cargo test` 전체 + `cargo clippy --all-targets -- -D warnings`.
- 시각검증: 대표 샘플(묶은세로막대형/누적세로막대형/백프로기준누적세로막대형/꺽은선형/2차원원형/
  표식만있는분산형 + hwp 변형) SVG→PNG를 `output/poc/chart_c1c/`에 산출, `pdf/chart/*-2022.pdf`
  정답지와 4항목(제목/팔레트/범례/축) 대조 → **작업지시자 시각판정 요청**.
- 최종보고서 `mydocs/report/task_m100_1882_report.md` → 승인 후 origin push + upstream devel PR
  (Refs #1431, #1882).

## 3. 회귀 리스크 (실코드 대조 — 깨지는 기존 테스트 1개뿐)

| # | 대상 | 판정 |
|---|---|---|
| R1 | `test_render_scatter_decimal_axis_labels`(renderer.rs:1221) `">2.4<"` | **깨짐 — Stage 2에서 `">2.5<"` 갱신(의도된 변경)** |
| R2 | percent 라벨 `100%`/`0%` (단위 1136 + 통합 issue_1453:70-92) | 안전 — (0,100,20) 명시 전달로 출력 동일 |
| R3 | 기하 distinct 테스트(1113-1133) | 안전 — 절대 x 미단언, pad 자릿수 무변화 |
| R4 | `test_render_combo_dual_axis` 범례 텍스트 | 안전 — LegendPos default=Bottom |
| R5 | `test_parse_combo_dual_axis` schemeClr 색 | 안전 — scheme_color 무변경 |
| R6 | 통합 `!contains("차트 (미지원)")` | 안전 — parser.rs:85 가드 무변경(모델 플래그 방식) |
| R7 | scatter `>0<` 0-baseline | 안전 — step 루프가 vmin 눈금 포함 |
| R8 | issue_1156/1251 (레거시 ole_chart 경로) | 무관 — ooxml 모듈 미통과 |

## 4. 커밋 계획

단계별 1커밋(+stage 보고서 동커밋), 메시지 `Task #1882: Stage N — <내용>`.
기능 변경과 포맷 변경 분리, 무관 rustfmt churn 금지.
