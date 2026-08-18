# 재현 트레이스

fixtures/traces/ 와 같은 id. argv 는 실명령.

## T01 — svg로 빼줘

명령: `export-svg`
메모: 시각 확인 1단
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T02 — 3쪽을 svg

명령: `export-svg`
메모: -p 2 (0 기준)
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T03 — debug overlay

명령: `export-svg`
메모: --debug-overlay
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T04 — 겹침 보이게

명령: `export-svg`
메모: --debug-overlay
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T05 — png로 비전 모델에

명령: `export-png`
메모: --vlm-target claude
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T06 — 인쇄용 pdf

명령: `export-pdf`
메모: --profile print 권장
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T07 — 본문만 텍스트

명령: `export-text`
메모: --json --max-chars 로 예산
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T08 — 마크다운으로

명령: `export-markdown`
메모: 표 병합은 별도
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T09 — 이 페이지 배치

명령: `dump-pages`
메모: 디버그 2단
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T10 — 문단 속성

명령: `dump`
메모: -s -p 는 구역/문단
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T11 — raw record

명령: `dump-records`
메모: HWP5 트리
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T12 — 번호가 이상해

명령: `diag`
메모: 개요/글머리표
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T13 — 몇 쪽이야

명령: `info`
메모: --json pageCount
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T14 — bbox 좌표

명령: `export-render-tree`
메모: 디버그 5단
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T15 — hwpx랑 hwp 비교

명령: `ir-diff`
메모: --json 이면 exit 3=차이
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T16 — 썸네일만

명령: `thumbnail`
메모: PrvImage, 렌더 아님
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T17 — 배포용 풀고 편집

명령: `convert`
메모: 출력은 .hwp
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T18 — 한컴 저장이랑 달라

명령: `hwp5-inventory-diff`
메모: oracle vs generated
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T19 — 표 저장 계약

명령: `hwp5-table-probe`
메모: hwp5 가족
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T20 — 특정 글자 주변 record

명령: `hwp5-anchor-trace`
메모: --needle
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T21 — CHAR_SHAPE 차이

명령: `hwp5-char-shape-audit`
메모: --out 필수
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T22 — 자기 라운드트립

명령: `hwp5-roundtrip`
메모: 한컴 호환이 아님
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T23 — 레이아웃 버그

명령: `export-svg`
메모: 디버그 1단부터
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T24 — 간격이 이상해

명령: `dump-pages`
메모: 2단 높이
페이지가 있으면 0 기준. 실패면 21장 봉투.

## T25 — 셀이 잘려

명령: `export-svg`
메모: overflowCellLines
페이지가 있으면 0 기준. 실패면 21장 봉투.
