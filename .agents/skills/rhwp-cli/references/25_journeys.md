# 실사용 여정

에이전트가 닫는 짧은 길이다. 전부 기존 CLI.

## J01 — 3쪽 겹침

첫 명령: `export-svg --debug-overlay -p 2`
정지: 라벨로 문단 특정
gym 경로 없음. 새 플래그 없음.

## J02 — 인쇄 PDF

첫 명령: `export-pdf --profile print`
정지: 폰트 경로 명시
gym 경로 없음. 새 플래그 없음.

## J03 — VLM 입력

첫 명령: `export-png --vlm-target claude -p 0`
정지: skia 게이트
gym 경로 없음. 새 플래그 없음.

## J04 — 본문 예산

첫 명령: `export-text --json --max-chars 4000`
정지: truncated 읽기
gym 경로 없음. 새 플래그 없음.

## J05 — 마크다운 초안

첫 명령: `export-markdown -p 0`
정지: 표는 따로
gym 경로 없음. 새 플래그 없음.

## J06 — 쪽 배치

첫 명령: `dump-pages -p 0`
정지: vpos
gym 경로 없음. 새 플래그 없음.

## J07 — 문단 속성

첫 명령: `dump -s 0 -p 3`
정지: LINE_SEG
gym 경로 없음. 새 플래그 없음.

## J08 — raw 트리

첫 명령: `dump-records`
정지: 암호면 stdin
gym 경로 없음. 새 플래그 없음.

## J09 — 번호 이상

첫 명령: `diag`
정지: 개요
gym 경로 없음. 새 플래그 없음.

## J10 — 규모

첫 명령: `info --json`
정지: pageCount
gym 경로 없음. 새 플래그 없음.

## J11 — bbox

첫 명령: `export-render-tree -p 0`
정지: translate
gym 경로 없음. 새 플래그 없음.

## J12 — 형식 쌍

첫 명령: `ir-diff a.hwpx b.hwp --json`
정지: exit 3 데이터
gym 경로 없음. 새 플래그 없음.

## J13 — 썸네일

첫 명령: `thumbnail --data-uri`
정지: PrvImage
gym 경로 없음. 새 플래그 없음.

## J14 — 배포용 해제

첫 명령: `convert in.hwp out.hwp --verify`
정지: exit 3 자기검증
gym 경로 없음. 새 플래그 없음.

## J15 — 저장 계약

첫 명령: `hwp5-inventory-diff oracle generated`
정지: 순서
gym 경로 없음. 새 플래그 없음.

## J16 — 표 probe

첫 명령: `hwp5-table-probe`
정지: out-dir
gym 경로 없음. 새 플래그 없음.

## J17 — 글자 주변

첫 명령: `hwp5-anchor-trace --needle`
정지: section 0
gym 경로 없음. 새 플래그 없음.

## J18 — CHAR_SHAPE

첫 명령: `hwp5-char-shape-audit --out`
정지: written:
gym 경로 없음. 새 플래그 없음.

## J19 — 없는 파일

첫 명령: `export-svg missing.hwp`
정지: exit 1
gym 경로 없음. 새 플래그 없음.

## J20 — 쪽 초과

첫 명령: `export-svg -p 99`
정지: exit 2
gym 경로 없음. 새 플래그 없음.

## J21 — PNG 부재

첫 명령: `export-png`
정지: exit 2 메시지
gym 경로 없음. 새 플래그 없음.

## J22 — 깨진 OLE

첫 명령: `info truncated.hwp`
정지: 파싱 실패
gym 경로 없음. 새 플래그 없음.

## J23 — 셀 소실

첫 명령: `export-svg --json`
정지: overflowCellLines
gym 경로 없음. 새 플래그 없음.

## J24 — 전후 SVG

첫 명령: `export-svg before/after`
정지: tree diff
gym 경로 없음. 새 플래그 없음.
