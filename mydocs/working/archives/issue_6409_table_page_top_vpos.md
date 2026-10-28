---
kind: working
status: active
issue: 6409
---

# 저장 vpos=0 TAC 표를 leftover 에 끼우지 않는다 (#6409)

브랜치: `fix/6409-table-page-top-vpos` (`upstream/devel` 격리 worktree)

## 한 줄

HWPX 가 글자처럼 취급 표를 쪽높이급 한 줄로 저장했고 그 높이가 leftover 에
안 들어가면, CellBreak 로 끼우지 않고 다음 쪽 상단에서 시작한다.
원본 XML vertpos=0 은 typeset 전 누적 vpos(524475)로 덮이므로 줄 높이를 본다.

## 실측

`samples/issue6031/3249937_asset_management_rules.hwpx`

- 41쪽(0-based 40): `<붙임 4>`(pageBreak=1) + 25×34 표(vertpos=2800)
- leftover ~210px 에 39×22 부동산거래계약 신고서(vertpos=0, vertsize=69344)가
  끼어 11행이 앞쪽으로 온다. 한글은 그 표를 42쪽 상단에 둔다.
- layout-anomaly 의 38.69px 는 Table2 **우측** 초과(선언 폭 51092 vs 단 48188).
  쪽 경계 문자 +199 는 별도 기전이다.

## 범위

- `src/renderer/typeset.rs` — 첫 행 이월 게이트에 저장 쪽-상단 TAC 표 조건
- `tests/cases/issue_6409_table_page_top_vpos.rs`

## 비범위

- 표 폭 38.69px 우측 초과(`noAdjust=1`) — 쪽 경계와 다른 축
- gym, 새 CLI, DocumentCore 편집 발명
