# samples/issue9285 — 색상표 없는 WMF 1비트 마스크 DIB

[#9285](https://github.com/edwardkim/rhwp/issues/9285)의 정식 회귀 fixture다.

산업통상자원부 보도자료 공개 문서(「제4차 어린이제품 안전관리 기본계획 발표」, HWPX, 9쪽)
원본이다.

| 항목 | 값 |
| --- | --- |
| 대상 그림 | 4쪽 1×4 표 셋째 칸 — `BinData/image7.wmf` |
| 레코드 | `META_DIBSTRETCHBLT` 쌍 넷 — 마스크 `SRCPAINT`(0x00EE0086) + 그림 `SRCAND` |
| 마스크 DIB | 512×512, 1비트, `BITMAPINFOHEADER` 40바이트 + 픽셀 32,768바이트 — 색상표 없음 |
| 근거 | [MS-WMF 2.2.2.9 DeviceIndependentBitmap](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/7376542a-cce9-4625-8ead-585e9538f9f1)의 색상표 크기 규칙 |

- 회귀 계약: 전체 9쪽. 이 WMF 가 변환되어 어느 쪽의 SVG 에도 변환하지 못한 WMF 원본
  (`image/x-wmf`)이 남지 않는다.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 원본 저장 제품은 `version.xml`의 `appVersion="11, 0, 0, 8658"`로 확인한 한컴 2020이다.

공식 기준은 Windows 데스크톱 한컴 2020 `11.0.0.9136`의 Hancom PDF Print 9쪽이다.
첨부본의 한컴 2022를 확인한 뒤 MCP 2024로도 인쇄했다. 2024 인쇄본에서 3·7·9쪽의 영문·숫자
누락을 발견해, 전체 9쪽의 문자를 대조한 MCP 2020 인쇄본을 사용한다.
[기준 PDF](../../pdf/issue9285/child-product-safety-plan-wmf-mask-dib-2020.pdf) ·
[인쇄 영수증과 쪽별 대조](../../mydocs/pr/assets/planet6897_9240_9282_20261121/wmf-9285/print-verification.json)
