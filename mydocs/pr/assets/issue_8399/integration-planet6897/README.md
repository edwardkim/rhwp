# #8399 조판 입력 대조 — 통합 후보 중간 검증

2026-11-01, 96 DPI, 한컴 2024 Print PDF와 Native Skia·fresh WASM Canvas의
`silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣)을 전쪽 비교했다.
각 문서는 1쪽이며 PDF와 쪽수가 같다. 글꼴은 검증 시스템에 설치된 글꼴을 자동 공급했다.
이는 렌더링 대조 환경이며 회귀 테스트가 해당 글꼴에 의존한다는 뜻은 아니다.

이 결과는 HWP3 번호·글꼴 보정 전 제품 소스 `6be28314c5`의 고정 실행 파일에서 산출했다.
해당 보정 후의 제출 후보 검증을 대신하지 않는다.
Native SHA-256 `4b03859cfa21cdba1a766d5141d260adf70d3c986c6481a3e8440d1ab992ae96`,
WASM SHA-256 `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`.

| 입력 | 입력 SHA-256 | Print PDF | PDF SHA-256 | Native | fresh WASM |
| --- | --- | --- | --- | ---: | ---: |
| `samples/issue8399/A.hwp` | `98c5ef50de81d704fcfa03b81c99ef89badfafb839a1ea78334a46b76f9e6987` | `pdf/issue8399/A-hwp-2024.pdf` | `45777e3678de4f0558867697c5bf2f71535c4b71bf9f70955be71367421c2e59` | [100.00%](native-A-hwp.tsv) | [100.00%](wasm-A-hwp.tsv) |
| `samples/issue8399/A.hwpx` | `fddd90eed7c4f8818e723bb526c50cad8269c7b1a02eddd64d5a0cda2a5773c7` | `pdf/issue8399/A-hwpx-2024.pdf` | `0450bebb11625b1acbcc1ce06ff36a12474a76b3a52b6ac143b10b000226633a` | [100.00%](native-A-hwpx.tsv) | [100.00%](wasm-A-hwpx.tsv) |
| `samples/issue8399/D.hwp` | `08faf214ad31724f027fc222616e8fa58e03ea77cc04771c801a510f32661b91` | `pdf/issue8399/D-hwp-2024.pdf` | `c058a040a80938dc448ea6e9181235ae6cb55c77320d45d459932d82f68ed46f` | [100.00%](native-D-hwp.tsv) | [100.00%](wasm-D-hwp.tsv) |
| `samples/issue8399/D.hwpx` | `fb3a7c5425cb7a42438988a267a3416e95bd8884bbb2cd7f21c9acbc7160492c` | `pdf/issue8399/D-hwpx-2024.pdf` | `8f554dea4519655eed59fbff7259c763e367795fef4fe400f9ee209aac098b9c` | [100.00%](native-D-hwpx.tsv) | [100.00%](wasm-D-hwpx.tsv) |
| `samples/issue8399/E.hwp` | `7427d910a308fb1c0a2daf8cdf31fce429a45c194899d7d5a79b264743277057` | `pdf/issue8399/E-hwp-2024.pdf` | `5f94ff87ec244609bae5efef5e741f8d7207648771e8b4c8a6c4c9336f5af136` | [100.00%](native-E-hwp.tsv) | [100.00%](wasm-E-hwp.tsv) |
| `samples/issue8399/E.hwpx` | `a4141e83a0dc8ecbd94bec888893be702a87b7fd3b6fefaad451720763eece1c` | `pdf/issue8399/E-hwpx-2024.pdf` | `bf580bc15a7b8f9c50763b4eecb3ac15eeb9ed0f80eb8defbb4772c6249ed643` | [100.00%](native-E-hwpx.tsv) | [100.00%](wasm-E-hwpx.tsv) |
| `samples/issue8399/P.hwp` | `b305d6d267ef5e23a42321e9ddbfd6388cfd576a9381cb49ae9fc6500f928907` | `pdf/issue8399/P-hwp-2024.pdf` | `bf586386e33796adee12082a3173d995e3614cf745973ae5a715a4e12506a889` | [99.98%](native-P-hwp.tsv) | [99.98%](wasm-P-hwp.tsv) |
| `samples/issue8399/P.hwpx` | `72016c8b292cd8325e910e165e6a5a0a7b82a05ca6f1f27ad20316a98018464f` | `pdf/issue8399/P-hwpx-2024.pdf` | `5d53dc4942b5639f16b306f5d29be6b1a80cecafd3bfae31391ee194215cffb1` | [99.98%](native-P-hwpx.tsv) | [99.98%](wasm-P-hwpx.tsv) |

각 TSV는 해당 원본과 PDF의 전체 1쪽이다. 원 PR이 지목한 A의 다음 문단, D의 글상자와
잘린 본문 줄, P의 그림과 후속 문단을 두 경로에서 직접 확인했다. P HWPX는 원 PR의
기존 경로 대조군이며 그 PR이 새롭게 해결했다고 주장하지 않는다.

| 대표 입력 | Native | fresh WASM |
| --- | --- | --- |
| A-hwp | [review](native-A-hwp-review.png) · [overlay](native-A-hwp-overlay.png) | [review](wasm-A-hwp-review.png) · [overlay](wasm-A-hwp-overlay.png) |
| D-hwpx | [review](native-D-hwpx-review.png) · [overlay](native-D-hwpx-overlay.png) | [review](wasm-D-hwpx-review.png) · [overlay](wasm-D-hwpx-overlay.png) |
| P-hwpx | [review](native-P-hwpx-review.png) · [overlay](native-P-hwpx-overlay.png) | [review](wasm-P-hwpx-review.png) · [overlay](wasm-P-hwpx-overlay.png) |

검증 명령은 `scripts/visual_sweep.py --silhouette-only --rasterizer png --compat 2024
--dpi 96 --page-workers 1 --rhwp-bin <고정 Native>`에 각 원본과 PDF를 전달했다.
WASM은 같은 명령에 `--wasm-pkg <fresh package>`를 추가했다. 대표 이미지는 같은 입력에
`--silhouette-only` 없이 `--page 1`을 지정해 생성했다.
