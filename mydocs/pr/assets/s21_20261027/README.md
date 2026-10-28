# 통합 후보의 검증 입력·전체 TSV·대표 증거

> **2026-10-27 증적 대응 정정:** 원 PR에 검증용 HWP/HWPX가 없는 사례는 유사 문서로 대체했음을 표시한다. 아래 수치는 실제 측정 입력에 한정하며 배경 실물 문서의 직접 검증으로 표현하지 않는다. [현재 대응 원장](../pr8319_evidence_alignment_20261027/alignment.json)과 개별 review를 우선한다.

코드·테스트 source는 `04c92da78165ccbf63b467855591df183655a98d`, 고정 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다.

[45개 PR의 주장 → 실제 입력·편집 상태·쪽](claim-evidence.json), [개별 리뷰](reviews.md), [실행 요약](execution-evidence.json), [계보](source-lineage.json), [독립 Print](api-independent-print.json), [실제 편집 세션](editing-session-evidence.json)을 함께 확인한다.

변경 전(base)의 90% 미만 값은 비교 자료로 허용하며, 변경 후 후보는 모든 관련 페이지가 Native/fresh WASM 각각 90% 이상이어야 한다. 아래 TSV는 원문 바이트를 복사했으며 수치·반올림을 바꾸지 않았다. 관계없는 문서와 빈 페이지 PNG로 구현 주장을 대신하지 않는다.

| 경로 | 조건 | 전체 쪽 | 최저 raw | 90% 미만·누락 | base 대비 0.1%p 초과 하락 |
| --- | ---: | ---: | ---: | ---: | ---: |
| native | 197 | 256 | 93.22230% | 0 | 0 |
| wasm | 197 | 256 | 93.52322% | 0 | 0 |

raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 같은 96DPI·실제 시스템 글꼴 공급·Print 프로필을 사용했다. Native는 Skia PNG, fresh WASM은 표준 release와 실제 wasm-opt를 완료한 current-092 Canvas2D PNG다. `compat 2022`는 렌더 호환 설정이며 PDF 제품 연도가 아니다. 모든 페이지를 98% 이상이라고 주장하지 않는다.

#8188의 지정 네 입력은 별도 전체 512쪽에서 raw100과 candidate/reference 정수 미일치0/0이다. [전쪽 정수 근거](visual-evidence.json)를 따른다. sample16·편람의 일반 입력·회귀 제외는 유지한다.

## 입력·Print 및 전체 페이지 TSV

[기존 입력·SHA](main-condition-index.tsv) / [공개 API 편집 입력·SHA](api-condition-index.tsv) / [파일 무결성](file-integrity.tsv). 기존 추적 파일은 중복 복사하지 않고 색인의 경로를 재사용한다. 새 편집 출력 209~214만 같은 원본을 MCP 2020 Print로 다시 출력했다.

| 조건 | Native base / 후보 | fresh WASM base / 후보 |
| --- | --- | --- |
| 015 | [base](n/b/015.tsv) / [후보](n/c/015.tsv) | [base](w/b/015.tsv) / [후보](w/c/015.tsv) |
| 018 | [base](n/b/018.tsv) / [후보](n/c/018.tsv) | [base](w/b/018.tsv) / [후보](w/c/018.tsv) |
| 019 | [base](n/b/019.tsv) / [후보](n/c/019.tsv) | [base](w/b/019.tsv) / [후보](w/c/019.tsv) |
| 020 | [base](n/b/020.tsv) / [후보](n/c/020.tsv) | [base](w/b/020.tsv) / [후보](w/c/020.tsv) |
| 021 | [base](n/b/021.tsv) / [후보](n/c/021.tsv) | [base](w/b/021.tsv) / [후보](w/c/021.tsv) |
| 022 | [base](n/b/022.tsv) / [후보](n/c/022.tsv) | [base](w/b/022.tsv) / [후보](w/c/022.tsv) |
| 023 | [base](n/b/023.tsv) / [후보](n/c/023.tsv) | [base](w/b/023.tsv) / [후보](w/c/023.tsv) |
| 024 | [base](n/b/024.tsv) / [후보](n/c/024.tsv) | [base](w/b/024.tsv) / [후보](w/c/024.tsv) |
| 025 | [base](n/b/025.tsv) / [후보](n/c/025.tsv) | [base](w/b/025.tsv) / [후보](w/c/025.tsv) |
| 026 | [base](n/b/026.tsv) / [후보](n/c/026.tsv) | [base](w/b/026.tsv) / [후보](w/c/026.tsv) |
| 027 | [base](n/b/027.tsv) / [후보](n/c/027.tsv) | [base](w/b/027.tsv) / [후보](w/c/027.tsv) |
| 028 | [base](n/b/028.tsv) / [후보](n/c/028.tsv) | [base](w/b/028.tsv) / [후보](w/c/028.tsv) |
| 029 | [base](n/b/029.tsv) / [후보](n/c/029.tsv) | [base](w/b/029.tsv) / [후보](w/c/029.tsv) |
| 030 | [base](n/b/030.tsv) / [후보](n/c/030.tsv) | [base](w/b/030.tsv) / [후보](w/c/030.tsv) |
| 031 | [base](n/b/031.tsv) / [후보](n/c/031.tsv) | [base](w/b/031.tsv) / [후보](w/c/031.tsv) |
| 032 | [base](n/b/032.tsv) / [후보](n/c/032.tsv) | [base](w/b/032.tsv) / [후보](w/c/032.tsv) |
| 033 | [base](n/b/033.tsv) / [후보](n/c/033.tsv) | [base](w/b/033.tsv) / [후보](w/c/033.tsv) |
| 034 | [base](n/b/034.tsv) / [후보](n/c/034.tsv) | [base](w/b/034.tsv) / [후보](w/c/034.tsv) |
| 035 | [base](n/b/035.tsv) / [후보](n/c/035.tsv) | [base](w/b/035.tsv) / [후보](w/c/035.tsv) |
| 036 | [base](n/b/036.tsv) / [후보](n/c/036.tsv) | [base](w/b/036.tsv) / [후보](w/c/036.tsv) |
| 037 | [base](n/b/037.tsv) / [후보](n/c/037.tsv) | [base](w/b/037.tsv) / [후보](w/c/037.tsv) |
| 038 | [base](n/b/038.tsv) / [후보](n/c/038.tsv) | [base](w/b/038.tsv) / [후보](w/c/038.tsv) |
| 039 | [base](n/b/039.tsv) / [후보](n/c/039.tsv) | [base](w/b/039.tsv) / [후보](w/c/039.tsv) |
| 040 | [base](n/b/040.tsv) / [후보](n/c/040.tsv) | [base](w/b/040.tsv) / [후보](w/c/040.tsv) |
| 041 | [base](n/b/041.tsv) / [후보](n/c/041.tsv) | [base](w/b/041.tsv) / [후보](w/c/041.tsv) |
| 042 | [base](n/b/042.tsv) / [후보](n/c/042.tsv) | [base](w/b/042.tsv) / [후보](w/c/042.tsv) |
| 043 | [base](n/b/043.tsv) / [후보](n/c/043.tsv) | [base](w/b/043.tsv) / [후보](w/c/043.tsv) |
| 044 | [base](n/b/044.tsv) / [후보](n/c/044.tsv) | [base](w/b/044.tsv) / [후보](w/c/044.tsv) |
| 045 | [base](n/b/045.tsv) / [후보](n/c/045.tsv) | [base](w/b/045.tsv) / [후보](w/c/045.tsv) |
| 046 | [base](n/b/046.tsv) / [후보](n/c/046.tsv) | [base](w/b/046.tsv) / [후보](w/c/046.tsv) |
| 047 | [base](n/b/047.tsv) / [후보](n/c/047.tsv) | [base](w/b/047.tsv) / [후보](w/c/047.tsv) |
| 048 | [base](n/b/048.tsv) / [후보](n/c/048.tsv) | [base](w/b/048.tsv) / [후보](w/c/048.tsv) |
| 049 | [base](n/b/049.tsv) / [후보](n/c/049.tsv) | [base](w/b/049.tsv) / [후보](w/c/049.tsv) |
| 050 | [base](n/b/050.tsv) / [후보](n/c/050.tsv) | [base](w/b/050.tsv) / [후보](w/c/050.tsv) |
| 051 | [base](n/b/051.tsv) / [후보](n/c/051.tsv) | [base](w/b/051.tsv) / [후보](w/c/051.tsv) |
| 052 | [base](n/b/052.tsv) / [후보](n/c/052.tsv) | [base](w/b/052.tsv) / [후보](w/c/052.tsv) |
| 053 | [base](n/b/053.tsv) / [후보](n/c/053.tsv) | [base](w/b/053.tsv) / [후보](w/c/053.tsv) |
| 054 | [base](n/b/054.tsv) / [후보](n/c/054.tsv) | [base](w/b/054.tsv) / [후보](w/c/054.tsv) |
| 055 | [base](n/b/055.tsv) / [후보](n/c/055.tsv) | [base](w/b/055.tsv) / [후보](w/c/055.tsv) |
| 056 | [base](n/b/056.tsv) / [후보](n/c/056.tsv) | [base](w/b/056.tsv) / [후보](w/c/056.tsv) |
| 057 | [base](n/b/057.tsv) / [후보](n/c/057.tsv) | [base](w/b/057.tsv) / [후보](w/c/057.tsv) |
| 058 | [base](n/b/058.tsv) / [후보](n/c/058.tsv) | [base](w/b/058.tsv) / [후보](w/c/058.tsv) |
| 059 | [base](n/b/059.tsv) / [후보](n/c/059.tsv) | [base](w/b/059.tsv) / [후보](w/c/059.tsv) |
| 060 | [base](n/b/060.tsv) / [후보](n/c/060.tsv) | [base](w/b/060.tsv) / [후보](w/c/060.tsv) |
| 061 | [base](n/b/061.tsv) / [후보](n/c/061.tsv) | [base](w/b/061.tsv) / [후보](w/c/061.tsv) |
| 062 | [base](n/b/062.tsv) / [후보](n/c/062.tsv) | [base](w/b/062.tsv) / [후보](w/c/062.tsv) |
| 063 | [base](n/b/063.tsv) / [후보](n/c/063.tsv) | [base](w/b/063.tsv) / [후보](w/c/063.tsv) |
| 064 | [base](n/b/064.tsv) / [후보](n/c/064.tsv) | [base](w/b/064.tsv) / [후보](w/c/064.tsv) |
| 065 | [base](n/b/065.tsv) / [후보](n/c/065.tsv) | [base](w/b/065.tsv) / [후보](w/c/065.tsv) |
| 066 | [base](n/b/066.tsv) / [후보](n/c/066.tsv) | [base](w/b/066.tsv) / [후보](w/c/066.tsv) |
| 067 | [base](n/b/067.tsv) / [후보](n/c/067.tsv) | [base](w/b/067.tsv) / [후보](w/c/067.tsv) |
| 068 | [base](n/b/068.tsv) / [후보](n/c/068.tsv) | [base](w/b/068.tsv) / [후보](w/c/068.tsv) |
| 069 | [base](n/b/069.tsv) / [후보](n/c/069.tsv) | [base](w/b/069.tsv) / [후보](w/c/069.tsv) |
| 070 | [base](n/b/070.tsv) / [후보](n/c/070.tsv) | [base](w/b/070.tsv) / [후보](w/c/070.tsv) |
| 071 | [base](n/b/071.tsv) / [후보](n/c/071.tsv) | [base](w/b/071.tsv) / [후보](w/c/071.tsv) |
| 072 | [base](n/b/072.tsv) / [후보](n/c/072.tsv) | [base](w/b/072.tsv) / [후보](w/c/072.tsv) |
| 073 | [base](n/b/073.tsv) / [후보](n/c/073.tsv) | [base](w/b/073.tsv) / [후보](w/c/073.tsv) |
| 074 | [base](n/b/074.tsv) / [후보](n/c/074.tsv) | [base](w/b/074.tsv) / [후보](w/c/074.tsv) |
| 075 | [base](n/b/075.tsv) / [후보](n/c/075.tsv) | [base](w/b/075.tsv) / [후보](w/c/075.tsv) |
| 076 | [base](n/b/076.tsv) / [후보](n/c/076.tsv) | [base](w/b/076.tsv) / [후보](w/c/076.tsv) |
| 077 | [base](n/b/077.tsv) / [후보](n/c/077.tsv) | [base](w/b/077.tsv) / [후보](w/c/077.tsv) |
| 078 | [base](n/b/078.tsv) / [후보](n/c/078.tsv) | [base](w/b/078.tsv) / [후보](w/c/078.tsv) |
| 079 | [base](n/b/079.tsv) / [후보](n/c/079.tsv) | [base](w/b/079.tsv) / [후보](w/c/079.tsv) |
| 080 | [base](n/b/080.tsv) / [후보](n/c/080.tsv) | [base](w/b/080.tsv) / [후보](w/c/080.tsv) |
| 081 | [base](n/b/081.tsv) / [후보](n/c/081.tsv) | [base](w/b/081.tsv) / [후보](w/c/081.tsv) |
| 082 | [base](n/b/082.tsv) / [후보](n/c/082.tsv) | [base](w/b/082.tsv) / [후보](w/c/082.tsv) |
| 083 | [base](n/b/083.tsv) / [후보](n/c/083.tsv) | [base](w/b/083.tsv) / [후보](w/c/083.tsv) |
| 084 | [base](n/b/084.tsv) / [후보](n/c/084.tsv) | [base](w/b/084.tsv) / [후보](w/c/084.tsv) |
| 085 | [base](n/b/085.tsv) / [후보](n/c/085.tsv) | [base](w/b/085.tsv) / [후보](w/c/085.tsv) |
| 086 | [base](n/b/086.tsv) / [후보](n/c/086.tsv) | [base](w/b/086.tsv) / [후보](w/c/086.tsv) |
| 087 | [base](n/b/087.tsv) / [후보](n/c/087.tsv) | [base](w/b/087.tsv) / [후보](w/c/087.tsv) |
| 088 | [base](n/b/088.tsv) / [후보](n/c/088.tsv) | [base](w/b/088.tsv) / [후보](w/c/088.tsv) |
| 089 | [base](n/b/089.tsv) / [후보](n/c/089.tsv) | [base](w/b/089.tsv) / [후보](w/c/089.tsv) |
| 090 | [base](n/b/090.tsv) / [후보](n/c/090.tsv) | [base](w/b/090.tsv) / [후보](w/c/090.tsv) |
| 091 | [base](n/b/091.tsv) / [후보](n/c/091.tsv) | [base](w/b/091.tsv) / [후보](w/c/091.tsv) |
| 092 | [base](n/b/092.tsv) / [후보](n/c/092.tsv) | [base](w/b/092.tsv) / [후보](w/c/092.tsv) |
| 093 | [base](n/b/093.tsv) / [후보](n/c/093.tsv) | [base](w/b/093.tsv) / [후보](w/c/093.tsv) |
| 094 | [base](n/b/094.tsv) / [후보](n/c/094.tsv) | [base](w/b/094.tsv) / [후보](w/c/094.tsv) |
| 095 | [base](n/b/095.tsv) / [후보](n/c/095.tsv) | [base](w/b/095.tsv) / [후보](w/c/095.tsv) |
| 096 | [base](n/b/096.tsv) / [후보](n/c/096.tsv) | [base](w/b/096.tsv) / [후보](w/c/096.tsv) |
| 097 | [base](n/b/097.tsv) / [후보](n/c/097.tsv) | [base](w/b/097.tsv) / [후보](w/c/097.tsv) |
| 098 | [base](n/b/098.tsv) / [후보](n/c/098.tsv) | [base](w/b/098.tsv) / [후보](w/c/098.tsv) |
| 099 | [base](n/b/099.tsv) / [후보](n/c/099.tsv) | [base](w/b/099.tsv) / [후보](w/c/099.tsv) |
| 100 | [base](n/b/100.tsv) / [후보](n/c/100.tsv) | [base](w/b/100.tsv) / [후보](w/c/100.tsv) |
| 101 | [base](n/b/101.tsv) / [후보](n/c/101.tsv) | [base](w/b/101.tsv) / [후보](w/c/101.tsv) |
| 102 | [base](n/b/102.tsv) / [후보](n/c/102.tsv) | [base](w/b/102.tsv) / [후보](w/c/102.tsv) |
| 103 | [base](n/b/103.tsv) / [후보](n/c/103.tsv) | [base](w/b/103.tsv) / [후보](w/c/103.tsv) |
| 104 | [base](n/b/104.tsv) / [후보](n/c/104.tsv) | [base](w/b/104.tsv) / [후보](w/c/104.tsv) |
| 105 | [base](n/b/105.tsv) / [후보](n/c/105.tsv) | [base](w/b/105.tsv) / [후보](w/c/105.tsv) |
| 106 | [base](n/b/106.tsv) / [후보](n/c/106.tsv) | [base](w/b/106.tsv) / [후보](w/c/106.tsv) |
| 107 | [base](n/b/107.tsv) / [후보](n/c/107.tsv) | [base](w/b/107.tsv) / [후보](w/c/107.tsv) |
| 108 | [base](n/b/108.tsv) / [후보](n/c/108.tsv) | [base](w/b/108.tsv) / [후보](w/c/108.tsv) |
| 109 | [base](n/b/109.tsv) / [후보](n/c/109.tsv) | [base](w/b/109.tsv) / [후보](w/c/109.tsv) |
| 110 | [base](n/b/110.tsv) / [후보](n/c/110.tsv) | [base](w/b/110.tsv) / [후보](w/c/110.tsv) |
| 111 | [base](n/b/111.tsv) / [후보](n/c/111.tsv) | [base](w/b/111.tsv) / [후보](w/c/111.tsv) |
| 112 | [base](n/b/112.tsv) / [후보](n/c/112.tsv) | [base](w/b/112.tsv) / [후보](w/c/112.tsv) |
| 113 | [base](n/b/113.tsv) / [후보](n/c/113.tsv) | [base](w/b/113.tsv) / [후보](w/c/113.tsv) |
| 115 | [base](n/b/115.tsv) / [후보](n/c/115.tsv) | [base](w/b/115.tsv) / [후보](w/c/115.tsv) |
| 116 | [base](n/b/116.tsv) / [후보](n/c/116.tsv) | [base](w/b/116.tsv) / [후보](w/c/116.tsv) |
| 117 | [base](n/b/117.tsv) / [후보](n/c/117.tsv) | [base](w/b/117.tsv) / [후보](w/c/117.tsv) |
| 118 | [base](n/b/118.tsv) / [후보](n/c/118.tsv) | [base](w/b/118.tsv) / [후보](w/c/118.tsv) |
| 119 | [base](n/b/119.tsv) / [후보](n/c/119.tsv) | [base](w/b/119.tsv) / [후보](w/c/119.tsv) |
| 120 | [base](n/b/120.tsv) / [후보](n/c/120.tsv) | [base](w/b/120.tsv) / [후보](w/c/120.tsv) |
| 121 | [base](n/b/121.tsv) / [후보](n/c/121.tsv) | [base](w/b/121.tsv) / [후보](w/c/121.tsv) |
| 122 | [base](n/b/122.tsv) / [후보](n/c/122.tsv) | [base](w/b/122.tsv) / [후보](w/c/122.tsv) |
| 123 | [base](n/b/123.tsv) / [후보](n/c/123.tsv) | [base](w/b/123.tsv) / [후보](w/c/123.tsv) |
| 124 | [base](n/b/124.tsv) / [후보](n/c/124.tsv) | [base](w/b/124.tsv) / [후보](w/c/124.tsv) |
| 125 | [base](n/b/125.tsv) / [후보](n/c/125.tsv) | [base](w/b/125.tsv) / [후보](w/c/125.tsv) |
| 126 | [base](n/b/126.tsv) / [후보](n/c/126.tsv) | [base](w/b/126.tsv) / [후보](w/c/126.tsv) |
| 127 | [base](n/b/127.tsv) / [후보](n/c/127.tsv) | [base](w/b/127.tsv) / [후보](w/c/127.tsv) |
| 128 | [base](n/b/128.tsv) / [후보](n/c/128.tsv) | [base](w/b/128.tsv) / [후보](w/c/128.tsv) |
| 129 | [base](n/b/129.tsv) / [후보](n/c/129.tsv) | [base](w/b/129.tsv) / [후보](w/c/129.tsv) |
| 130 | [base](n/b/130.tsv) / [후보](n/c/130.tsv) | [base](w/b/130.tsv) / [후보](w/c/130.tsv) |
| 131 | [base](n/b/131.tsv) / [후보](n/c/131.tsv) | [base](w/b/131.tsv) / [후보](w/c/131.tsv) |
| 132 | [base](n/b/132.tsv) / [후보](n/c/132.tsv) | [base](w/b/132.tsv) / [후보](w/c/132.tsv) |
| 133 | [base](n/b/133.tsv) / [후보](n/c/133.tsv) | [base](w/b/133.tsv) / [후보](w/c/133.tsv) |
| 134 | [base](n/b/134.tsv) / [후보](n/c/134.tsv) | [base](w/b/134.tsv) / [후보](w/c/134.tsv) |
| 135 | [base](n/b/135.tsv) / [후보](n/c/135.tsv) | [base](w/b/135.tsv) / [후보](w/c/135.tsv) |
| 136 | [base](n/b/136.tsv) / [후보](n/c/136.tsv) | [base](w/b/136.tsv) / [후보](w/c/136.tsv) |
| 137 | [base](n/b/137.tsv) / [후보](n/c/137.tsv) | [base](w/b/137.tsv) / [후보](w/c/137.tsv) |
| 138 | [base](n/b/138.tsv) / [후보](n/c/138.tsv) | [base](w/b/138.tsv) / [후보](w/c/138.tsv) |
| 139 | [base](n/b/139.tsv) / [후보](n/c/139.tsv) | [base](w/b/139.tsv) / [후보](w/c/139.tsv) |
| 140 | [base](n/b/140.tsv) / [후보](n/c/140.tsv) | [base](w/b/140.tsv) / [후보](w/c/140.tsv) |
| 141 | [base](n/b/141.tsv) / [후보](n/c/141.tsv) | [base](w/b/141.tsv) / [후보](w/c/141.tsv) |
| 142 | [base](n/b/142.tsv) / [후보](n/c/142.tsv) | [base](w/b/142.tsv) / [후보](w/c/142.tsv) |
| 143 | [base](n/b/143.tsv) / [후보](n/c/143.tsv) | [base](w/b/143.tsv) / [후보](w/c/143.tsv) |
| 144 | [base](n/b/144.tsv) / [후보](n/c/144.tsv) | [base](w/b/144.tsv) / [후보](w/c/144.tsv) |
| 145 | [base](n/b/145.tsv) / [후보](n/c/145.tsv) | [base](w/b/145.tsv) / [후보](w/c/145.tsv) |
| 146 | [base](n/b/146.tsv) / [후보](n/c/146.tsv) | [base](w/b/146.tsv) / [후보](w/c/146.tsv) |
| 147 | [base](n/b/147.tsv) / [후보](n/c/147.tsv) | [base](w/b/147.tsv) / [후보](w/c/147.tsv) |
| 148 | [base](n/b/148.tsv) / [후보](n/c/148.tsv) | [base](w/b/148.tsv) / [후보](w/c/148.tsv) |
| 149 | [base](n/b/149.tsv) / [후보](n/c/149.tsv) | [base](w/b/149.tsv) / [후보](w/c/149.tsv) |
| 150 | [base](n/b/150.tsv) / [후보](n/c/150.tsv) | [base](w/b/150.tsv) / [후보](w/c/150.tsv) |
| 151 | [base](n/b/151.tsv) / [후보](n/c/151.tsv) | [base](w/b/151.tsv) / [후보](w/c/151.tsv) |
| 152 | [base](n/b/152.tsv) / [후보](n/c/152.tsv) | [base](w/b/152.tsv) / [후보](w/c/152.tsv) |
| 153 | [base](n/b/153.tsv) / [후보](n/c/153.tsv) | [base](w/b/153.tsv) / [후보](w/c/153.tsv) |
| 154 | [base](n/b/154.tsv) / [후보](n/c/154.tsv) | [base](w/b/154.tsv) / [후보](w/c/154.tsv) |
| 155 | [base](n/b/155.tsv) / [후보](n/c/155.tsv) | [base](w/b/155.tsv) / [후보](w/c/155.tsv) |
| 156 | [base](n/b/156.tsv) / [후보](n/c/156.tsv) | [base](w/b/156.tsv) / [후보](w/c/156.tsv) |
| 157 | [base](n/b/157.tsv) / [후보](n/c/157.tsv) | [base](w/b/157.tsv) / [후보](w/c/157.tsv) |
| 158 | [base](n/b/158.tsv) / [후보](n/c/158.tsv) | [base](w/b/158.tsv) / [후보](w/c/158.tsv) |
| 159 | [base](n/b/159.tsv) / [후보](n/c/159.tsv) | [base](w/b/159.tsv) / [후보](w/c/159.tsv) |
| 160 | [base](n/b/160.tsv) / [후보](n/c/160.tsv) | [base](w/b/160.tsv) / [후보](w/c/160.tsv) |
| 161 | [base](n/b/161.tsv) / [후보](n/c/161.tsv) | [base](w/b/161.tsv) / [후보](w/c/161.tsv) |
| 162 | [base](n/b/162.tsv) / [후보](n/c/162.tsv) | [base](w/b/162.tsv) / [후보](w/c/162.tsv) |
| 163 | [base](n/b/163.tsv) / [후보](n/c/163.tsv) | [base](w/b/163.tsv) / [후보](w/c/163.tsv) |
| 164 | [base](n/b/164.tsv) / [후보](n/c/164.tsv) | [base](w/b/164.tsv) / [후보](w/c/164.tsv) |
| 165 | [base](n/b/165.tsv) / [후보](n/c/165.tsv) | [base](w/b/165.tsv) / [후보](w/c/165.tsv) |
| 166 | [base](n/b/166.tsv) / [후보](n/c/166.tsv) | [base](w/b/166.tsv) / [후보](w/c/166.tsv) |
| 167 | [base](n/b/167.tsv) / [후보](n/c/167.tsv) | [base](w/b/167.tsv) / [후보](w/c/167.tsv) |
| 168 | [base](n/b/168.tsv) / [후보](n/c/168.tsv) | [base](w/b/168.tsv) / [후보](w/c/168.tsv) |
| 169 | [base](n/b/169.tsv) / [후보](n/c/169.tsv) | [base](w/b/169.tsv) / [후보](w/c/169.tsv) |
| 170 | [base](n/b/170.tsv) / [후보](n/c/170.tsv) | [base](w/b/170.tsv) / [후보](w/c/170.tsv) |
| 171 | [base](n/b/171.tsv) / [후보](n/c/171.tsv) | [base](w/b/171.tsv) / [후보](w/c/171.tsv) |
| 172 | [base](n/b/172.tsv) / [후보](n/c/172.tsv) | [base](w/b/172.tsv) / [후보](w/c/172.tsv) |
| 173 | [base](n/b/173.tsv) / [후보](n/c/173.tsv) | [base](w/b/173.tsv) / [후보](w/c/173.tsv) |
| 174 | [base](n/b/174.tsv) / [후보](n/c/174.tsv) | [base](w/b/174.tsv) / [후보](w/c/174.tsv) |
| 175 | [base](n/b/175.tsv) / [후보](n/c/175.tsv) | [base](w/b/175.tsv) / [후보](w/c/175.tsv) |
| 176 | [base](n/b/176.tsv) / [후보](n/c/176.tsv) | [base](w/b/176.tsv) / [후보](w/c/176.tsv) |
| 177 | [base](n/b/177.tsv) / [후보](n/c/177.tsv) | [base](w/b/177.tsv) / [후보](w/c/177.tsv) |
| 178 | [base](n/b/178.tsv) / [후보](n/c/178.tsv) | [base](w/b/178.tsv) / [후보](w/c/178.tsv) |
| 179 | [base](n/b/179.tsv) / [후보](n/c/179.tsv) | [base](w/b/179.tsv) / [후보](w/c/179.tsv) |
| 180 | [base](n/b/180.tsv) / [후보](n/c/180.tsv) | [base](w/b/180.tsv) / [후보](w/c/180.tsv) |
| 181 | [base](n/b/181.tsv) / [후보](n/c/181.tsv) | [base](w/b/181.tsv) / [후보](w/c/181.tsv) |
| 182 | [base](n/b/182.tsv) / [후보](n/c/182.tsv) | [base](w/b/182.tsv) / [후보](w/c/182.tsv) |
| 183 | [base](n/b/183.tsv) / [후보](n/c/183.tsv) | [base](w/b/183.tsv) / [후보](w/c/183.tsv) |
| 184 | [base](n/b/184.tsv) / [후보](n/c/184.tsv) | [base](w/b/184.tsv) / [후보](w/c/184.tsv) |
| 185 | [base](n/b/185.tsv) / [후보](n/c/185.tsv) | [base](w/b/185.tsv) / [후보](w/c/185.tsv) |
| 186 | [base](n/b/186.tsv) / [후보](n/c/186.tsv) | [base](w/b/186.tsv) / [후보](w/c/186.tsv) |
| 187 | [base](n/b/187.tsv) / [후보](n/c/187.tsv) | [base](w/b/187.tsv) / [후보](w/c/187.tsv) |
| 188 | [base](n/b/188.tsv) / [후보](n/c/188.tsv) | [base](w/b/188.tsv) / [후보](w/c/188.tsv) |
| 189 | [base](n/b/189.tsv) / [후보](n/c/189.tsv) | [base](w/b/189.tsv) / [후보](w/c/189.tsv) |
| 190 | [base](n/b/190.tsv) / [후보](n/c/190.tsv) | [base](w/b/190.tsv) / [후보](w/c/190.tsv) |
| 191 | [base](n/b/191.tsv) / [후보](n/c/191.tsv) | [base](w/b/191.tsv) / [후보](w/c/191.tsv) |
| 192 | [base](n/b/192.tsv) / [후보](n/c/192.tsv) | [base](w/b/192.tsv) / [후보](w/c/192.tsv) |
| 193 | [base](n/b/193.tsv) / [후보](n/c/193.tsv) | [base](w/b/193.tsv) / [후보](w/c/193.tsv) |
| 194 | [base](n/b/194.tsv) / [후보](n/c/194.tsv) | [base](w/b/194.tsv) / [후보](w/c/194.tsv) |
| 195 | [base](n/b/195.tsv) / [후보](n/c/195.tsv) | [base](w/b/195.tsv) / [후보](w/c/195.tsv) |
| 196 | [base](n/b/196.tsv) / [후보](n/c/196.tsv) | [base](w/b/196.tsv) / [후보](w/c/196.tsv) |
| 197 | [base](n/b/197.tsv) / [후보](n/c/197.tsv) | [base](w/b/197.tsv) / [후보](w/c/197.tsv) |
| 198 | [base](n/b/198.tsv) / [후보](n/c/198.tsv) | [base](w/b/198.tsv) / [후보](w/c/198.tsv) |
| 199 | [base](n/b/199.tsv) / [후보](n/c/199.tsv) | [base](w/b/199.tsv) / [후보](w/c/199.tsv) |
| 200 | [base](n/b/200.tsv) / [후보](n/c/200.tsv) | [base](w/b/200.tsv) / [후보](w/c/200.tsv) |
| 201 | [base](n/b/201.tsv) / [후보](n/c/201.tsv) | [base](w/b/201.tsv) / [후보](w/c/201.tsv) |
| 202 | [base](n/b/202.tsv) / [후보](n/c/202.tsv) | [base](w/b/202.tsv) / [후보](w/c/202.tsv) |
| 203 | [base](n/b/203.tsv) / [후보](n/c/203.tsv) | [base](w/b/203.tsv) / [후보](w/c/203.tsv) |
| 204 | [base](n/b/204.tsv) / [후보](n/c/204.tsv) | [base](w/b/204.tsv) / [후보](w/c/204.tsv) |
| 205 | [base](n/b/205.tsv) / [후보](n/c/205.tsv) | [base](w/b/205.tsv) / [후보](w/c/205.tsv) |
| 206 | [base](n/b/206.tsv) / [후보](n/c/206.tsv) | [base](w/b/206.tsv) / [후보](w/c/206.tsv) |
| 207 | [base](n/b/207.tsv) / [후보](n/c/207.tsv) | [base](w/b/207.tsv) / [후보](w/c/207.tsv) |
| 208 | [base](n/b/208.tsv) / [후보](n/c/208.tsv) | [base](w/b/208.tsv) / [후보](w/c/208.tsv) |
| 209 | [base](n/b/209.tsv) / [후보](n/c/209.tsv) | [base](w/b/209.tsv) / [후보](w/c/209.tsv) |
| 210 | [base](n/b/210.tsv) / [후보](n/c/210.tsv) | [base](w/b/210.tsv) / [후보](w/c/210.tsv) |
| 211 | [base](n/b/211.tsv) / [후보](n/c/211.tsv) | [base](w/b/211.tsv) / [후보](w/c/211.tsv) |
| 212 | [base](n/b/212.tsv) / [후보](n/c/212.tsv) | [base](w/b/212.tsv) / [후보](w/c/212.tsv) |
| 213 | [base](n/b/213.tsv) / [후보](n/c/213.tsv) | [base](w/b/213.tsv) / [후보](w/c/213.tsv) |
| 214 | [base](n/b/214.tsv) / [후보](n/c/214.tsv) | [base](w/b/214.tsv) / [후보](w/c/214.tsv) |

## PR 주장에 맞는 대표 쪽

대표는 각 PR의 변경 동작과 실제 편집 후 상태를 기준으로 명시적으로 선택했다. 최초 조건·높은 점수·다른 문서의 임의 페이지로 자동 선택하지 않았다. API·파싱·함수 반환 계약에는 관계없는 PNG를 붙이지 않고 실제 실행 함수를 개별 리뷰에 연결했다.

| 원 PR | 실제 주장·대표 쪽 | Native review / overlay | fresh review / overlay |
| --- | --- | --- | --- |
| #7559 | 077 p1: 새 표를 HWPX로 저장·재열기한 표와 뒤 본문의 원점 | [review](panels/native/077-p001-review.png) / [overlay](panels/native/077-p001-overlay.png) | [review](panels/wasm/077-p001-review.png) / [overlay](panels/wasm/077-p001-overlay.png) |
| #7572 | 178 p1: 본문 조합 뒤 첫 문단과 명시적 배치의 최종 출력; 배치 중 재조판 지연은 실제 API 검사로 구분 | [review](panels/native/178-p001-review.png) / [overlay](panels/native/178-p001-overlay.png) | [review](panels/wasm/178-p001-review.png) / [overlay](panels/wasm/178-p001-overlay.png) |
| #7573 | 181 p1: 시작 경계에서 오려 붙인 뒤 표 하나와 앞뒤 텍스트의 소유·순서 | [review](panels/native/181-p001-review.png) / [overlay](panels/native/181-p001-overlay.png) | [review](panels/wasm/181-p001-review.png) / [overlay](panels/wasm/181-p001-overlay.png) |
| #7588 | 021 p1: 30도 회전 묶음을 푼 뒤 자식과 누름틀의 유지 | [review](panels/native/021-p001-review.png) / [overlay](panels/native/021-p001-overlay.png) | [review](panels/wasm/021-p001-review.png) / [overlay](panels/wasm/021-p001-overlay.png) |
| #7589 | 074 p1: 여러 줄 입력 상자의 한컴 Print 표시와 false 대조군 | [review](panels/native/074-p001-review.png) / [overlay](panels/native/074-p001-overlay.png) | [review](panels/wasm/074-p001-review.png) / [overlay](panels/wasm/074-p001-overlay.png) |
| #7662 | 192 p1: 개체 바로 앞 문단 나눔 뒤 인라인 사각형과 뒤 텍스트의 다음 문단 소속 | [review](panels/native/192-p001-review.png) / [overlay](panels/native/192-p001-overlay.png) | [review](panels/wasm/192-p001-review.png) / [overlay](panels/wasm/192-p001-overlay.png) |
| #7694 | 125 p1: 차트를 논리 슬롯으로 옮긴 뒤 본문·차트·후속 문단의 순서 | [review](panels/native/125-p001-review.png) / [overlay](panels/native/125-p001-overlay.png) | [review](panels/wasm/125-p001-review.png) / [overlay](panels/wasm/125-p001-overlay.png) |
| #7819 | 082 p1: 비 BMP 글자 뒤 미주가 있는 실제 출력과 논리 슬롯의 API 검사 | [review](panels/native/082-p001-review.png) / [overlay](panels/native/082-p001-overlay.png) | [review](panels/wasm/082-p001-review.png) / [overlay](panels/wasm/082-p001-overlay.png) |
| #7833 | 134 p1: 셀 경계 오리기 실행 후 결과; 오리기 전 133은 대조군 | [review](panels/native/134-p001-review.png) / [overlay](panels/native/134-p001-overlay.png) | [review](panels/wasm/134-p001-review.png) / [overlay](panels/wasm/134-p001-overlay.png) |
| #7904 | 211 p1: 실제 WASM의 인라인 그림 뒤 IME 확정 결과; 확정·취소·Undo/Redo 상태 기록과 연결 | [review](panels/native/211-p001-review.png) / [overlay](panels/native/211-p001-overlay.png) | [review](panels/wasm/211-p001-review.png) / [overlay](panels/wasm/211-p001-overlay.png) |
| #8014 | 195 p1: NONE 선이 보이지 않는 Print 출력과 실선 196 대조군 | [review](panels/native/195-p001-review.png) / [overlay](panels/native/195-p001-overlay.png) | [review](panels/wasm/195-p001-review.png) / [overlay](panels/wasm/195-p001-overlay.png) |
| #8025 | 113 p4: bold=false인 굵은 표시용 실제 HY헤드라인M의 Canvas 잉크 | [review](panels/native/113-p004-review.png) / [overlay](panels/native/113-p004-overlay.png) | [review](panels/wasm/113-p004-review.png) / [overlay](panels/wasm/113-p004-overlay.png) |
| #8027 | EMR_POLYGON16·EMR_POLYLINE16의 원시 레코드 번호와 파싱·닫힘 경로를 실제 네 함수에서 검사한다. 일반 OLE 차트 PNG를 이 레코드 결함 검출의 증거로 사용하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8029 | PostScript 굵기 접미사에 대한 실제 메트릭 조회 결과·별칭·미지원 이름의 API 계약을 검사한다. HY헤드라인M PNG를 Pretendard 접미사 조회의 증거로 사용하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8030 | 전진폭보다 넓음·좁음·같음·폭 없음 네 독립 함수 경계를 검사한다. 표시용 굵은 글꼴 PNG를 반각 배율의 결함 검출로 사용하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8031 | 145 p1: 공개 API로 표 셀 안에 정의한 쪽 번호의 실제 출력 | [review](panels/native/145-p001-review.png) / [overlay](panels/native/145-p001-overlay.png) | [review](panels/wasm/145-p001-review.png) / [overlay](panels/wasm/145-p001-overlay.png) |
| #8032 | 141 p1: 큰 인라인 그림이 남은 줄 폭에 들어가지 않아 그림 앞에서 줄바꿈; 작은 그림 142는 같은 줄 정상 대조군 | [review](panels/native/141-p001-review.png) / [overlay](panels/native/141-p001-overlay.png) | [review](panels/wasm/141-p001-review.png) / [overlay](panels/wasm/141-p001-overlay.png) |
| #8033 | 170 p4: 중첩 표의 시작 조각·바깥여백과 뒤 본문; 같은 측정·배치 프레임 소비 | [review](panels/native/170-p004-review.png) / [overlay](panels/native/170-p004-overlay.png) | [review](panels/wasm/170-p004-review.png) / [overlay](panels/wasm/170-p004-overlay.png) |
| #8035 | 212 p1: 그림을 지운 뒤 앞글뒤글의 굵게 경계; 다른 두 삭제 시나리오 TSV와 API 경계 대조 | [review](panels/native/212-p001-review.png) / [overlay](panels/native/212-p001-overlay.png) | [review](panels/wasm/212-p001-review.png) / [overlay](panels/wasm/212-p001-overlay.png) |
| #8036 | 144 p1: 큰 저장 vpos 뒤 실제 한 쪽 출력과 뒤 문단 보존; 143은 영 원점 대조군 | [review](panels/native/144-p001-review.png) / [overlay](panels/native/144-p001-overlay.png) | [review](panels/wasm/144-p001-review.png) / [overlay](panels/wasm/144-p001-overlay.png) |
| #8037 | 065 p1: 쪽 기준 그림을 가진 표 문단의 양수 offset과 바깥여백 | [review](panels/native/065-p001-review.png) / [overlay](panels/native/065-p001-overlay.png) | [review](panels/wasm/065-p001-review.png) / [overlay](panels/wasm/065-p001-overlay.png) |
| #8038 | 073 p1: 첫 줄 윗강조점 잉크와 본문 clip 상단 | [review](panels/native/073-p001-review.png) / [overlay](panels/native/073-p001-overlay.png) | [review](panels/wasm/073-p001-review.png) / [overlay](panels/wasm/073-p001-overlay.png) |
| #8039 | 130 p1: TAC 표 뒤 컨트롤 전용 저장 줄의 점유와 NEXT 본문; false 대조군 131 | [review](panels/native/130-p001-review.png) / [overlay](panels/native/130-p001-overlay.png) | [review](panels/wasm/130-p001-review.png) / [overlay](panels/wasm/130-p001-overlay.png) |
| #8041 | 026 p1: 실제 자식 틀을 사용하는 도형 겹침 순서의 편집 후 출력 | [review](panels/native/026-p001-review.png) / [overlay](panels/native/026-p001-overlay.png) | [review](panels/wasm/026-p001-review.png) / [overlay](panels/wasm/026-p001-overlay.png) |
| #8042 | 052 p1: 묶기·풀기 편집 뒤 누름틀과 자식 회전 상태 | [review](panels/native/052-p001-review.png) / [overlay](panels/native/052-p001-overlay.png) | [review](panels/wasm/052-p001-review.png) / [overlay](panels/wasm/052-p001-overlay.png) |
| #8043 | 197 p1: 책갈피 링크 문단의 표시와 순서; 링크 대상 읽기·갱신은 별도 API 검사 | [review](panels/native/197-p001-review.png) / [overlay](panels/native/197-p001-overlay.png) | [review](panels/wasm/197-p001-review.png) / [overlay](panels/wasm/197-p001-overlay.png) |
| #8045 | 112 p1: 한컴 저장본의 셀 안 어울림 중첩 표 옆 글 시작 위치 | [review](panels/native/112-p001-review.png) / [overlay](panels/native/112-p001-overlay.png) | [review](panels/wasm/112-p001-review.png) / [overlay](panels/wasm/112-p001-overlay.png) |
| #8049 | 046 p1: 캡션 자동 번호·접두부·뒤 문자의 표시 | [review](panels/native/046-p001-review.png) / [overlay](panels/native/046-p001-overlay.png) | [review](panels/wasm/046-p001-review.png) / [overlay](panels/wasm/046-p001-overlay.png) |
| #8050 | 198 p1: NONE 셀 변을 표 외곽선으로 채우지 않는 실제 출력 | [review](panels/native/198-p001-review.png) / [overlay](panels/native/198-p001-overlay.png) | [review](panels/wasm/198-p001-review.png) / [overlay](panels/wasm/198-p001-overlay.png) |
| #8051 | 199 p1: 첫 표 조각의 cellzone 색과 외곽선; 뒤 빈 쪽은 쪽수 대조이며 가시 내용 증거가 아님 | [review](panels/native/199-p001-review.png) / [overlay](panels/native/199-p001-overlay.png) | [review](panels/wasm/199-p001-review.png) / [overlay](panels/wasm/199-p001-overlay.png) |
| #8052 | 실제 빈 마지막 문단의 명시적 쪽 나눔·쪽수 계약을 검사한다. 실제 빈 페이지 raw100을 가시 내용 배치 정확성으로 제시하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8053 | 202 p1: 용지 기준 머리말 도형과 본문 기준 도형의 상대 원점 | [review](panels/native/202-p001-review.png) / [overlay](panels/native/202-p001-overlay.png) | [review](panels/wasm/202-p001-review.png) / [overlay](panels/wasm/202-p001-overlay.png) |
| #8054 | 203 p1: offset 표 아래에서 같은 문단 본문이 재개하는 순서와 위치 | [review](panels/native/203-p001-review.png) / [overlay](panels/native/203-p001-overlay.png) | [review](panels/wasm/203-p001-review.png) / [overlay](panels/wasm/203-p001-overlay.png) |
| #8055 | 170 p8: 표 이어받기·종료 조각 뒤 첫 줄의 간격과 후속 본문 | [review](panels/native/170-p008-review.png) / [overlay](panels/native/170-p008-overlay.png) | [review](panels/wasm/170-p008-review.png) / [overlay](panels/wasm/170-p008-overlay.png) |
| #8056 | 205 p1: 줄 끝 공백이 있는 숫자의 실제 오른쪽 정렬; 204는 공백 없는 대조군 | [review](panels/native/205-p001-review.png) / [overlay](panels/native/205-p001-overlay.png) | [review](panels/wasm/205-p001-review.png) / [overlay](panels/wasm/205-p001-overlay.png) |
| #8057 | 107 p1: U+2024를 가운뎃점으로 사용한 실제 본문과 A·B 간격 | [review](panels/native/107-p001-review.png) / [overlay](panels/native/107-p001-overlay.png) | [review](panels/wasm/107-p001-review.png) / [overlay](panels/wasm/107-p001-overlay.png) |
| #8058 | 음수·양수 자간과 기호 잉크에 대한 실제 Canvas 배율 함수 계약을 검사한다. 일반 굵은 제목 PNG를 과도한 축소 검출로 사용하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8061 | 120 p1: 쪽 기준 떠 있는 그림의 높은 z-order와 실제 셀 내용 쌓기 순서; 115는 정상 대조군 | [review](panels/native/120-p001-review.png) / [overlay](panels/native/120-p001-overlay.png) | [review](panels/wasm/120-p001-review.png) / [overlay](panels/wasm/120-p001-overlay.png) |
| #8062 | 개체 전용 줄과 저장 vpos 차이의 실제 배치 계약을 검사한다. 이 최소 입력의 실제 빈 Print를 가시 본문의 위치 정확성으로 제시하지 않는다. | 실제 계약 함수의 PASS 연결 | 실제 계약·소비 경로와 구분 |
| #8065 | 207 p1: 안 여백 아래 가 글줄이 셀 안에 포함되는 실제 행 높이 | [review](panels/native/207-p001-review.png) / [overlay](panels/native/207-p001-overlay.png) | [review](panels/wasm/207-p001-review.png) / [overlay](panels/wasm/207-p001-overlay.png) |
| #8066 | 208 p1: 마지막 줄 간격만 넘치는 TAC RowBreak 행과 가 글줄 | [review](panels/native/208-p001-review.png) / [overlay](panels/native/208-p001-overlay.png) | [review](panels/wasm/208-p001-review.png) / [overlay](panels/wasm/208-p001-overlay.png) |
| #8090 | 209 p2: 두 구역 중 앞 구역에 삽입·편집한 미주가 마지막 쪽에 표시되고 캐럿 소속이 일치 | [review](panels/native/209-p002-review.png) / [overlay](panels/native/209-p002-overlay.png) | [review](panels/wasm/209-p002-review.png) / [overlay](panels/wasm/209-p002-overlay.png) |
| #8093 | 110 p1: 가와 나 사이 TAC 양식 개체의 표시; 논리 한 칸·캐럿은 API 검사 | [review](panels/native/110-p001-review.png) / [overlay](panels/native/110-p001-overlay.png) | [review](panels/wasm/110-p001-review.png) / [overlay](panels/wasm/110-p001-overlay.png) |
| #8094 | 046 p1: 캡션 자동 번호와 뒤 글자의 표시; 캐럿·클릭 역변환은 API 검사 | [review](panels/native/046-p001-review.png) / [overlay](panels/native/046-p001-overlay.png) | [review](panels/wasm/046-p001-review.png) / [overlay](panels/wasm/046-p001-overlay.png) |
| #8096 | 126 p1: 저장 줄 없는 양식 전용 문단의 다섯 개체 표시 | [review](panels/native/126-p001-review.png) / [overlay](panels/native/126-p001-overlay.png) | [review](panels/wasm/126-p001-review.png) / [overlay](panels/wasm/126-p001-overlay.png) |

## 실행 완료와 잔여 차이

전체 nextest 11,832 PASS / 0 FAIL / 45 skipped, release lib 묶음 3,983 PASS / 0 FAIL / 12 ignored, Skia lib 묶음 4,040 PASS / 0 FAIL / 12 ignored, 집중 73 PASS, doctest 8 PASS / 0 FAIL / 3 ignored. Studio Node 2,597 PASS와 실제 fresh 글꼴 추적·production build·브라우저 3종을 종료 코드0으로 확인했다. 원문 로그는 커밋하지 않았다.

대표 쪽의 glyph·선 농도·작은 AA 잔차는 [직접 판독](visual-evidence.json)에 별도로 기록했다. API 계약 PASS를 한컴 시각 일치나 실제 브라우저 이벤트 실행으로 바꾸어 보고하지 않는다. #8090 캐시·캐럿과 #7904 입력·이력은 같은 세션의 실제 WASM 상태로 입증하고 정적 PNG와 구분했다. 원격 CI·merge·후속 처리는 아직 별도 절차다.
