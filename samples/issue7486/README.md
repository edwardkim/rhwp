# PR #7487 — 동일 Enter 입력과 한컴 기준 PDF

검증 source: `c741f24135d03470588440d1bb67dff3defd3024`; 관측 최신 base: `4a7cf61c8652586ffe86158344771029714f3681`.
이 파일들은 한컴 저장본이 아니라 rhwp 편집 API로 만든 합성 HWPX다. XML·LineSeg를 수동 수정하지 않았다. 생성은 fresh WASM에서 `createBlankDocument → applyParaFormat(lineSpacing, Percent) → splitParagraph × Enter 횟수 → exportHwpx` 순서로 수행했다. 정식 회귀의 Native 편집 API와 같은 Rust 동작 경로를 사용하며, 아래 **동일 바이트 파일**을 Native/fresh WASM 및 한컴 MCP가 읽었다.

`rhwp info --json`의 `format: hwpx`, `lastSavedWith.product: hancom-office-2020`, `version: 11.0.0.3524`를 확인하여 규정에 따라 npx client 0.9.0의 `2020` 엔진을 명시했다. `start → succeeded → download success`를 확인했다. 서비스가 반환한 engine/profile은 `2020`, 한컴 버전은 `11.0.0.9136`, backend는 `hwp-managed-direct-dll-host`, worker는 32bit다. PDF 출력은 `hancom2020_pdf_driver_one_up`, 원문 전처리는 `none`, font_scope는 verified다. 저장 제품 값은 rhwp 기본 빈 문서 template의 메타데이터이며 한컴에서 이 입력을 생성·저장했다는 증거가 아니다. 이 변환은 데스크톱 한컴에서 직접 조작하거나 2024 엔진을 실행했다는 증거도 아니다. 서비스 주소·인증 값은 기록하지 않는다.

| 입력 | 줄간격 / Enter | 한컴·Native·fresh WASM 쪽수 | 문단 수 / 마지막 owner(0-based) | MCP job |
| --- | --- | --- | --- | --- |
| [pr7487-spacing200-enter33.hwpx](pr7487-spacing200-enter33.hwpx) | 200% / 33 | 2 / 2 / 2 | 34 / p1 | `f4e478b2-65f1-4dff-9238-e7c4cad4f945` |
| [pr7487-spacing300-enter22.hwpx](pr7487-spacing300-enter22.hwpx) | 300% / 22 | 2 / 2 / 2 | 23 / p1 | `51cdccb0-eff2-4081-907e-6914cd8cc9d9` |
| [pr7487-spacing160-enter60.hwpx](pr7487-spacing160-enter60.hwpx) | 160% / 60 | 2 / 2 / 2 | 61 / p1 | `986f458c-2bba-4a21-a5d8-6149de08cf6f` |
| [pr7487-spacing100-enter90.hwpx](pr7487-spacing100-enter90.hwpx) | 100% / 90 | 2 / 2 / 2 | 91 / p1 | `8308f463-e870-4627-a47a-f57642155b22` |
| [pr7487-spacing130-enter90.hwpx](pr7487-spacing130-enter90.hwpx) | 130% / 90 | 2 / 2 / 2 | 91 / p1 | `64b537a2-c365-498a-8f1d-5c4f6530e725` |
| [pr7487-spacing160-enter90.hwpx](pr7487-spacing160-enter90.hwpx) | 160% / 90 | 3 / 3 / 3 | 91 / p2 | `ebee4347-5ba3-47c5-9589-ab2200545517` |
| [pr7487-spacing180-enter90.hwpx](pr7487-spacing180-enter90.hwpx) | 180% / 90 | 3 / 3 / 3 | 91 / p2 | `261da9b9-954d-406f-86eb-ffca98837712` |
| [pr7487-spacing200-enter90.hwpx](pr7487-spacing200-enter90.hwpx) | 200% / 90 | 3 / 3 / 3 | 91 / p2 | `a71eb364-8138-4c3e-8bc0-c77ae904a788` |
| [pr7487-spacing300-enter90.hwpx](pr7487-spacing300-enter90.hwpx) | 300% / 90 | 5 / 5 / 5 | 91 / p4 | `7c7cd374-3661-4668-a014-e652736cc4bd` |

문단은 Native 덤프에서 `0..Enter 횟수` 순서로 정확히 한 번씩 나타났고, 같은 byte의 fresh WASM 재열기 owner와 일치했다. 200% Enter33의 문단 33과 300% Enter22의 문단 22는 새 쪽(index 1)을 소유한다. 이 owner는 rhwp의 의미 검사이며 한컴 PDF가 빈 문단의 캐럿 좌표를 제공한다는 뜻은 아니다.

Native/fresh WASM 인쇄 프로필의 전체 24쪽씩을 비교했다. [전쪽 TSV](../../mydocs/pr/assets/pr_7487/hancom-enter-silhouette.tsv)의 최저 2px 관용 실루엣 일치율은 각각 100%, 90% 미만·누락 0쪽이다. 다만 모든 페이지가 빈 출력(`content_union_pixels=0`)이므로 **100%는 흰 페이지 비교값**이다. 이 값만으로 빈 줄의 좌표·캐럿·Studio 스크롤의 정확성을 입증하지 않는다. 독립 한컴 PDF의 전체 쪽수와 의미 소유 검사, 대표 compare/overlay 직접 판독을 함께 사용한다.

| 입력 | HWPX SHA-256 | 기준 PDF / SHA-256 |
| --- | --- | --- |
| pr7487-spacing200-enter33.hwpx | `c19c69528d77c06aa81fb706f7189940cd4b26f86a73cc95863ba2069eddc241` | [pr7487-spacing200-enter33-2020.pdf](../../pdf/pr7487-spacing200-enter33-2020.pdf) / `184b1168457aab9a21bc7ef3240346bceaa4d1f5188ce2974f835e9cc1bcb528` |
| pr7487-spacing300-enter22.hwpx | `187d7defb216b6cef021b38be08a7030cd988dfdb79593e6d09567dea41da8da` | [pr7487-spacing300-enter22-2020.pdf](../../pdf/pr7487-spacing300-enter22-2020.pdf) / `a357da66ac9ff12cdb5570394af218b8fed8c81cb1039b9108c81323c811b93f` |
| pr7487-spacing160-enter60.hwpx | `7bae5da2da29a64d18d9101a5d9ffe25b1782c33186316123b38719529cdb4f8` | [pr7487-spacing160-enter60-2020.pdf](../../pdf/pr7487-spacing160-enter60-2020.pdf) / `59b8654fdf68335ccfa8f69437208566b32359946467266de0e065b48cb22b5f` |
| pr7487-spacing100-enter90.hwpx | `c82a23d3cada522dd49d1448fac84432fdec033bcc9a814e8611109179f8877a` | [pr7487-spacing100-enter90-2020.pdf](../../pdf/pr7487-spacing100-enter90-2020.pdf) / `61a467c5e98edf358c282c8fc48e8fde36f677894cf4ebfa1c801f224abc0085` |
| pr7487-spacing130-enter90.hwpx | `60cbc3be3a8480af47c08d1ea0ccf0537953bd1dbc448a57a1c823e1e8fd99a8` | [pr7487-spacing130-enter90-2020.pdf](../../pdf/pr7487-spacing130-enter90-2020.pdf) / `773d463e032e661c131449540ade4854b941ba599e561234ef192cf3839ddbf6` |
| pr7487-spacing160-enter90.hwpx | `1df0719bd0b97915761865d7e0bbc16ca724c767eb6f09aa75a42600ddeccc89` | [pr7487-spacing160-enter90-2020.pdf](../../pdf/pr7487-spacing160-enter90-2020.pdf) / `966b5612d930ce46dbbf79d8c7c86f40adca11a39b12b2662eaa28f5f9931138` |
| pr7487-spacing180-enter90.hwpx | `92cf4229b3dfa6dc2ee35a098b4b3d18b1c659d84012fee6badf53c89b688b53` | [pr7487-spacing180-enter90-2020.pdf](../../pdf/pr7487-spacing180-enter90-2020.pdf) / `a6aca7dd85e060e7e9f9c1010e6aad6ef7845d7ecc09343d466f1d4c9649dec5` |
| pr7487-spacing200-enter90.hwpx | `651fbb0a708aca8e1641307072ffd8bb246943c8cd0c409c8f9ae5946f58af71` | [pr7487-spacing200-enter90-2020.pdf](../../pdf/pr7487-spacing200-enter90-2020.pdf) / `4a9e620b72b111647414bf20559921452bb5d04a497dea201d156fefbf4de02a` |
| pr7487-spacing300-enter90.hwpx | `611aa65699c75a8024de6203b632e78132bfceea339beff7ea9676fc497469b9` | [pr7487-spacing300-enter90-2020.pdf](../../pdf/pr7487-spacing300-enter90-2020.pdf) / `e5b3f953bd82bcdf8db8b9570fe8d3e8ec08c9cb4d257dccd852d8167dbb628d` |

모든 PDF는 50 MiB 미만이고 client/server/local 바이트 수·SHA-256이 일치했다. 모든 PDF의 `pdfinfo`에서 PDF 1.4, Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, A4 595×841 pt, 암호화 없음·JavaScript 없음을 확인했다. 변환 profile·실제 한컴 버전과 PDF Creator의 연도·빌드는 구분한다.

## 재검증

저장소 루트에서 검토 source의 Native CLI와 fresh WASM을 먼저 빌드한다.

```sh
CARGO_TARGET_DIR=target/pr-review cargo build --locked --profile release-test --bin rhwp
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
```

위 WASM은 host 진단용 `--no-opt` package이며 최적화된 배포 산출물이라고 보고하지 않는다. 당시 JS SHA-256은 `2b7e7bb01cbbff0cb0d3c9a3222c6cb187f9d9710045013bcfb077d7abef4133`, WASM SHA-256은 `3c4854430292e17c4b9943203dacfa59865eef60bb6b9207b4f172f11861d855`이며 root pkg와 Studio public이 일치했다.

예를 들어 경계 입력은 다음과 같이 같은 PDF로 두 backend를 비교한다.

```sh
python3 scripts/visual_sweep.py --silhouette-only \
  --file-target spacing200-enter33 samples/issue7486/pr7487-spacing200-enter33.hwpx pdf/pr7487-spacing200-enter33-2020.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp --dpi 96 --out output/pr7487-native
python3 scripts/visual_sweep.py --silhouette-only \
  --file-target spacing200-enter33 samples/issue7486/pr7487-spacing200-enter33.hwpx pdf/pr7487-spacing200-enter33-2020.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp --wasm-pkg pkg --dpi 96 --out output/pr7487-wasm
```

대표 compare·standalone overlay·review PNG는 `mydocs/pr/assets/pr_7487/hancom-*-p002.png`다. 최신 p122 p2 대조군도 별도 재출력해 Native/fresh WASM 모두 관용 실루엣 100%, 엄격 잉크 보조값 99.50139%를 확인했다. 그림의 원점·외곽·crop은 직접 비교했으며 모서리 resampling 잔차는 남았다. p122 결과는 Enter 입력의 한컴 대조를 대신하지 않는다.

작업지시자의 시각·부분 해결 범위 확인과 이 입력/기준 파일을 포함한 새 head의 CI는 별도 제출 조건이다. Studio 캐럿·스크롤 및 표 뒤 Enter는 해결 범위에 포함하지 않는다.
