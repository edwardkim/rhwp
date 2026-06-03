# Task #1116 Stage 19 보고서 — k-water-rfp-2024 p5 RowBreak 표 절단 보정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 승인 후 구현 및 검증 완료

## 1. 승인 범위

작업지시자가 Stage 18의 `k-water-rfp-2024.hwp` p5 표 절단 한 줄 차이 보정 진행을 승인했다.

대상은 전역 5쪽, 섹션 1 page_num 3의 `pi=52` 4x4 RowBreak 표다. 한컴오피스 3mm 격자 기준으로 p5 하단은 `◦ 규격 : A4 가로방향...`에서 끝나야 하며, RHWP처럼 다음 `◦ 유의사항...` 첫 줄이 p5에 남으면 안 된다.

## 2. 수정

수정 파일:

```text
src/renderer/layout/table_layout.rs
tests/issue_1105.rs
```

변경:

1. RowBreak 표의 cut 계산에서 hard-break 직전 unit이 같은 문단의 직전 줄이면, 첫 조각에 그 줄만 고립시키지 않도록 한 unit 되감는다.
2. `advance_row_cut`과 `advance_row_block_cut`에서 같은 helper를 사용하도록 했다.
3. `k-water-rfp-2024.hwp` p5 `pi=52` 표 절단 기대값을 `end_cut=[3, 4, 2, 4, 4, 2, 20]`으로 고정했다.
4. p5 SVG에서 `A4 가로방향` 행은 남고 `유의사항` 행은 남지 않는지 회귀 테스트를 추가했다.

## 3. 결과

수정 후 p5 dump:

```text
PartialTable   pi=52 ci=0  rows=0..4  cont=false  4x4  vpos=12480  start_cut=[] end_cut=[3, 4, 2, 4, 4, 2, 20]
```

수정 후 p5 SVG 주요 행:

```text
885.4  ㉯발표자료
973.5  ◦규격:A4가로방향(297mmx210mm)
```

`◦ 유의사항...` 행은 p5에서 제외된다.

산출물:

```text
output/poc/render-spacing/k-water-rfp-2024-stage19-page5-after-cut/k-water-rfp-2024_005.svg
```

## 4. 검증

완료:

1. `cargo test --test issue_1105 -- --nocapture`
2. `cargo test --test issue_1086 -- --nocapture`
3. `cargo test --test issue_1116 -- --nocapture`
4. `cargo test --test issue_1035_alignment -- --nocapture`
5. `cargo test --test issue_713 -- --nocapture`
6. `cargo fmt --all -- --check`
7. `cargo build --bin rhwp`
8. `git diff --check`
