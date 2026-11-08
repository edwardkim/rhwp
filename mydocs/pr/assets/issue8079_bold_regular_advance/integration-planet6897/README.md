# #8079 통합 후보 Print 대조

2026-11-01 KST, 96 DPI에서 세 문서의 독립 한컴 Print와 전쪽을 비교했다. 각 6쪽이며 최저값은 아래 표와 같다. 고정 좌표 검사의 1쪽은 두 출력 모두 98% 이상이다. 독립 PDF와 같은 쪽의 글줄·숫자 간격 및 HFT 굵은 제목을 확대해 확인했다.

- Native 실행 파일 SHA-256: `4b03859cfa21cdba1a766d5141d260adf70d3c986c6481a3e8440d1ab992ae96`.
- WASM SHA-256: `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`.
- 이 증거는 제품 코드 생산 커밋 `6be28314c5`의 이전 통합 후보를 대상으로 한다. 이후 제품 코드가 변경됐으므로 최신 head의 재검증 결과로 취급하지 않는다. 시스템 글꼴을 사용했고 글꼴 파일은 추가하지 않았다.
- 표의 TSV에 사용한 1쪽 PNG와 직접 판독한 review·overlay 생성 PNG는 각각 SHA-256이 동일하다.

| 입력 | 입력 SHA-256 | Print PDF SHA-256 | Native 1쪽 / 전쪽 최저 | WASM 1쪽 / 전쪽 최저 |
| --- | --- | --- | --- | --- |
| `samples/2026_oss_rst.hwp` | `b3a43578fc441d9668d06cd55c0cd267332ecb087e47ada4c5ac456af2afc78b` | `bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e` | [99.93% / 99.92%](native-oss-hwp.tsv) | [100.00% / 100.00%](wasm-oss-hwp.tsv) |
| `samples/hwpx/2026_oss_rst.hwpx` | `90858f0ecf9e95afe46b6dace6a61960b03552236a1204be530ce3d1866a8398` | `bec53a601cc7714a40ca340d26f971d1ab49eeb355682fc9db7b15cd5e04c86e` | [99.93% / 99.92%](native-oss-hwpx.tsv) | [100.00% / 100.00%](wasm-oss-hwpx.tsv) |
| `samples/issue5169_viewtext_changetracking.hwp` | `c003cdb0dd9aac5fda9c82d4f3be20a327522cf2deb680dc25134c7c6c8c3462` | `affa9d0f6325d77317b4d206feec9c3cd5f62f9181a11f39efffc41fbda88f18` | [100.00% / 99.99%](native-issue5169.tsv) | [100.00% / 99.99%](wasm-issue5169.tsv) |

같은 1쪽의 대표 증거:

- oss-hwp: Native [review](native-oss-hwp-review-001.png) · [overlay](native-oss-hwp-overlay-001.png), WASM [review](wasm-oss-hwp-review-001.png) · [overlay](wasm-oss-hwp-overlay-001.png)
- issue5169: Native [review](native-issue5169-review-001.png) · [overlay](native-issue5169-overlay-001.png), WASM [review](wasm-issue5169-review-001.png) · [overlay](wasm-issue5169-overlay-001.png)
