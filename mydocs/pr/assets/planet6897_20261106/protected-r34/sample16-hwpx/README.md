# 원 HFT 대응 OTF의 굵기 소비 — 추가 보호 대조군

이 자료는 기존 PR #8389·#8397의 보호 입력 검증이다. 원 PR #8605의 30396 권고문 주장에
대한 증거와 구분하며 다른 원본을 검증한 것으로 대체하지 않는다.

- 입력: `samples/hwp3-sample16-hwp5.hwpx`, SHA-256 `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`.
- 독립 한컴 2024 Print: `pdf/hwp3-sample16-hwp5-hwpx-2024.pdf`, SHA-256 `be4a0286d3ad5209a1a0787fd4ff691e110ddbcf7bde3a52a4547958e452a1ee`.
- 기존 PDF를 그대로 사용했다. 재출력·문서 변경·그림 제외는 하지 않았다.
- 조건: 2026-11-07, Print, compat 2024, 96 DPI, 보정 전 2px raw.
- 전체 시스템 글꼴과 기존 승인 `hwp3-2024-font-environment.json`을 적용했다.
- Native는 ef77d740e1의 실제 출력이다. 99f1340628까지 제품 diff가 wasm32 전용 함수·모듈뿐인
  것을 확인해 동일한 Native 결과를 재사용한다. 새로운 Native 실행으로 표현하지 않는다.
- fresh WASM 제품 source: `99f13406287499302e564d27ab8b61d38287e759`.
- WASM SHA-256: `3626cf87f10ce7412bf2f22e8ddf92baed12fe608584db5ff4b90812912d0f69`.
- 실행: `scripts/visual_sweep.py --silhouette-only --resume --hwp samples/hwp3-sample16-hwp5.hwpx
  --pdf pdf/hwp3-sample16-hwp5-hwpx-2024.pdf --key sample16-hwpx --compat 2024 --dpi 96`,
  Native는 `rhwp-batch-corrections-r33`, WASM은 추가로 `--wasm-pkg wasm-hft-bold-r34`를 지정했다.

| 경로 | 전체 쪽수 | 전쪽 최저 raw | 40쪽 | 51쪽 | 전체 TSV |
| --- | ---: | ---: | ---: | ---: | --- |
| Native | 64 | 99.99% | 100.00% | 100.00% | [native.tsv](native.tsv) |
| fresh WASM | 64 | 99.99% | 100.00% | 100.00% | [wasm.tsv](wasm.tsv) |

Native/fresh WASM 모두 독립 PDF와 64쪽이 일치한다. 기존 보호 TSV의 같은 64행을
소수 셋째 자리에서 반올림한 표시값으로 대조했고 하락은 0행이다. 실제 전체 최저값을
대표 2쪽의 100.00%로 바꾸어 표시하지 않는다. 다른 보호 문서의 전쪽 검증은 별도다.

- Native TSV SHA-256: `804fedcc655075b2739abaf7adfb4764c1938357fcaeaaceb62dca421ecb4f49`.
- WASM TSV SHA-256: `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa`.

## 실제 같은 쪽의 판독

40쪽은 굵은 ‘건설·관리’의 한점과 앞뒤 본문, 51쪽은 ‘○○ 사업 제안응모 신청서’의
원 기호·제목·밑줄과 뒤 표·본문을 같은 배율로 대조한다. 51쪽은 PDF 인쇄 쪽 번호49에
해당한다. 글꼴 프로그램을 포함하지 않으며 기존 OTF의 CFF 윤곽을 실제 paint에서 소비한다.

[40쪽 review](wasm-review-040.png) · [40쪽 overlay](wasm-overlay-040.png)
[51쪽 review](wasm-review-051.png) · [51쪽 overlay](wasm-overlay-051.png)

raw 100.00%는 2px 이웃 실루엣의 표시값이다. 기존 선·획의 서브픽셀 래스터 차이가 있으므로
픽셀 동일이라고 주장하지 않는다. 표의 칸·내용·소유는 보존되며 이번 보정은 glyph 소비에 한정된다.
