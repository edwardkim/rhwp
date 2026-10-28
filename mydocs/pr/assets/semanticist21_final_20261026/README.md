# 통합 후보 전쪽 시각 검증 — 2026-10-26

고정한 45개 원 PR과 이미 승인된 리베이스 검증 입력만 포함한다. 다른 실물 HWP로 범위를 넓히지 않았다.
생산 코드는 `a90a2057a7e967a617c043c62e71f4fab26186df`, base는 `238de6de41a208648f4ade6b494fe433a81c1fce`다.
Native 257과 최적화 fresh WASM 074로 실제 재출력했다. 이후 검사·문서 정리는 생산 코드를 바꾸지 않았다.

**이 묶음은 완료한 통과 자료의 부분집합이다. 원 PR 재현 57조건의 추가 후보/base 대조와 개별 PR 판정은 별도로 진행 중이며, 이 묶음만으로 PR 준비 완료를 선언하지 않는다.**

129조건·각 344쪽에서 90% 미만·누락·쪽수 불일치는 0쪽이다. 최저 raw는 Native 92.04708%(4599 p34), fresh WASM 91.53306%(6970 p3)다.
같은 쪽을 대응할 수 있는 684행에서 Decimal로 계산한 base 대비 0.1%p 초과 하락은 0행이다.
#6133은 base 1쪽을 독립 Print와 같은 2쪽으로 복원한 구조 개선이다. 대응할 수 없는 base 둘째 쪽의 점수를 만들지 않았다.
이전 [쪽수·내용 보존 근거](../semanticist21_rebase_qualified_20261026/README.md#6133의-쪽수-복원)와 현재 양쪽 전쪽 raw 100%를 함께 확인한다.

## 전쪽 TSV 원문

파일은 원 실행의 TSV 바이트 그대로다. base 열이 비어 있는 #6133은 위 구조 대조를 따른다. 입력·PDF·실제 명령·producer·글꼴·SHA는 [provenance](provenance.json)에 있다.
`--compat 2022`는 RHWP의 호환 세대이며 한컴 2022에서 출력했다는 뜻이 아니다. 독립 Print의 실제 2020/2024 엔진 출처는 입력별 기록을 따른다.

| 번호 | 조건 | 쪽 | Native 후보 | Native base | fresh WASM 후보 | fresh WASM base |
| --- | --- | ---: | --- | --- | --- | --- |
| 1 | 1510-hwp | 1 | [100.00000%](native/candidate/001.tsv) | [TSV](native/base/001.tsv) | [100.00000%](wasm/candidate/001.tsv) | [TSV](wasm/base/001.tsv) |
| 2 | 1510 | 2 | [100.00000%](native/candidate/002.tsv) | [TSV](native/base/002.tsv) | [100.00000%](wasm/candidate/002.tsv) | [TSV](wasm/base/002.tsv) |
| 3 | 1535 | 2 | [100.00000%](native/candidate/003.tsv) | [TSV](native/base/003.tsv) | [100.00000%](wasm/candidate/003.tsv) | [TSV](wasm/base/003.tsv) |
| 4 | 6133 | 2 | [100.00000%](native/candidate/004.tsv) | 구조 대조 | [100.00000%](wasm/candidate/004.tsv) | 구조 대조 |
| 5 | 6970 | 3 | [99.79447%](native/candidate/005.tsv) | [TSV](native/base/005.tsv) | [91.53306%](wasm/candidate/005.tsv) | [TSV](wasm/base/005.tsv) |
| 6 | field-reset | 2 | [100.00000%](native/candidate/006.tsv) | [TSV](native/base/006.tsv) | [100.00000%](wasm/candidate/006.tsv) | [TSV](wasm/base/006.tsv) |
| 7 | field-plain | 2 | [100.00000%](native/candidate/007.tsv) | [TSV](native/base/007.tsv) | [100.00000%](wasm/candidate/007.tsv) | [TSV](wasm/base/007.tsv) |
| 8 | titled-host | 4 | [100.00000%](native/candidate/008.tsv) | [TSV](native/base/008.tsv) | [100.00000%](wasm/candidate/008.tsv) | [TSV](wasm/base/008.tsv) |
| 9 | saved-host | 4 | [100.00000%](native/candidate/009.tsv) | [TSV](native/base/009.tsv) | [100.00000%](wasm/candidate/009.tsv) | [TSV](wasm/base/009.tsv) |
| 10 | omitted-after | 2 | [100.00000%](native/candidate/010.tsv) | [TSV](native/base/010.tsv) | [100.00000%](wasm/candidate/010.tsv) | [TSV](wasm/base/010.tsv) |
| 11 | symbol-slot-2020 | 1 | [100.00000%](native/candidate/011.tsv) | [TSV](native/base/011.tsv) | [100.00000%](wasm/candidate/011.tsv) | [TSV](wasm/base/011.tsv) |
| 12 | symbol-slot-2024 | 1 | [100.00000%](native/candidate/012.tsv) | [TSV](native/base/012.tsv) | [100.00000%](wasm/candidate/012.tsv) | [TSV](wasm/base/012.tsv) |
| 13 | 4599 | 64 | [92.04708%](native/candidate/013.tsv) | [TSV](native/base/013.tsv) | [92.02762%](wasm/candidate/013.tsv) | [TSV](wasm/base/013.tsv) |
| 14 | 5846 | 66 | [97.15069%](native/candidate/014.tsv) | [TSV](native/base/014.tsv) | [97.06823%](wasm/candidate/014.tsv) | [TSV](wasm/base/014.tsv) |
| 15 | 7518 | 6 | [99.96132%](native/candidate/015.tsv) | [TSV](native/base/015.tsv) | [99.95693%](wasm/candidate/015.tsv) | [TSV](wasm/base/015.tsv) |
| 16 | 8060 | 48 | [96.73952%](native/candidate/016.tsv) | [TSV](native/base/016.tsv) | [96.69927%](wasm/candidate/016.tsv) | [TSV](wasm/base/016.tsv) |
| 17 | 6448 | 3 | [99.05213%](native/candidate/017.tsv) | [TSV](native/base/017.tsv) | [99.30014%](wasm/candidate/017.tsv) | [TSV](wasm/base/017.tsv) |
| 18 | pr7588-pr8042-group-0-hwpx | 1 | [100.00000%](native/candidate/018.tsv) | [TSV](native/base/018.tsv) | [100.00000%](wasm/candidate/018.tsv) | [TSV](wasm/base/018.tsv) |
| 19 | pr7588-pr8042-group-30-hwpx | 1 | [100.00000%](native/candidate/019.tsv) | [TSV](native/base/019.tsv) | [100.00000%](wasm/candidate/019.tsv) | [TSV](wasm/base/019.tsv) |
| 20 | pr7588-pr8042-ungroup-0-hwpx | 1 | [100.00000%](native/candidate/020.tsv) | [TSV](native/base/020.tsv) | [100.00000%](wasm/candidate/020.tsv) | [TSV](wasm/base/020.tsv) |
| 21 | pr7588-pr8042-ungroup-30-hwpx | 1 | [100.00000%](native/candidate/021.tsv) | [TSV](native/base/021.tsv) | [100.00000%](wasm/candidate/021.tsv) | [TSV](wasm/base/021.tsv) |
| 22 | pr8041-case-1-after-hwpx | 1 | [100.00000%](native/candidate/022.tsv) | [TSV](native/base/022.tsv) | [100.00000%](wasm/candidate/022.tsv) | [TSV](wasm/base/022.tsv) |
| 23 | pr8041-case-1-before-hwpx | 1 | [100.00000%](native/candidate/023.tsv) | [TSV](native/base/023.tsv) | [100.00000%](wasm/candidate/023.tsv) | [TSV](wasm/base/023.tsv) |
| 24 | pr8041-case-2-after-hwpx | 1 | [100.00000%](native/candidate/024.tsv) | [TSV](native/base/024.tsv) | [100.00000%](wasm/candidate/024.tsv) | [TSV](wasm/base/024.tsv) |
| 25 | pr8041-case-2-before-hwpx | 1 | [100.00000%](native/candidate/025.tsv) | [TSV](native/base/025.tsv) | [100.00000%](wasm/candidate/025.tsv) | [TSV](wasm/base/025.tsv) |
| 26 | pr8041-case-3-base-0-after-hwpx | 1 | [100.00000%](native/candidate/026.tsv) | [TSV](native/base/026.tsv) | [100.00000%](wasm/candidate/026.tsv) | [TSV](wasm/base/026.tsv) |
| 27 | pr8041-case-3-base-0-before-hwpx | 1 | [100.00000%](native/candidate/027.tsv) | [TSV](native/base/027.tsv) | [100.00000%](wasm/candidate/027.tsv) | [TSV](wasm/base/027.tsv) |
| 28 | pr8041-case-3-base-1-after-hwpx | 1 | [100.00000%](native/candidate/028.tsv) | [TSV](native/base/028.tsv) | [100.00000%](wasm/candidate/028.tsv) | [TSV](wasm/base/028.tsv) |
| 29 | pr8041-case-3-base-1-before-hwpx | 1 | [100.00000%](native/candidate/029.tsv) | [TSV](native/base/029.tsv) | [100.00000%](wasm/candidate/029.tsv) | [TSV](wasm/base/029.tsv) |
| 30 | pr8041-case-3-base-2-after-hwpx | 1 | [100.00000%](native/candidate/030.tsv) | [TSV](native/base/030.tsv) | [100.00000%](wasm/candidate/030.tsv) | [TSV](wasm/base/030.tsv) |
| 31 | pr8041-case-3-base-2-before-hwpx | 1 | [100.00000%](native/candidate/031.tsv) | [TSV](native/base/031.tsv) | [100.00000%](wasm/candidate/031.tsv) | [TSV](wasm/base/031.tsv) |
| 32 | pr7588-pr8042-group-0-hwp | 1 | [100.00000%](native/candidate/032.tsv) | [TSV](native/base/032.tsv) | [100.00000%](wasm/candidate/032.tsv) | [TSV](wasm/base/032.tsv) |
| 33 | pr7588-pr8042-group-30-hwp | 1 | [100.00000%](native/candidate/033.tsv) | [TSV](native/base/033.tsv) | [100.00000%](wasm/candidate/033.tsv) | [TSV](wasm/base/033.tsv) |
| 34 | pr7588-pr8042-ungroup-0-hwp | 1 | [100.00000%](native/candidate/034.tsv) | [TSV](native/base/034.tsv) | [100.00000%](wasm/candidate/034.tsv) | [TSV](wasm/base/034.tsv) |
| 35 | pr7588-pr8042-ungroup-30-hwp | 1 | [100.00000%](native/candidate/035.tsv) | [TSV](native/base/035.tsv) | [100.00000%](wasm/candidate/035.tsv) | [TSV](wasm/base/035.tsv) |
| 36 | pr8041-case-1-after-hwp | 1 | [100.00000%](native/candidate/036.tsv) | [TSV](native/base/036.tsv) | [100.00000%](wasm/candidate/036.tsv) | [TSV](wasm/base/036.tsv) |
| 37 | pr8041-case-1-before-hwp | 1 | [100.00000%](native/candidate/037.tsv) | [TSV](native/base/037.tsv) | [100.00000%](wasm/candidate/037.tsv) | [TSV](wasm/base/037.tsv) |
| 38 | pr8041-case-2-after-hwp | 1 | [100.00000%](native/candidate/038.tsv) | [TSV](native/base/038.tsv) | [100.00000%](wasm/candidate/038.tsv) | [TSV](wasm/base/038.tsv) |
| 39 | pr8041-case-2-before-hwp | 1 | [100.00000%](native/candidate/039.tsv) | [TSV](native/base/039.tsv) | [100.00000%](wasm/candidate/039.tsv) | [TSV](wasm/base/039.tsv) |
| 40 | pr8041-case-3-base-0-after-hwp | 1 | [100.00000%](native/candidate/040.tsv) | [TSV](native/base/040.tsv) | [100.00000%](wasm/candidate/040.tsv) | [TSV](wasm/base/040.tsv) |
| 41 | pr8041-case-3-base-0-before-hwp | 1 | [100.00000%](native/candidate/041.tsv) | [TSV](native/base/041.tsv) | [100.00000%](wasm/candidate/041.tsv) | [TSV](wasm/base/041.tsv) |
| 42 | pr8041-case-3-base-1-after-hwp | 1 | [100.00000%](native/candidate/042.tsv) | [TSV](native/base/042.tsv) | [100.00000%](wasm/candidate/042.tsv) | [TSV](wasm/base/042.tsv) |
| 43 | pr8041-case-3-base-1-before-hwp | 1 | [100.00000%](native/candidate/043.tsv) | [TSV](native/base/043.tsv) | [100.00000%](wasm/candidate/043.tsv) | [TSV](wasm/base/043.tsv) |
| 44 | pr8041-case-3-base-2-after-hwp | 1 | [100.00000%](native/candidate/044.tsv) | [TSV](native/base/044.tsv) | [100.00000%](wasm/candidate/044.tsv) | [TSV](wasm/base/044.tsv) |
| 45 | pr8041-case-3-base-2-before-hwp | 1 | [100.00000%](native/candidate/045.tsv) | [TSV](native/base/045.tsv) | [100.00000%](wasm/candidate/045.tsv) | [TSV](wasm/base/045.tsv) |
| 46 | ta-pic-001-r | 1 | [99.98107%](native/candidate/046.tsv) | [TSV](native/base/046.tsv) | [99.98109%](wasm/candidate/046.tsv) | [TSV](wasm/base/046.tsv) |
| 47 | ta-pic-caption-tight | 1 | [99.98107%](native/candidate/047.tsv) | [TSV](native/base/047.tsv) | [99.98110%](wasm/candidate/047.tsv) | [TSV](wasm/base/047.tsv) |
| 48 | group-after-hwp | 1 | [100.00000%](native/candidate/048.tsv) | [TSV](native/base/048.tsv) | [100.00000%](wasm/candidate/048.tsv) | [TSV](wasm/base/048.tsv) |
| 49 | group-after-hwpx | 1 | [100.00000%](native/candidate/049.tsv) | [TSV](native/base/049.tsv) | [100.00000%](wasm/candidate/049.tsv) | [TSV](wasm/base/049.tsv) |
| 50 | group-before-hwp | 1 | [100.00000%](native/candidate/050.tsv) | [TSV](native/base/050.tsv) | [100.00000%](wasm/candidate/050.tsv) | [TSV](wasm/base/050.tsv) |
| 51 | group-before-hwpx | 1 | [100.00000%](native/candidate/051.tsv) | [TSV](native/base/051.tsv) | [100.00000%](wasm/candidate/051.tsv) | [TSV](wasm/base/051.tsv) |
| 52 | ungroup-after-hwp | 1 | [100.00000%](native/candidate/052.tsv) | [TSV](native/base/052.tsv) | [100.00000%](wasm/candidate/052.tsv) | [TSV](wasm/base/052.tsv) |
| 53 | ungroup-after-hwpx | 1 | [100.00000%](native/candidate/053.tsv) | [TSV](native/base/053.tsv) | [100.00000%](wasm/candidate/053.tsv) | [TSV](wasm/base/053.tsv) |
| 54 | ungroup-before-hwp | 1 | [100.00000%](native/candidate/054.tsv) | [TSV](native/base/054.tsv) | [100.00000%](wasm/candidate/054.tsv) | [TSV](wasm/base/054.tsv) |
| 55 | ungroup-before-hwpx | 1 | [100.00000%](native/candidate/055.tsv) | [TSV](native/base/055.tsv) | [100.00000%](wasm/candidate/055.tsv) | [TSV](wasm/base/055.tsv) |
| 56 | emphasis-boundary-241-split | 1 | [100.00000%](native/candidate/056.tsv) | [TSV](native/base/056.tsv) | [100.00000%](wasm/candidate/056.tsv) | [TSV](wasm/base/056.tsv) |
| 57 | emphasis-style-boundary-equal-properties-distinct-ids | 1 | [100.00000%](native/candidate/057.tsv) | [TSV](native/base/057.tsv) | [100.00000%](wasm/candidate/057.tsv) | [TSV](wasm/base/057.tsv) |
| 58 | emphasis-style-boundary-shared-source-style | 1 | [100.00000%](native/candidate/058.tsv) | [TSV](native/base/058.tsv) | [100.00000%](wasm/candidate/058.tsv) | [TSV](wasm/base/058.tsv) |
| 59 | host-size-1000-offset-0-hwp | 1 | [100.00000%](native/candidate/059.tsv) | [TSV](native/base/059.tsv) | [100.00000%](wasm/candidate/059.tsv) | [TSV](wasm/base/059.tsv) |
| 60 | host-size-1000-offset-0-hwpx | 1 | [100.00000%](native/candidate/060.tsv) | [TSV](native/base/060.tsv) | [100.00000%](wasm/candidate/060.tsv) | [TSV](wasm/base/060.tsv) |
| 61 | host-size-1000-offset-1700-hwp | 1 | [100.00000%](native/candidate/061.tsv) | [TSV](native/base/061.tsv) | [100.00000%](wasm/candidate/061.tsv) | [TSV](wasm/base/061.tsv) |
| 62 | host-size-1000-offset-1700-hwpx | 1 | [100.00000%](native/candidate/062.tsv) | [TSV](native/base/062.tsv) | [100.00000%](wasm/candidate/062.tsv) | [TSV](wasm/base/062.tsv) |
| 63 | host-size-250-offset-0-hwp | 1 | [100.00000%](native/candidate/063.tsv) | [TSV](native/base/063.tsv) | [100.00000%](wasm/candidate/063.tsv) | [TSV](wasm/base/063.tsv) |
| 64 | host-size-250-offset-0-hwpx | 1 | [100.00000%](native/candidate/064.tsv) | [TSV](native/base/064.tsv) | [100.00000%](wasm/candidate/064.tsv) | [TSV](wasm/base/064.tsv) |
| 65 | host-size-250-offset-1700-hwp | 1 | [100.00000%](native/candidate/065.tsv) | [TSV](native/base/065.tsv) | [100.00000%](wasm/candidate/065.tsv) | [TSV](wasm/base/065.tsv) |
| 66 | host-size-250-offset-1700-hwpx | 1 | [100.00000%](native/candidate/066.tsv) | [TSV](native/base/066.tsv) | [100.00000%](wasm/candidate/066.tsv) | [TSV](wasm/base/066.tsv) |
| 67 | host-size-4000-offset-0-hwp | 1 | [100.00000%](native/candidate/067.tsv) | [TSV](native/base/067.tsv) | [100.00000%](wasm/candidate/067.tsv) | [TSV](wasm/base/067.tsv) |
| 68 | host-size-4000-offset-0-hwpx | 1 | [100.00000%](native/candidate/068.tsv) | [TSV](native/base/068.tsv) | [100.00000%](wasm/candidate/068.tsv) | [TSV](wasm/base/068.tsv) |
| 69 | host-size-4000-offset-1700-hwp | 1 | [100.00000%](native/candidate/069.tsv) | [TSV](native/base/069.tsv) | [100.00000%](wasm/candidate/069.tsv) | [TSV](wasm/base/069.tsv) |
| 70 | host-size-4000-offset-1700-hwpx | 1 | [100.00000%](native/candidate/070.tsv) | [TSV](native/base/070.tsv) | [100.00000%](wasm/candidate/070.tsv) | [TSV](wasm/base/070.tsv) |
| 71 | unsplit-rows-2-body-true-flow-false-hwp | 1 | [100.00000%](native/candidate/071.tsv) | [TSV](native/base/071.tsv) | [100.00000%](wasm/candidate/071.tsv) | [TSV](wasm/base/071.tsv) |
| 72 | unsplit-rows-2-body-true-flow-false-hwpx | 1 | [100.00000%](native/candidate/072.tsv) | [TSV](native/base/072.tsv) | [100.00000%](wasm/candidate/072.tsv) | [TSV](wasm/base/072.tsv) |
| 73 | raised-dot | 1 | [100.00000%](native/candidate/073.tsv) | [TSV](native/base/073.tsv) | [100.00000%](wasm/candidate/073.tsv) | [TSV](wasm/base/073.tsv) |
| 74 | multiline-true | 1 | [100.00000%](native/candidate/074.tsv) | [TSV](native/base/074.tsv) | [100.00000%](wasm/candidate/074.tsv) | [TSV](wasm/base/074.tsv) |
| 75 | multiline-false | 1 | [100.00000%](native/candidate/075.tsv) | [TSV](native/base/075.tsv) | [100.00000%](wasm/candidate/075.tsv) | [TSV](wasm/base/075.tsv) |
| 76 | offset-adapter-hwp | 1 | [100.00000%](native/candidate/076.tsv) | [TSV](native/base/076.tsv) | [100.00000%](wasm/candidate/076.tsv) | [TSV](wasm/base/076.tsv) |
| 77 | offset-hwpx | 1 | [100.00000%](native/candidate/077.tsv) | [TSV](native/base/077.tsv) | [100.00000%](wasm/candidate/077.tsv) | [TSV](wasm/base/077.tsv) |
| 78 | padding-adapter-hwp | 1 | [100.00000%](native/candidate/078.tsv) | [TSV](native/base/078.tsv) | [100.00000%](wasm/candidate/078.tsv) | [TSV](wasm/base/078.tsv) |
| 79 | padding-hwpx | 1 | [100.00000%](native/candidate/079.tsv) | [TSV](native/base/079.tsv) | [100.00000%](wasm/candidate/079.tsv) | [TSV](wasm/base/079.tsv) |
| 80 | caret-nonbmp-endnote-false-hwp | 1 | [100.00000%](native/candidate/080.tsv) | [TSV](native/base/080.tsv) | [100.00000%](wasm/candidate/080.tsv) | [TSV](wasm/base/080.tsv) |
| 81 | caret-nonbmp-endnote-false-hwpx | 1 | [100.00000%](native/candidate/081.tsv) | [TSV](native/base/081.tsv) | [100.00000%](wasm/candidate/081.tsv) | [TSV](wasm/base/081.tsv) |
| 82 | caret-nonbmp-endnote-true-hwp | 1 | [100.00000%](native/candidate/082.tsv) | [TSV](native/base/082.tsv) | [100.00000%](wasm/candidate/082.tsv) | [TSV](wasm/base/082.tsv) |
| 83 | caret-nonbmp-endnote-true-hwpx | 1 | [100.00000%](native/candidate/083.tsv) | [TSV](native/base/083.tsv) | [100.00000%](wasm/candidate/083.tsv) | [TSV](wasm/base/083.tsv) |
| 84 | caret-same-anchor-terminal-false-note-first-false-left-hwp | 1 | [100.00000%](native/candidate/084.tsv) | [TSV](native/base/084.tsv) | [100.00000%](wasm/candidate/084.tsv) | [TSV](wasm/base/084.tsv) |
| 85 | caret-same-anchor-terminal-false-note-first-false-left-hwpx | 1 | [100.00000%](native/candidate/085.tsv) | [TSV](native/base/085.tsv) | [100.00000%](wasm/candidate/085.tsv) | [TSV](wasm/base/085.tsv) |
| 86 | caret-same-anchor-terminal-false-note-first-false-right-hwp | 1 | [100.00000%](native/candidate/086.tsv) | [TSV](native/base/086.tsv) | [100.00000%](wasm/candidate/086.tsv) | [TSV](wasm/base/086.tsv) |
| 87 | caret-same-anchor-terminal-false-note-first-false-right-hwpx | 1 | [100.00000%](native/candidate/087.tsv) | [TSV](native/base/087.tsv) | [100.00000%](wasm/candidate/087.tsv) | [TSV](wasm/base/087.tsv) |
| 88 | caret-same-anchor-terminal-false-note-first-true-left-hwp | 1 | [100.00000%](native/candidate/088.tsv) | [TSV](native/base/088.tsv) | [100.00000%](wasm/candidate/088.tsv) | [TSV](wasm/base/088.tsv) |
| 89 | caret-same-anchor-terminal-false-note-first-true-left-hwpx | 1 | [100.00000%](native/candidate/089.tsv) | [TSV](native/base/089.tsv) | [100.00000%](wasm/candidate/089.tsv) | [TSV](wasm/base/089.tsv) |
| 90 | caret-same-anchor-terminal-false-note-first-true-right-hwp | 1 | [100.00000%](native/candidate/090.tsv) | [TSV](native/base/090.tsv) | [100.00000%](wasm/candidate/090.tsv) | [TSV](wasm/base/090.tsv) |
| 91 | caret-same-anchor-terminal-false-note-first-true-right-hwpx | 1 | [100.00000%](native/candidate/091.tsv) | [TSV](native/base/091.tsv) | [100.00000%](wasm/candidate/091.tsv) | [TSV](wasm/base/091.tsv) |
| 92 | caret-same-anchor-terminal-true-note-first-false-left-hwp | 1 | [100.00000%](native/candidate/092.tsv) | [TSV](native/base/092.tsv) | [100.00000%](wasm/candidate/092.tsv) | [TSV](wasm/base/092.tsv) |
| 93 | caret-same-anchor-terminal-true-note-first-false-left-hwpx | 1 | [100.00000%](native/candidate/093.tsv) | [TSV](native/base/093.tsv) | [100.00000%](wasm/candidate/093.tsv) | [TSV](wasm/base/093.tsv) |
| 94 | caret-same-anchor-terminal-true-note-first-false-right-hwp | 1 | [100.00000%](native/candidate/094.tsv) | [TSV](native/base/094.tsv) | [100.00000%](wasm/candidate/094.tsv) | [TSV](wasm/base/094.tsv) |
| 95 | caret-same-anchor-terminal-true-note-first-false-right-hwpx | 1 | [100.00000%](native/candidate/095.tsv) | [TSV](native/base/095.tsv) | [100.00000%](wasm/candidate/095.tsv) | [TSV](wasm/base/095.tsv) |
| 96 | caret-same-anchor-terminal-true-note-first-true-left-hwp | 1 | [100.00000%](native/candidate/096.tsv) | [TSV](native/base/096.tsv) | [100.00000%](wasm/candidate/096.tsv) | [TSV](wasm/base/096.tsv) |
| 97 | caret-same-anchor-terminal-true-note-first-true-left-hwpx | 1 | [100.00000%](native/candidate/097.tsv) | [TSV](native/base/097.tsv) | [100.00000%](wasm/candidate/097.tsv) | [TSV](wasm/base/097.tsv) |
| 98 | caret-same-anchor-terminal-true-note-first-true-right-hwp | 1 | [100.00000%](native/candidate/098.tsv) | [TSV](native/base/098.tsv) | [100.00000%](wasm/candidate/098.tsv) | [TSV](wasm/base/098.tsv) |
| 99 | caret-same-anchor-terminal-true-note-first-true-right-hwpx | 1 | [100.00000%](native/candidate/099.tsv) | [TSV](native/base/099.tsv) | [100.00000%](wasm/candidate/099.tsv) | [TSV](wasm/base/099.tsv) |
| 100 | caret-single-payload-hwp | 1 | [100.00000%](native/candidate/100.tsv) | [TSV](native/base/100.tsv) | [100.00000%](wasm/candidate/100.tsv) | [TSV](wasm/base/100.tsv) |
| 101 | caret-single-payload-hwpx | 1 | [100.00000%](native/candidate/101.tsv) | [TSV](native/base/101.tsv) | [100.00000%](wasm/candidate/101.tsv) | [TSV](wasm/base/101.tsv) |
| 102 | overlap-size-neg3 | 1 | [100.00000%](native/candidate/102.tsv) | [TSV](native/base/102.tsv) | [100.00000%](wasm/candidate/102.tsv) | [TSV](wasm/base/102.tsv) |
| 103 | overlap-size-0 | 1 | [100.00000%](native/candidate/103.tsv) | [TSV](native/base/103.tsv) | [100.00000%](wasm/candidate/103.tsv) | [TSV](wasm/base/103.tsv) |
| 104 | overlap-size-10 | 1 | [100.00000%](native/candidate/104.tsv) | [TSV](native/base/104.tsv) | [100.00000%](wasm/candidate/104.tsv) | [TSV](wasm/base/104.tsv) |
| 105 | overlap-size-20 | 1 | [100.00000%](native/candidate/105.tsv) | [TSV](native/base/105.tsv) | [100.00000%](wasm/candidate/105.tsv) | [TSV](wasm/base/105.tsv) |
| 106 | overlap-size-50 | 1 | [95.73216%](native/candidate/106.tsv) | [TSV](native/base/106.tsv) | [95.27616%](wasm/candidate/106.tsv) | [TSV](wasm/base/106.tsv) |
| 107 | point-gulim | 1 | [100.00000%](native/candidate/107.tsv) | [TSV](native/base/107.tsv) | [100.00000%](wasm/candidate/107.tsv) | [TSV](wasm/base/107.tsv) |
| 108 | point-arial | 1 | [100.00000%](native/candidate/108.tsv) | [TSV](native/base/108.tsv) | [100.00000%](wasm/candidate/108.tsv) | [TSV](wasm/base/108.tsv) |
| 109 | point-hcr-batang | 1 | [100.00000%](native/candidate/109.tsv) | [TSV](native/base/109.tsv) | [100.00000%](wasm/candidate/109.tsv) | [TSV](wasm/base/109.tsv) |
| 110 | pr8093-form-slot-hwp | 1 | [100.00000%](native/candidate/110.tsv) | [TSV](native/base/110.tsv) | [100.00000%](wasm/candidate/110.tsv) | [TSV](wasm/base/110.tsv) |
| 111 | pr8093-form-slot-hwpx | 1 | [100.00000%](native/candidate/111.tsv) | [TSV](native/base/111.tsv) | [100.00000%](wasm/candidate/111.tsv) | [TSV](wasm/base/111.tsv) |
| 112 | pr8045-hancom-saved-square | 1 | [100.00000%](native/candidate/112.tsv) | [TSV](native/base/112.tsv) | [100.00000%](wasm/candidate/112.tsv) | [TSV](wasm/base/112.tsv) |
| 113 | intrinsic_heavy_face_20261026 | 19 | [96.62490%](native/candidate/113.tsv) | [TSV](native/base/113.tsv) | [96.89977%](wasm/candidate/113.tsv) | [TSV](wasm/base/113.tsv) |
| 114 | legacy_chart_physical_layout_20261020/current-20261026 | 1 | [99.28860%](native/candidate/114.tsv) | [TSV](native/base/114.tsv) | [99.30234%](wasm/candidate/114.tsv) | [TSV](wasm/base/114.tsv) |
| 115 | pr8061-explicit-geometry | 1 | [100.00000%](native/candidate/115.tsv) | [TSV](native/base/115.tsv) | [100.00000%](wasm/candidate/115.tsv) | [TSV](wasm/base/115.tsv) |
| 116 | pr8061-no-paper | 1 | [100.00000%](native/candidate/116.tsv) | [TSV](native/base/116.tsv) | [100.00000%](wasm/candidate/116.tsv) | [TSV](wasm/base/116.tsv) |
| 117 | pr8061-picture-z2 | 1 | [100.00000%](native/candidate/117.tsv) | [TSV](native/base/117.tsv) | [100.00000%](wasm/candidate/117.tsv) | [TSV](wasm/base/117.tsv) |
| 118 | pr8061-picture-follow-cell | 1 | [100.00000%](native/candidate/118.tsv) | [TSV](native/base/118.tsv) | [100.00000%](wasm/candidate/118.tsv) | [TSV](wasm/base/118.tsv) |
| 119 | pr8061-floating-page-order | 1 | [100.00000%](native/candidate/119.tsv) | [TSV](native/base/119.tsv) | [100.00000%](wasm/candidate/119.tsv) | [TSV](wasm/base/119.tsv) |
| 120 | pr8061-floating-picture-z2 | 1 | [100.00000%](native/candidate/120.tsv) | [TSV](native/base/120.tsv) | [100.00000%](wasm/candidate/120.tsv) | [TSV](wasm/base/120.tsv) |
| 121 | pr8061-floating-follow-cell | 1 | [100.00000%](native/candidate/121.tsv) | [TSV](native/base/121.tsv) | [100.00000%](wasm/candidate/121.tsv) | [TSV](wasm/base/121.tsv) |
| 122 | pr8061-bordered-page-picture | 1 | [100.00000%](native/candidate/122.tsv) | [TSV](native/base/122.tsv) | [100.00000%](wasm/candidate/122.tsv) | [TSV](wasm/base/122.tsv) |
| 123 | pr8055-normal-rowbreak | 2 | [100.00000%](native/candidate/123.tsv) | [TSV](native/base/123.tsv) | [100.00000%](wasm/candidate/123.tsv) | [TSV](wasm/base/123.tsv) |
| 124 | pr7694-updated-198-chart-before | 1 | [99.80761%](native/candidate/124.tsv) | [TSV](native/base/124.tsv) | [99.66483%](wasm/candidate/124.tsv) | [TSV](wasm/base/124.tsv) |
| 125 | pr7694-updated-198-chart-after | 1 | [99.71479%](native/candidate/125.tsv) | [TSV](native/base/125.tsv) | [99.73289%](wasm/candidate/125.tsv) | [TSV](wasm/base/125.tsv) |
| 126 | form-only-defaults-hwp | 1 | [99.99194%](native/candidate/126.tsv) | [TSV](native/base/126.tsv) | [99.99194%](wasm/candidate/126.tsv) | [TSV](wasm/base/126.tsv) |
| 127 | form-only-defaults-hwpx | 1 | [99.99194%](native/candidate/127.tsv) | [TSV](native/base/127.tsv) | [99.99194%](wasm/candidate/127.tsv) | [TSV](wasm/base/127.tsv) |
| 128 | pr7694-chart-before-hwpx | 1 | [99.80761%](native/candidate/128.tsv) | [TSV](native/base/128.tsv) | [99.66483%](wasm/candidate/128.tsv) | [TSV](wasm/base/128.tsv) |
| 129 | pr7694-chart-after-hwpx | 1 | [99.71479%](native/candidate/129.tsv) | [TSV](native/base/129.tsv) | [99.73289%](wasm/candidate/129.tsv) | [TSV](wasm/base/129.tsv) |

[페이지별 base/후보 비교 원문](base_candidate_comparison.tsv)도 보존했다.

## 대표 이미지와 직접 판독

대표 review와 standalone overlay를 같이 연결한다. 전체 쪽의 overlay 합성 대신 전쪽 TSV와 변경·최저·구조 경계의 판독을 사용한다.

| 조건·경로 | review | overlay |
| --- | --- | --- |
| pr7694-updated-198-chart-before native p1 | [review](png/native/pr7694-updated-198-chart-before/review_001.png) | [overlay](png/native/pr7694-updated-198-chart-before/overlay_001.png) |
| pr7694-updated-198-chart-after native p1 | [review](png/native/pr7694-updated-198-chart-after/review_001.png) | [overlay](png/native/pr7694-updated-198-chart-after/overlay_001.png) |
| form-only-defaults-hwp native p1 | [review](png/native/form-only-defaults-hwp/review_001.png) | [overlay](png/native/form-only-defaults-hwp/overlay_001.png) |
| form-only-defaults-hwpx native p1 | [review](png/native/form-only-defaults-hwpx/review_001.png) | [overlay](png/native/form-only-defaults-hwpx/overlay_001.png) |
| pr7694-updated-198-chart-before wasm p1 | [review](png/wasm/pr7694-updated-198-chart-before/review_001.png) | [overlay](png/wasm/pr7694-updated-198-chart-before/overlay_001.png) |
| pr7694-updated-198-chart-after wasm p1 | [review](png/wasm/pr7694-updated-198-chart-after/review_001.png) | [overlay](png/wasm/pr7694-updated-198-chart-after/overlay_001.png) |
| form-only-defaults-hwp wasm p1 | [review](png/wasm/form-only-defaults-hwp/review_001.png) | [overlay](png/wasm/form-only-defaults-hwp/overlay_001.png) |
| form-only-defaults-hwpx wasm p1 | [review](png/wasm/form-only-defaults-hwpx/review_001.png) | [overlay](png/wasm/form-only-defaults-hwpx/overlay_001.png) |
| 4599 native p34 | [review](png/native/minimum/review/review_034.png) | [overlay](png/native/minimum/overlay/overlay_034.png) |
| 6970 wasm p3 | [review](png/wasm/minimum/review/review_003.png) | [overlay](png/wasm/minimum/overlay/overlay_003.png) |

차트의 막대·축·범례·차트 바깥 본문을 같은 좌표로 확인했다. 가는 테두리와 글자 래스터 농도 차이는 남으며 RGB 동일이라고 주장하지 않는다.
Form 기본값 표본의 검은 배경은 원본의 명시적 0 값과 독립 Print에도 있는 모양이다. 다섯 종류의 높이·순서·전후 본문 전진을 검증한 것이며 모든 Form UI 속성이나 보이지 않는 캡션까지 확인했다는 뜻은 아니다.
[셀 그림 8조건·RowBreak의 직접 판독](../cell_picture_layer_qualified_20261026/README.md)과 이전 원 PR별 경계 판독도 연결한다.
새 캡처와 이전 판독의 픽셀 동일성 및 HWP/HWPX 차트 판독 재사용 근거를 provenance에 별도로 기록했다. 새로 생성한 패널은 실제 개봉 여부와 구분한다.

## 제외와 실행 정정

일반 before/after HWPX의 기존 미달은 [#8266 증거](../issue_8266/README.md)이며 정상 시각 범위에 넣지 않았다. 정확한 모델·저장 검사는 유지한다.
손상 HWP 두 파일과 sample16·편람의 삭제를 유지했다. 삭제된 표본의 대체본·파생 참조를 만들지 않았다.
4183 실행의 OLE 두 경로는 로그 파일명의 `/` 처리 오류로 렌더 시작 전에 실패했다. 그 두 경로만 4202에서 실제 재실행해 통과했으며 원 실패 기록은 output에 보존했다.
TSV 성공은 개별 PR의 주장·반례 검토나 원격 CI·merge 완료를 뜻하지 않는다. 개별 리뷰 문서와 최종 제출 게이트를 별도로 확인한다.

최저 두 페이지의 정본 review·overlay도 실제로 열어 확인했다. 4599 p34의 차트·캡션·
표·후속 본문 배치는 유지되며 글자·괘선 농도 차이가 남는다. 6970 p3의 단 소유·
글줄 수·내용 순서·개체 위치는 보존되고 base에도 있던 glyph 전진·윤곽 잔차가 남는다.
이 잔차의 해결이나 해당 페이지의 px 고정 자격을 주장하지 않는다.

## 문서 전용 base 갱신 뒤 증거의 적용 범위

#8240의 `THIRD_PARTY.md` 고지 41줄을 반영해 `4e690c930105fa09fef484787ed0a2e5e182d444`
위로 리베이스했다. 이전 base와 새 base의 차이는 이 문서 하나이며, 후보의 src·crates·tests·
scripts·Studio·npm·Cargo 파일은 리베이스 전후 Git tree ID가 모두 같다. 따라서 측정한
base `238de6de41a208648f4ade6b494fe433a81c1fce`와 새 base의 실행 결과 대응을 유지한다.
이 문서 변경을 렌더링 재실행으로 표시하지 않았고, 새 base로 정책 검사와 삭제 보존을
다시 확인한다. 실제 생산 코드나 입력이 바뀌면 그 영향을 받는 검증은 다시 수행한다.

리베이스로 replay한 356개 commit은 `git range-diff`에서 전부 같은 patch로 확인했다.
[원 PR별 현재 체리픽 대응](source_commit_map.json)과
[리베이스 전후 tree ID](rebase_equivalence.json)를 기록했다. 실제 실행 당시의 옛
producer SHA를 새로 실행한 SHA처럼 바꾸지 않고, 같은 코드라는 근거를 분리했다.
