# Task #1116 Stage 18 분석 보고서 — k-water-rfp-2024 p5 표 절단 한 줄 차이

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 원인 재분류 완료, 소스 수정 전 승인 대기

## 1. 추가 확인

작업지시자가 `k-water-rfp-2024.hwp`의 한컴오피스 3mm 격자 화면을 다시 제공했다.

Stage 17에서는 `pi=52` RowBreak 표가 대상이라는 점까지는 맞게 잡았지만, 실제 차이를 p6 상단 이어짐 위치로만 보았다. 이번 3mm 격자에서는 p5 하단 절단 위치가 더 직접적인 단서다.

## 2. 현재 RHWP

대상:

```text
samples/k-water-rfp-2024.hwp
global page 5, section=1, page_num=3
paragraph pi=52, 4x4 RowBreak table
```

현재 RHWP p5:

```text
PartialTable   pi=52 ci=0  rows=0..4  cont=false  4x4  vpos=12480
start_cut=[]
end_cut=[3, 4, 2, 4, 4, 2, 21]
```

SVG 주요 좌표:

| 항목 | RHWP y |
| --- | ---: |
| 표 상단선 | 279.7px |
| `품목매체분야세부내용` | 295.2px |
| `㉯ 발표자료` | 885.4px |
| `◦ 규격 : A4 가로방향...` | 973.5px |
| `◦ 유의사항 : 그림·사진...` | 995.5px |
| 표 하단선 | 1006.9px |

## 3. 한컴 3mm 격자 기준

첨부 한컴오피스 3mm 격자 p5는 `pi=52` 표가 `◦ 규격 : A4 가로방향(297mm x 210mm)` 근처에서 끝나고, RHWP처럼 `◦ 유의사항 : 그림·사진...` 첫 줄까지 p5에 보이지 않는다.

따라서 실제 차이는:

```text
RHWP: p5에 p17 line0 (`◦ 유의사항...`)까지 표시
한컴: p5는 p16 (`◦ 규격 : A4 가로방향...`)에서 절단
```

## 4. end_cut 해석

`cell[14]`의 단위 인덱스:

| unit | 의미 |
| ---: | --- |
| 15 | p12 `㉯ 발표자료` |
| 16 | p13 `◦ 기준쪽수...` |
| 17 | p14 `◦ 색도...` |
| 18 | p15 `◦ 표지...` |
| 19 | p16 `◦ 규격 : A4 가로방향...` |
| 20 | p17 line0 `◦ 유의사항 : 그림·사진...` |
| 21 | p17 line1 `금지, 제안서에 없는 내용 포함 금지` — 내부 vpos reset |

현재 `end_cut` 마지막 값은 `21`이므로 unit `0..20`이 p5에 표시된다. 한컴 3mm 격자 기준으로는 마지막 값이 `20`이어야 p5가 unit `19`에서 끝난다.

```text
현재: end_cut=[3, 4, 2, 4, 4, 2, 21]
기대: end_cut=[3, 4, 2, 4, 4, 2, 20]
```

## 5. 구현 후보

소스 수정 전 승인 필요.

후속 승인 시 수정 후보:

1. `src/renderer/layout/table_layout.rs::advance_row_block_cut`에서 RowBreak rowspan 블록의 `hard_break_before`를 처리할 때, 같은 문단 안의 다음 unit이 hard break이면 직전 unit만 p5 하단에 고립시키지 않도록 한다.
2. 이 케이스에서는 p17 line1의 reset을 보고 p17 line0까지 p5에 넣는 대신, p17 전체를 다음 fragment로 넘긴다.
3. `tests/issue_1105.rs`에 `k-water-rfp-2024.hwp` p5의 `end_cut=[3, 4, 2, 4, 4, 2, 20]` 및 p5에 `유의사항`이 나타나지 않는 회귀 테스트를 추가한다.
4. 기존 `k-water-rfp.hwp`, sample16 계열, `issue_1086`, `issue_1116` 회귀 테스트를 함께 확인한다.

## 6. 현재 중단점

분석은 여기까지 완료했다. 소스 수정과 테스트 추가는 작업지시자 승인 후 진행한다.
