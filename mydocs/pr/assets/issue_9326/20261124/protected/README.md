# 정본 보호 비교

제품 검증 커밋은 `24ec51a6f9186d509d3f3285faa6f9f6e067af5e`이며 비교 devel은
`2c0c1d205a54b8d5bd262241bac1391cffe0d801`이다. 동일 제품 산출물의 46개 보호 조건·4,046쪽에서
공식 두 자리 표시값이 낮아진 쪽은 0쪽이다. 공급 환경을 추가 확인한 조건은 다음과 같다.

| 조건 | 쪽수 | 후보 최저 | 보호값보다 낮은 쪽 |
| --- | ---: | ---: | ---: |
| 종합부동산세법 Native | 6 | 98.86% | 0 |
| 종합부동산세법 fresh WASM | 6 | 98.86% | 0 |
| sample10 Native | 763 | 99.57% | 0 |
| 중첩 표 Native | 17 | 100.00% | 0 |

중첩 표 Native는 [정본 공급 목록](../../../../../verification/protected-font-supply/manifest.json)의
`nested`에 등록된 21개 파일의 SHA-256·배열 순서와
[Native 환경 설정](../../../../../verification/protected-font-supply/environments/nested-table-native.json)을
사용했다. 고정 devel과 후보의 전쪽 PNG 해시가 일치했다. 동일 HWP, 한컴 2020 Hancom PDF,
compat 2022, 96 DPI, 2px raw 실루엣으로 비교했다. 2·7쪽은 99.99%에서, 17쪽은 99.98%에서
각각 100.00%로 상승했다.

[고정 devel 17쪽 TSV](nested-native-devel.tsv) · [후보 17쪽 TSV](nested-native-candidate.tsv)
