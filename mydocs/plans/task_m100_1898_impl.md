# 구현 계획서 — Task M100 #1898 (기전 1)

## 근본 원인

`layout.rs` 의 항목 후처리(#409/#1027 계열)가 `PageItem::Shape` 전부에 대해
vpos 기준점(page/lazy base)을 초기화한다. 근거는 "표/Shape 의 LINE_SEG lh 는
개체 높이를 포함해 순차 y 와 drift" — 이는 **tac-전용 문단**(빈 텍스트,
lh=개체 높이)에는 맞지만, **텍스트 줄에 통합된 tac 개체**(불릿 그림)에는
성립하지 않는다(호스트 LINE_SEG 는 텍스트 줄 높이, Shape 항목 dy=0).

초기화되면 다음 문단의 `HeightCursor::vpos_adjust` 가 base 부재 → lazy_base
재산출 경로 진입 → 직전 문단과 vpos 가 불연속(spacing_before 500HU 갭)이므로
trailing-ls bridge(+880HU=11.7px) 적용 → 불릿 문단마다 렌더 y 과대 전진.
문서 전반에 불릿이 많아 페이지 내 누적 밀림(참고자료 리스트 +44.8px/줄)이 커진다.

## 수정

`layout.rs` 초기화 조건에 예외 추가:

```
is_inline_tac_object = PageItem::Shape 이고
  호스트 문단이 실제 텍스트 보유(para_has_visible_text)
  ∧ 해당 컨트롤(Picture/Shape/Equation)의 treat_as_char=true
→ 기준점 초기화 생략
```

- 기존 예외(Para-float 표)와 같은 구조로 병렬 배치.
- tac-전용 문단(sample16 pi=71 RFP 박스)은 텍스트 부재로 예외 미적용 —
  issue_1116 한컴 핀 2건 보존 (1차 blanket 면제안이 이 핀에서 −8.5px 회귀,
  텍스트 보유 조건으로 정제).

## 계측 개선 (동반)

`RHWP_DEBUG_PARA_TAC` 의 대상 pi 가 651/652 하드코딩이던 것을 콤마 목록
환경변수 값으로 일반화 (빈 값/`all` = 전체).

## 검증 계획

- 36388711 p9: TextLine 간격 44.8 → 33.1px (한컴 32.9) + 핀 테스트
- issue_1116 (sample16 한컴 핀) 13/13
- cargo test 전 스위트
- big_hwp/big_hwpx 2,500×2 A/B (origin/devel 054be69c 베이스 exe)
