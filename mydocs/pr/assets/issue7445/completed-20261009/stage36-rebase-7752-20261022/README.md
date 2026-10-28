# #7752 리베이스의 고정 devel Native 대조군

- 기준: `2e087a5b4bc1515ad877e8020b30114639a59392`(#8148). 정확한 Git 소스 archive를 같은 Mac에서 `release-test/native-skia`, locked, jobs 8로 빌드했다.
- 실제 Native CLI SHA-256: `39beec91d5bdfcc3a356071f3c497efdf8bd3f4cf402df6fa72de7c47a211913`.
- 각 원본과 정상 KoPub 한컴 2024 Print PDF는 아래 기존 정식 경로를 사용했다. 실행 파일과 기준 commit의 바이트가 같은지 확인했다.
  - `samples/2025 행정업무운영 편람(최종).hwp` ↔ `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`
  - `samples/2025 행정업무운영 편람(최종).hwpx` ↔ `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`
- print, compat 2024, 96dpi, 현재 PDFium 변환과 색상 경계 보정 전 2px raw 계산, capture 총 8이다. 실제 설치 글꼴 713개의 SHA를 고정했으며 이전 공급 조건 대비 변경 0개다.
- 아래 수치는 **고정 devel의 결과**다. 후보 통과로 해석하지 않는다. 같은 Mac에서 두 형식 모두 277쪽이 기존 미달로 관측됐다. Linux의 원 PR 수치를 이 실행의 결과로 대신하지 않았다.

| 원본 | 전쪽 수 | 최저 raw | 90% 미달 쪽 | 94쪽 raw | 전쪽 TSV |
| --- | ---: | --- | --- | --- | --- |
| handbook-hwpx | 383 | 88.76094% · 277쪽 | 277 | 92.69771% | [devel-native-handbook-hwpx.tsv](devel-native-handbook-hwpx.tsv) |
| handbook-hwp | 383 | 88.85188% · 277쪽 | 277 | 92.63808% | [devel-native-handbook-hwp.tsv](devel-native-handbook-hwp.tsv) |

TSV는 생성 파일의 바이트를 그대로 보존했다. 후보 전쪽 비교·fresh WASM·확대 판독 및 실제 소비 경로 검증은 별도이며 현재 제출 조건은 미완료다.
