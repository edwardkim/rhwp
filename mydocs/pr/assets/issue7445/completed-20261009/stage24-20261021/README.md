# 새 개체 캡션의 번호·물리 줄 검증

[구현·원 PR·검사 판정](../../../../../working/task_m100_7445_rebase_repair_stage24.md).
[입력10개·독립 Print SHA와 job](../../../../../../tests/fixtures/issue1470_authored_caption_numbering/README.md).
원본은 기존 Hancom2020 blank에서 공개 API로 생성했다. 모든 조건은1쪽이며,
Native/fresh WASM 각각10/10쪽, canonical96DPI/compat2022/PDFium smooth_rgb_v1/raw2px 최저100.00000%다.
미달·누락·중복0이다. raw100%는 엄격RGB 일치를 뜻하지 않는다.

| 조건 | Native TSV | fresh WASM TSV | Native / WASM 최저 |
|---|---|---|---:|
| `tables-all-hwpx` | [TSV](native/tables-all-hwpx.tsv) | [TSV](wasm/tables-all-hwpx.tsv) | 100.00000 / 100.00000% |
| `tables-all-hwp` | [TSV](native/tables-all-hwp.tsv) | [TSV](wasm/tables-all-hwp.tsv) | 100.00000 / 100.00000% |
| `tables-caption-removed-hwpx` | [TSV](native/tables-caption-removed-hwpx.tsv) | [TSV](wasm/tables-caption-removed-hwpx.tsv) | 100.00000 / 100.00000% |
| `tables-caption-removed-hwp` | [TSV](native/tables-caption-removed-hwp.tsv) | [TSV](wasm/tables-caption-removed-hwp.tsv) | 100.00000 / 100.00000% |
| `pictures-all-hwpx` | [TSV](native/pictures-all-hwpx.tsv) | [TSV](wasm/pictures-all-hwpx.tsv) | 100.00000 / 100.00000% |
| `pictures-all-hwp` | [TSV](native/pictures-all-hwp.tsv) | [TSV](wasm/pictures-all-hwp.tsv) | 100.00000 / 100.00000% |
| `pictures-caption-removed-hwpx` | [TSV](native/pictures-caption-removed-hwpx.tsv) | [TSV](wasm/pictures-caption-removed-hwpx.tsv) | 100.00000 / 100.00000% |
| `pictures-caption-removed-hwp` | [TSV](native/pictures-caption-removed-hwp.tsv) | [TSV](wasm/pictures-caption-removed-hwp.tsv) | 100.00000 / 100.00000% |
| `tables-caption-left-hwpx` | [TSV](native/tables-caption-left-hwpx.tsv) | [TSV](wasm/tables-caption-left-hwpx.tsv) | 100.00000 / 100.00000% |
| `tables-caption-left-hwp` | [TSV](native/tables-caption-left-hwp.tsv) | [TSV](wasm/tables-caption-left-hwp.tsv) | 100.00000 / 100.00000% |

## 대표 직접 판독

- Native 전체 캡션: [review](native-all-review.png) / [overlay](native-all-overlay.png)
- Native 저장 후 왼쪽 캡션: [review](native-left-review.png) / [overlay](native-left-overlay.png)
- fresh WASM 저장 후 왼쪽 캡션: [review](wasm-left-review.png) / [overlay](wasm-left-overlay.png)
- fresh WASM 중간 그림 캡션만 삭제: [review](wasm-picture-removed-review.png) / [overlay](wasm-picture-removed-overlay.png)
- 이전 후보의 저장 후3.8CSSpx 상향 오류: [review](before-native-left-review.png) / [overlay](before-native-left-overlay.png)

세 표 소유와 전체1·2·3/삭제후1·3, 마지막 표 왼쪽 캡션, 앞뒤 표 간격을 읽었다.
그림3개는 투명1px 대조군이며 그림 자체 모양 수용으로 해석하지 않는다.
Native와PDF의 가는 괘선 진하기는 다르다. 대응한 PDF0.36pt/SVG0.48CSSpx 및 검정색은 같으며,
위치 회귀가 고쳐졌다는 판정과 엄격 색상·래스터 일치를 구분한다.

## 실행 신원

- 부모 HEAD: `e8145da134f64bd5a04d864531dc9aca1318aa99`; runtime 변경6파일은 아래 SHA로 고정했다.
- Native CLI SHA256: `085d58af5cb821a3ca4c6699fefac28bbe102193036270d138dd78c9a9bf74f9`
- Native library SHA256: `79a98c6cc6da4c9f67a29dfae24f09e4f05aa263b97256fe8e07ada67b3e4ae6`
- fresh JS SHA256: `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`
- fresh WASM SHA256: `b34e8613fdfda5106accee99c385833be5fbaf7e41e3925d121840e7056b5b0c`
- Mac locked wrapper `--target web --out-dir pkg --no-opt`, 로컬 대체 빌드이며 Docker 최적화가 아니다.

| runtime source | SHA256 |
|---|---|
| `src/document_core/commands/object_ops/picture.rs` | `511fc37c182a32177287c8ea6c3b43c417857cd35f31bb1eb35d21fe1f4d9fd5` |
| `src/document_core/commands/object_ops/table.rs` | `e4417374bcb1ae945872cf603e5d79154873af61ee44a320aa07130ab71d203e` |
| `src/document_core/commands/table_ops.rs` | `48bb806f7352c981af57c81b72f642b18e4869ecd32e4e720a0505877ef8ca77` |
| `src/renderer/composer/line_breaking.rs` | `de8d2b7d8344f3c160eee168df89fe388745180d44e80f2e81b6669085a92148` |
| `src/renderer/typeset.rs` | `fab1be46cf4ed64550866caf522eeb8773d10fc049b63c2be8b541ca29967dea` |
| `src/renderer/typeset/controls/stored_tac/owned_row.rs` | `13ea89515641c506f90815f1164d75261d558ac27bae67aff5f3d640cb4e95f0` |

임시 producer/run manifest/변환 응답/로그는 output의repair908–919에만 보존한다.
이20쪽 결과는 브랜치 전체52조건·전체nextest·실제Studio/CanvasKit 수용을 대체하지 않는다.
