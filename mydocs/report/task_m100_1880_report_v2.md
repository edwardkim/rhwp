# 최종 결과보고서 — Task M100 #1880

## 이슈

[#1880 빈-앵커 host_line_spacing 소스무관 억제가 convert-HWP 실제갭 과억제 (3075729 oracle p13 → rhwp convert-render p12)](https://github.com/edwardkim/rhwp/issues/1880)

## 요약

이슈 전제를 정정한다: 결함 축은 #1836 의 host_line_spacing 억제도, vpos 프레임
차이도 아니었다. **typeset 의 `wrap=1 → spacing_before 제외` 분기가 raw
`table.attr` 을 직독**해, 같은 IR 인데 HWPX 파스(attr=0, 미발동)와 HWP5 파스
(raw wrap 비트, 발동)가 갈라졌다. 빈-앵커 **스택**의 첫 앵커에서 sb 13.3px×2
가 convert 쪽만 빠져 heading 이 p12 로 오-페이지네이션. 스택 예외 추가로
양 인코딩 모두 **한컴 정합 p13** 수렴.

## 진단 경로 (이슈 추정 배제 과정)

1. 저장 vpos 갭 전수 대조(앵커 9곳): 두 인코딩 **완전 동일** — vpos 프레임
   차이 가설 기각. 모든 앵커에서 갭이 vpos 에 실재.
2. 플래그 프로브: conv 는 #1770 마커로 is_hwpx_variant=true → is_hwpx_source
   양쪽 true, is_hwp3_variant 양쪽 false — 소스 분기 가설 기각.
3. FLOW_DBG cur_h 사다리: 분기 = pi=104, 107 (빈-앵커 스택 첫 앵커) 각 +13.3px.
4. host_spacing 계측: suppress(#1836 억제)는 양쪽 동일. **before** 만 다름 —
   A(HWPX) sb 적용 13.3 / B(convert) wrap=1 분기로 0.0.

## 근본 원인

`table.attr` raw 비트 직독의 인코딩 불안정 (#1892 ③·#1842 와 같은
stale-pair 클래스): HWPX 파서는 enum 필드(text_wrap)만 채우고 attr=0,
HWP5 파서는 raw attr 보존. `wrap=1 sb 제외` 분기(자리차지 표 절대배치
전제)가 HWP5 파스에서만 발동 — 빈-앵커 스택은 절대배치가 아니라 flow 이므로
sb 가 한컴이 실제 계상하는 간격이다 (오라클 p13 = sb 보존).

## 수정

wrap=1 sb 제외 분기에 빈-앵커 스택 예외:
`!(is_topbottom_empty_anchor && next_is_empty_table_anchor)` —
아래 #1863 보존 규칙과 동일 조건쌍·동일 근거. HWPX 파스 경로는 원래
미발동이라 불변, convert/native HWP5 의 스택 케이스만 sb 보존으로 정렬.

## 검증

- 3075729: HWPX p13 불변 ✓ / convert-HWP p12 → **p13** (한컴 양 인코딩 p13 정합)
- 핀: `tests/issue_1880.rs` (13쪽 + Via::Hwp 왕복 구조 자기정합) + 픽스처 39KB
- razor-thin 인접 핀: rowbreak 20/20 (#1863 rowbreak-problem-pages 포함),
  issue_1133/1156/1488/1748/rowbreak_chart/1585 전부 PASS
- big_hwp 2,500 A/B: **완전 동일** (PASS 2495/OVER 5)
- big_hwpx 2,500 A/B: **완전 동일** (PASS 2483/STRUCT 9/OVER 8)
- 풀 스위트: PR CI 위임

## 남는 축 (후속 기록)

`table.attr`/`common.attr` raw 비트 직독처(is_tac 포함)의 인코딩 불안정은
시스템 축 — enum 필드 단일 권위화(파서별 attr 정규화)가 근본 해소이나
전면 감사가 필요한 설계 사안. 이슈의 "roundtrip proxy 개선 5건 oracle 재검"
요청은 본 수정이 #1836 억제를 건드리지 않으므로 별도 확인 불요.

## 산출물

- 수정: src/renderer/typeset.rs
- 픽스처·테스트: samples/issue1880_anchor_stack_sb_convert.hwpx, tests/issue_1880.rs
- 문서: plans/task_m100_1880.md, 본 보고서
