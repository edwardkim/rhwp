# samples/issue9285 — 색상표 없는 WMF 1비트 마스크 DIB

[#9285](https://github.com/edwardkim/rhwp/issues/9285)의 정식 회귀 fixture다.

산업통상자원부 보도자료 공개 문서(「제4차 어린이제품 안전관리 기본계획 발표」, HWPX, 9쪽)
원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 그림 | 4쪽 1×4 표 셋째 칸 — `BinData/image7.wmf` |
| 레코드 | `META_DIBSTRETCHBLT` 쌍 넷 — 마스크 `SRCPAINT`(0x00EE0086) + 그림 `SRCAND` |
| 마스크 DIB | 512×512, 1비트, `BITMAPINFOHEADER` 40바이트 + 픽셀 32,768바이트 — 색상표 없음 |
| 근거 | [MS-WMF] 2.2.2.9 DeviceIndependentBitmap — `Colors` 의 크기는 담는 레코드에서 정한다 |

- 회귀 계약: 전체 9쪽. 이 WMF 가 변환되어 어느 쪽의 SVG 에도 변환하지 못한 WMF 원본
  (`image/x-wmf`)이 남지 않는다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `version.xml` 의 `appVersion="11, 0, 0, 8658"`(2024 저장본이 아니므로
  engine 2020).

기준 PDF: [`pdf/issue9285/child-product-safety-plan-wmf-mask-dib-2020.pdf`](../../pdf/issue9285/child-product-safety-plan-wmf-mask-dib-2020.pdf)
— 665,605 bytes, SHA-256 `975e799947cd2d3f66f7687fc05b8513cfcb1d84580f5591cb0b146d15825fc8`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), 9쪽.
