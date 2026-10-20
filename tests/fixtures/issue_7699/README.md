# #7699 입력과 독립 기준

원본은 [한국정보기술학회 공식 논문 양식](https://ki-it.or.kr/board/maininfo/article/284486)의 첨부다. 원 PR 기여자의 MD5 `92a2749109181e14440e02eaa6ce6680`과 동일하다.

- `kiit-standard-original.hwp`: 원본 바이트 그대로.
- `kiit-standard-hancom.hwpx`: 원본을 실제 한컴2020에서 HWPX로 저장한 정상 대조군.
- `kiit-standard-allow-overlap.hwpx`: 정상 HWPX의 첫 Square 표 `pos/@allowOverlap`만0→1. 나머지 XML·저장 LineSeg는 동일하다. 독립 Print 전3쪽은 정상 HWPX와 RGB 동일하다.
- `kiit-narrow85-reflow-input.hwpx`: 정상 HWPX의 첫 Square 표 및 그 셀 너비만36142HU(본문42520HU의85%)로 변경하고 무효화된 LineSeg 배열103개를 제거한 재생성 입력.
- `kiit-narrow85-hancom.hwp`: 위 재생성 입력을 실제 한컴2020에서 HWP 저장하여 정상 저장 LineSeg를 다시 생성했다. 원본의 일치 증거를 이 대조군으로 대신하지 않는다.

독립 Print는 `pdf/pr7699-*-2020.pdf`에 있다. 모두 실제 MCP engine2020, Hancom11.0.0.9136, PrintMethod0(1-up), 전3쪽이다. GUI/LibreOffice/RHWP PDF는 기준으로 사용하지 않았다. 글꼴은 호스트 원본 설치 경로에서 직접 공급하며 배포 폰트는 추가하지 않는다.

| 파일 | 바이트 | SHA-256 |
| --- | ---: | --- |
| `tests/fixtures/issue_7699/kiit-standard-original.hwp` | 180224 | `0e9d59a6ecf52dce759598997568680edc3955b189efdfb9b115ec9b2cc3057c` |
| `tests/fixtures/issue_7699/kiit-standard-hancom.hwpx` | 100850 | `f330fdcff5e78d50bed278a5bbfc514409caa00053f53ba4e7c24cdc4eb33b99` |
| `tests/fixtures/issue_7699/kiit-standard-allow-overlap.hwpx` | 97192 | `7ffa4d23b9e70d6e9f0b0d9f2c31852a9cd06a1a7f21dc4b10522aa0d29a9e26` |
| `tests/fixtures/issue_7699/kiit-narrow85-hancom.hwp` | 103936 | `3886833ba1af65f61a0cbcededbba73c0b3f1a2a7c83f65f03ccc19ae22e07f2` |
| `tests/fixtures/issue_7699/kiit-narrow85-reflow-input.hwpx` | 95416 | `cc60b3bb6d64e4fdfc553ebbda466569c4347424d37a3d72b1ffe353e242adb4` |
| `pdf/pr7699-kiit-allow-overlap-2020.pdf` | 226625 | `630d747445d940b6f189f5ac9e5ec3e08cad41163b42643b1427a54af1861598` |
| `pdf/pr7699-kiit-hancom-hwpx-2020.pdf` | 226625 | `e58e449ece97a37694824042dbe7764095fcaf462bd883fbf8155399f6bb710f` |
| `pdf/pr7699-kiit-narrow85-2020.pdf` | 227007 | `77507f0aaca100b8a46381c71b4b30d5c4ad4323d3bc37ee0b01c3e5d5548a10` |
| `pdf/pr7699-kiit-standard-2020.pdf` | 226625 | `e9ed8db0abf47eaaef4d0c14aed49a5a1943f29c304555dc076593222b8c77fb` |

전쪽 TSV·Native/fresh WASM 수치와 중요한 페이지의 독립 px 기준선은 [시각 증적](../../../mydocs/pr/assets/pr7699/README.md)에 기록한다.
