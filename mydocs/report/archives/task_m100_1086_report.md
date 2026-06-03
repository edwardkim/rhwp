# Task #1086 최종 보고서 — k-water-rfp.hwp +2 over-split 및 hwpspec.hwp 한컴 대조 정정

- 이슈: [edwardkim/rhwp#1086](https://github.com/edwardkim/rhwp/issues/1086)
- 브랜치: `local/task1086`
- 기준: `upstream/devel` `dae5c94f`

## 결과

`samples/k-water-rfp.hwp` 의 페이지 수를 한컴 PDF 정답과 같은 27 pages 로 정합했다. 이후 작업지시자 한컴 오피스 스크린샷 기준으로 `samples/hwpspec.hwp` 를 재분석해 20쪽 경계와 전체 178 pages 도 정합했다.

```text
k-water Before: 29 pages
k-water After:  27 pages
k-water Oracle: pdf/k-water-rfp-2022.pdf = 27 pages

hwpspec Before: 176/177 pages
hwpspec After:  178 pages
hwpspec Oracle: 작업지시자 한컴 오피스 스크린샷 = 178 pages
```

## 변경 요약

1. RowBreak + rowspan 표 블록에서 셀 내부 hard break 가 있는 경우만 block cut 을 허용했다.
2. block RowBreak 표의 셀 마지막 줄 trailing line spacing 을 fit 높이에서 제외했다.
3. TAC 표는 기존 geometry 를 보존해 KTX TOC snapshot 회귀를 막았다.
4. vpos reset paragraph 의 첫 줄이 현재 페이지에 들어갈 때 paragraph 전체 선행 page advance 를 차단했다.
5. HWP3-origin page tolerance 대상 문서에서만 `vpos=200/500 HU` near-top reset 을 인정했다.
6. 표지성 구역 시작 패턴에 한해 선행 빈 쪽을 삽입해 한컴의 홀수쪽 제목 정렬을 맞췄다.
7. `tests/issue_1086.rs` 를 추가해 k-water-rfp 27 pages, hwpspec 178 pages, hwpspec page 20/21 경계를 고정했다.

## 핵심 확인 지점

```text
k-water pi=52:  rows=0..4 / rows=2..4 with cut
k-water pi=66:  lines=0..1 / lines=1..2
k-water pi=180: rows=0..15 / rows=15..32
aift pi=767:    rows=0..20 / rows=20..30
aift pi=901:    rows=0..13 / rows=13..17
hwpspec page20: pi=78 rows=19..26 + pi=79..88, pi=89 없음
hwpspec page21: pi=89 확장 컨트롤 그림 시작
hwpspec Part II: page_num 57, "1. 개요" page_num 59, "변경 사항 이력" page_num 165
```

## 검증

```text
cargo fmt --all -- --check = pass
cargo build --release --bin rhwp = pass
cargo clippy --release --lib -- -D warnings = pass
cargo test --release --lib = pass
cargo test --release --test issue_1086 --test issue_554 = pass
cargo test --release --test issue_nested_table_border --test svg_snapshot = pass
```

직접 page count:

```text
samples/k-water-rfp.hwp = 27 pages
samples/aift.hwp = 74 pages
samples/hwpspec.hwp = 178 pages
samples/2022년 국립국어원 업무계획.hwp = 35 pages
```

## 비고

통합 테스트 중 기존 테스트 코드 warning 6건이 출력되지만, 본 변경의 `clippy -D warnings` 게이트는 통과한다.
