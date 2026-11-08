# 통합 후보의 원 PR별 시각 검증

제품 코드 `d536455b4e`에서 Native Skia와 fresh WASM Canvas를 96 DPI, Print 프로필로 출력하고 각 입력의 독립 한컴 Print PDF와 전체 페이지를 대조했다. HWP3은 한컴 2020 기준의 2022 호환 모드이고 나머지는 2024 호환 모드다. 이후 변경은 회귀 테스트 이동·서식·검토 문서로, 제품 출력 경로는 바뀌지 않는다.

- Native 실행 파일 SHA-256: `d53e4dac28839e01f4c6bf19761c4bbc181ef70a7a38eb65abf6f6c4050b96fc`.
- fresh WASM SHA-256: `3afec6e53b9374a8a6bc6f9ea23bc7c66831f8c496d3aad265120f5dfd38c0f6`.
- 각 `native-*.tsv`·`wasm-*.tsv`는 색상 경계 보정 전 2px 내용 실루엣의 전체 쪽 결과다. 같은 입력·쪽의 `review`와 `overlay`를 함께 보존한다.
- 시스템에 설치된 글꼴을 이용했으며 글꼴 파일 자체는 포함하지 않는다.

| 입력 | 입력 SHA-256 | 독립 Print PDF | PDF SHA-256 |
| --- | --- | --- | --- |
| `samples/issue6551/113424_evaluation_guideline.hwpx` | `21cf24446bcf558c62b715965a62e8d656e9dcd64e8b071b110f571cc3fb81e0` | `pdf/113424_evaluation_guideline-2024.pdf` | `f2db782bb42cabd2b96cd8985415554803ad7d4322904177a9e1bc903115bf6d` |
| `samples/HWP3-password-123456.hwp` | `db743d084efc9e08e839a5b4d978b16b8676434011776e090e4cda43e57304be` | `pdf/issue8304/HWP3-password-123456-print-2020.pdf` | `279effba40fd2fe3bff6682cd6ee272cc9f49ca3fb1616f929871672637a8348` |
| `samples/issue8399/A.hwp` | `98c5ef50de81d704fcfa03b81c99ef89badfafb839a1ea78334a46b76f9e6987` | `pdf/issue8399/A-hwp-2024.pdf` | `45777e3678de4f0558867697c5bf2f71535c4b71bf9f70955be71367421c2e59` |
| `samples/issue8399/A.hwpx` | `fddd90eed7c4f8818e723bb526c50cad8269c7b1a02eddd64d5a0cda2a5773c7` | `pdf/issue8399/A-hwpx-2024.pdf` | `0450bebb11625b1acbcc1ce06ff36a12474a76b3a52b6ac143b10b000226633a` |
| `samples/issue8399/D.hwp` | `08faf214ad31724f027fc222616e8fa58e03ea77cc04771c801a510f32661b91` | `pdf/issue8399/D-hwp-2024.pdf` | `c058a040a80938dc448ea6e9181235ae6cb55c77320d45d459932d82f68ed46f` |
| `samples/issue8399/D.hwpx` | `fb3a7c5425cb7a42438988a267a3416e95bd8884bbb2cd7f21c9acbc7160492c` | `pdf/issue8399/D-hwpx-2024.pdf` | `8f554dea4519655eed59fbff7259c763e367795fef4fe400f9ee209aac098b9c` |
| `samples/issue8399/E.hwp` | `7427d910a308fb1c0a2daf8cdf31fce429a45c194899d7d5a79b264743277057` | `pdf/issue8399/E-hwp-2024.pdf` | `5f94ff87ec244609bae5efef5e741f8d7207648771e8b4c8a6c4c9336f5af136` |
| `samples/issue8399/E.hwpx` | `a4141e83a0dc8ecbd94bec888893be702a87b7fd3b6fefaad451720763eece1c` | `pdf/issue8399/E-hwpx-2024.pdf` | `bf580bc15a7b8f9c50763b4eecb3ac15eeb9ed0f80eb8defbb4772c6249ed643` |
| `samples/issue8399/P.hwp` | `b305d6d267ef5e23a42321e9ddbfd6388cfd576a9381cb49ae9fc6500f928907` | `pdf/issue8399/P-hwp-2024.pdf` | `bf586386e33796adee12082a3173d995e3614cf745973ae5a715a4e12506a889` |
| `samples/issue8399/P.hwpx` | `72016c8b292cd8325e910e165e6a5a0a7b82a05ca6f1f27ad20316a98018464f` | `pdf/issue8399/P-hwpx-2024.pdf` | `5d53dc4942b5639f16b306f5d29be6b1a80cecafd3bfae31391ee194215cffb1` |
