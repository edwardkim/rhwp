# 어울림 그림 바깥 여백 이중 적용 수정 증거 (#8730)

- 측정 source: head `21e4d05da3`(수정 커밋, 이후 커밋은 검사·증거뿐), base `b8dd67c500`
- 96 DPI, `silhouette_raw_match_percent`, 독립 한컴 Print PDF
- `sample5`, `hwp5`, `sample5hwpx`는 굴림만 공급했다. 큰 글꼴 경로는 Native 64MiB 예산을 넘겨 한글이 빠진다.
- `<입력>/{base,head}_{native,wasm}_silhouette.tsv`: 전쪽 TSV 원본
- `rep/{native,wasm}/`: 대표 쪽 review·overlay PNG

| 입력 | 쪽 | base Native/WASM | head Native/WASM | head 전쪽 최저 N/W |
|---|---:|---:|---:|---:|
| hwp3-sample5.hwp | 22 | 97.90 / 97.90 | 100.00 / 100.00 | 95.70 / 95.63 |
| hwp3-sample5-hwp5.hwp | 22 | 93.59 / 93.69 | 100.00 / 100.00 | 99.51 / 99.49 |
| square_picture_band_host.hwp | 1 | 80.82 / 79.48 | 92.15 / 93.42 | 92.15 / 93.42 |
| employment_service_interim_report.hwp | 14 | 91.27 / 90.99 | 99.92 / 99.88 | 94.36 / 94.30 |
| hwp3-sample5-hwpx.hwpx | 16 | 89.76 / 89.65 | 89.76 / 89.65 | 89.76 / 89.65 |

`hwp3-sample5-hwpx.hwpx` 16쪽은 수정 전후 값이 같은 기존 미달이다. 회귀 입력에서 제외했다.
