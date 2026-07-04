# Task #1836 최종 보고 — seoul_0776 라운드트립 회귀: 빈-앵커 host_line_spacing 억제 소스 무관화

## 결론

`typeset.rs format_table` 의 빈-앵커 host_line_spacing 억제(#1147)가 `is_hwpx_source`
게이트라 HWPX 만 억제하고 HWP5 재파스는 미억제(+12px phantom) → 라운드트립 pagination
divergence. 렌더 경로(layout.rs)는 이미 소스 무관 억제이므로, typeset 을 대칭으로
소스 무관화하여 seoul_0776 STRUCT_MISMATCH 를 해소.

## 진단 (이등분 + 계측)

- `git bisect run`: 첫 BAD `e0c471a7`(#1763 clamp). 소거 검증으로 #1763 은 트리거,
  근본은 #1147 게이트로 규명 (clamp 트레이스 A/B 동일, TABLE_DRIFT host_sp A=0/B=12).
- 한글 대조: 넘침 줄 한글 p4 = B(미억제). 단, 렌더는 layout 이 소스 무관 억제로
  833px(p3) 자기정합 — 46px/p4 fidelity 는 별개 선존 축.

## 수정

`typeset.rs`: `is_hwpx_source` 게이트 제거, `is_topbottom_empty_anchor` 소스 무관화.
미사용 `is_hwpx_source: bool` 파라미터 제거(시그니처+호출 2곳). #1147(HWPX) 억제 유지,
#1133(next=table anchor) 보존 조건 유지.

## 검증

- seoul_0776: STRUCT_MISMATCH 646.65 → **PASS 0.00**
- issue_1133_nested_table_valign / svg_snapshot 골든: 통과
- big_hwpx 2,500 render-diff: 직전 스택(1841) 대비 **회귀 0 / 개선 11** (seoul_0776 STRUCT→PASS 포함; 나머지는 최신 devel merge + 본 수정 기여)
- big_hwp 2,500 네이티브 render-diff: **회귀 0 / 개선 2**, PASS 2494 / STRUCT 0 (본 수정이 HWP5 억제로 전환하는 핵심 게이트 — 네이티브 회귀 없음 확인)

## 비고

- 트리거였던 #1763 clamp 는 정상 동작(한글 정합 개선)이라 불변 유지.
- 계측 도구(RHWP_TABLE_DRIFT host_sp, RHWP_FLOW_DBG cur_h)로 pi=20 12px divergence 확인.

## 후속 수정 (PR #1863 CI 회귀)

소스 무관화가 CI `Build & Test` 의 `issue_rowbreak_chart_overlap` 2건을 깨뜨림
(native HWP5 한컴-정합 회귀 — render-diff 는 A/B 자기정합 지표라 못 잡는 축):

- `rowbreak_hwp_page12_reference_text_stays_inside_body`: p12 PartialTable(pi=11)
  42.5px overflow
- `rowbreak_page13_preserves_linear_empty_spacer_in_excerpt_table`: p13 첫 텍스트박스
  y=533.72 (< 572)

원인: rowbreak-problem-pages.hwp sec1 pi=2 빈 앵커(ls=1200HU)의 다음 문단 pi=3 이
**빈 TAC-표 앵커**(텍스트 없이 TAC 표만) — 표-표 스택이라 host_line_spacing 이
한컴 페이지 채움에 실제 계상되는 간격인데, #1133 보존 조건이 TopAndBottom 앵커만
인식해 억제가 발동, p11 fill 이 rows 0..6→0..7 로 어긋나 p12/p13 연쇄 붕괴.

수정: `para_is_empty_tac_table_anchor` 헬퍼 추가, 보존 조건을
`next = 빈 TopAndBottom 앵커 ∨ 빈 TAC-표 앵커` 로 확장. 판정은 여전히 소스 무관
→ 라운드트립 A==B 자기정합 불변 (seoul_0776 PASS 유지), #1147/#1133 원 케이스 불변.

검증: `issue_rowbreak_chart_overlap` 20/20 (HWPX/HWP5 쌍 포함), 전체
`cargo test --release` green (PR head + 수정 / devel merge + 수정 각각),
rustfmt·clippy clean.
