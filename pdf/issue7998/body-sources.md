# P3 본문 문단 원본

태그 없는 독립 PDF 연산자로 작성했다. 기대 문단은 제품의 인식 결과가 아니라
`generate_body_sources.py`의 생성 주석에서 정한다. 시각 기준은 이 원본 PDF이며
엔진의 PNG를 golden으로 사용하지 않는다. 생성기는 로컬 source 주석도 출력한다.

96 DPI·compat 2022의 Native Skia/fresh Docker WASM에서 PDF·HWP·HWPX를 비교했다.
저장 후 재열기의 PNG는 같은 backend의 직접 PDF 입력과 모두 동일했다. 두 단·제목·
들여쓰기·기준선·모호한 소유·paint 중첩·그림 앞뒤의 확대 판독을 별도로 수행했다.
아래 값은 해당 원본의 직접 입력과 두 저장 형식 중 최저값이다.

| 원본 | SHA-256 | Native 최저 % | WASM 최저 % |
| --- | --- | --- | --- |
| body-ambiguous.pdf | a7961dd1872fdc0c87fafe4ae4bacb74a28ec8db6726e672893616405b7c4362 | 100.00000 | 100.00000 |
| body-baseline.pdf | da32520b1e1ff7e185f025769c50b0c8c08d872cd51a79a7c3e6b879e1c13d79 | 100.00000 | 100.00000 |
| body-basic.pdf | 87ff7c61595a04c6e2ebb6b8ba67a3002c15c070b1f7368d09f3d23b6b51a566 | 100.00000 | 100.00000 |
| body-center.pdf | e6fffd6a1bf9c32d11c2d6df2f0b67bf01e9873740a9980b86ebc38a269ce2f4 | 100.00000 | 100.00000 |
| body-close-title.pdf | 764e37e11eb4f324ab0eae989f78fabeff82a3b29a478ad40fd848ac1ebeb0ce | 99.95064 | 99.80459 |
| body-columns.pdf | 999effd0bea78b4bfc7a94a75d712d5aa9eb858b410dfd5acc067ad85b100c4b | 100.00000 | 100.00000 |
| body-indent.pdf | 8c857be28916d445a4205b87fbcd84c48e38fe8975f12130d8c28db4f6eaac36 | 100.00000 | 100.00000 |
| body-list.pdf | 7933b277f1456516fc078681f3ef1db5b0e147580a0e68c4086c43b0c96c8acc | 100.00000 | 100.00000 |
| body-occlusion.pdf | 2e82d29c9d7cac4f1330668bef6790f0aa780f5ebdf8598dd7aa4027a32e6115 | 100.00000 | 100.00000 |
| body-picture.pdf | 89c18d66a080543ab1b117faee1260a18e3fd871568300dd1189935500d60a5d | 99.94388 | 99.94373 |
| body-reordered.pdf | b66bdd0e7d5fcf882607120a68e85561fd2a731bd8389e9fdb437c93667d7a7b | 100.00000 | 100.00000 |
| body-title-columns.pdf | 52bc11085cc91a0705429fb12eae0150ed4ce5f0ee316cd1bbd8dd0d4b552df7 | 100.00000 | 100.00000 |
| body-tj.pdf | 493c6efeda07d0b9c71f5140f1ca71670bb4447731c7907979e6c7c8043a2806 | 100.00000 | 100.00000 |
| body-unrepresentable.pdf | 0b7078e6fc88f083b6a223646d504c3a27f4300e58b79e0bc92dfaa110580606 | 100.00000 | 100.00000 |

producer source는 `77bf8d0dd715acf00f661b4969544a83076c3f22` 위의 P3 변경이며,
정확한 소스 지문·명령·실행 결과는 stage3 보고서에 기록했다. 런타임은 캡처 전에 고정했다.

- Native SHA-256: `4c048d888a4a2ff1c64123125ab87a6ec4bcba05471d8584cfbc85934b8a6987`
- WASM SHA-256: `b360c49ed3a435374d3c4dbf28ed82a6b37f813df3e20491f14558f331292fc8`

전쪽 TSV·명령·원본 주석·직접 비교는 ignored `output/pr-review/issue7998/p3-body/`에
보존했다. 검사는 내용·문단/글줄 소속·배치 관계와 편집·저장 경로를 보호한다. 작성 당시의
Tagged PDF 복원이나 한컴편집기 상호운영 완료를 주장하지 않는다. 한컴 통합 확인과
최신 devel 비교는 P5에 남는다. fixture 등록이 이전 P2 대비 점수 하락을 면제하지 않는다.
