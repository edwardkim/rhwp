# 고사성어 원 기호 소유 보정의 실제 검증

- 제품 source: `ef77d740e1da7c9eaeb908ac3bd653a9670b312a`. 이후 테스트·문서 추가는 이 제품 실행과 구분한다.
- 측정: 2026-11-07, Print, 96 DPI, compat 2022, 색상 경계 보정 전 2px raw.
- Native/fresh WASM 모두 독립 PDF와 전체 94쪽이 일치한다.
- 실제 글꼴 목록과 선택은 backend 실행별로 기록했다. 같은 목록이 같은 face 선택을 보장한다고 가정하지 않는다.

| 기준 | 경로 | SHA-256 |
| --- | --- | --- |
| input | [sungeo.hwp](../../../../../../samples/basic/sungeo.hwp) | `3b13d15833cbd0fb42b5f7ac5909eee7a68ec4505a914eb70931168c01f79d96` |
| pdf | [sungeo-hwp-2020.pdf](../../../../../../pdf/basic/sungeo-hwp-2020.pdf) | `4b7a90e1cb7b8fae8da4b5c4bfbbb1f22858f1ff9b4cbd3ca13ea5cb3c924e1e` |

| 출력 | 전쪽 최저 | 검사 5쪽 | 전체 TSV | 5쪽 review / overlay |
| --- | ---: | ---: | --- | --- |
| native | 99.98% | 100.00% | [native.tsv](native.tsv) | [review](native-review-005.png) / [overlay](native-overlay-005.png) |
| wasm | 99.99% | 99.99% | [wasm.tsv](wasm.tsv) | [review](wasm-review-005.png) / [overlay](wasm-overlay-005.png) |

## 실제 실행과 무결성

같은 원본·PDF에서 산출한 전쪽 TSV와 같은 5쪽 패널을 재사용한다. PDF를 다시 출력하지 않았다.
```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --hwp samples/basic/sungeo.hwp --pdf pdf/basic/sungeo-hwp-2020.pdf \
  --key sungeo --rhwp-bin output/pr-review/planet6897-integration-20261106/rhwp-batch-corrections-r33 \
  --compat 2022 --dpi 96 --out output/pr-review/planet6897-integration-20261106/r33-direction-native/sungeo
# WASM: 같은 옵션에 --wasm-pkg output/pr-review/planet6897-integration-20261106/wasm-batch-corrections-r33 추가,
 # out은 r33-direction-wasm/sungeo를 사용한다.
```

- native TSV SHA-256: `88a242c62a57837a07e9eaad3dabb8d1eef64b9572115c22ee42a75fd81eda3d`.
- native 실제 5쪽 PNG SHA-256: `52ff77cd4079031f95372cfd3b79d3d7aead8ac4fb9160b40ce3e1fb726f4ad2`.
- native producer SHA-256: `6ca45decd6793ba38e33c895d3571fc16d0c2c8395ebfa19297bb95fde445e22`.
- wasm TSV SHA-256: `bdc6456db9aac7b9cacefcb0412b41d6b725cd622c8696c11c362589ff850da1`.
- wasm 실제 5쪽 PNG SHA-256: `b2045a851d711a54782d94c81ca4ce7b9ce7d4a4eb3dbbdd0174e514d590c99f`.
- wasm rhwp.js SHA-256: `0099b8a6e00f750e1dbddb3a89bf42874f24effcf8bf4dc5eb328d69e691606b`.
- wasm rhwp_bg.wasm SHA-256: `e3d498d8b935eed0da341c5cb64dafdca07201b620b932d3ff0c7273687214ca`.

## 직접 판독과 검사 경계

5쪽의 첫 두 머리 기호·한자 제목·ASCII 괄호·다음 본문을 같은 배율로 확대했다.
원 기호의 크기와 양각을 복원했으며 본문 줄 소속·내용 순서·원점은 유지된다.
PDF와 글꼴 래스터의 획 두께·서브픽셀 차이는 남아 있어 raw 100.00%를 픽셀 동일로 표현하지 않는다.
독립 PDF의 Wingdings-Regular glyph 3은 UPEM 2048, advance 937, bbox (173,444)..(765,1036)이다.
원 기호의 em 계약과 실제 Native/Canvas 소비를 글꼴 설치에 의존하지 않는 회귀로 보호한다.
전27 PR 최종 승인·전체 Rust·lint·보호 문서 전쪽 비교는 별도 완료 조건이다.
