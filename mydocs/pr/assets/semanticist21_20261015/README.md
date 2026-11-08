# semanticist21 통합 검토 — 시각 증거와 페이지별 회귀

## 범위와 판정

기준은 `upstream/devel`의 `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`다. #7663·#7670·#7671·#7673 관련 입력 및 명시적 반례 **29문서42쪽은 Native/fresh WASM 모두 전쪽100%**다. 각 경로의15개 확대 시트(각44영역)를 실제 열어 탭, 번호, 각주·미주 참조, 그림 뒤 소속, 줄바꿈, 양쪽 정렬, 단 순서와 선 끝을 Print와 직접 대조했다. 수식 변경은 없다.

추가 HWP3 대조군2문서2쪽을 포함하면31문서44쪽이며 전쪽90% 이상, base 대비0.01%p 이상 하락0이다. HWP3 표 날짜행은 기준부터 `일( )`가 셀 밖에 잘린다. 해당 페이지는 완료로 승인하거나 신규 px 회귀로 고정하지 않았다. 현재 source PR의 HWP3 parser 보정 또는 그 날짜행 개선을 주장하지 않는다. 기존 정상 회귀는 유지한다. 다른 HWP3 대조군의 원형 기호 모양/간격 차이도 완전 일치로 보고하지 않는다.

숫자 gate와 직접 판독을 구분한다. 이 문서는 전체 Rust/lint/Skia 및 통합 PR CI의 성공을 대신하지 않는다. 최종 실행 결과는 원 PR별 archive review에서 연결한다.

## 생산물과 공급 조건

검증 code는 `20d93fe1d4226acab640b66875964d09c7d6d59b`다. 원본 저장 캐시 보존 보정 후 Native와 fresh WASM 전쪽 PNG88개를 재생성했고 이전 직접 판독 캡처와 모두 픽셀 단위로 동일했다. 기존 전체 확대 판독에 더해 새 대표 시트 및 실제 Studio backend5개 시트를 직접 대조했다. 과거 캡처를 새 producer 출력으로 대체하지 않았다.

Native는 명시적 `cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir target/pr-review` 뒤 동결한 CLI의 `export-png`다. WASM은 저장소 루트의 `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`로 새로 빌드했다. Docker가 없는 호스트의 문서화된 fallback이며 최적화 Docker 검증으로 표기하지 않는다. Cargo jobs8, WASM release codegen-units16, 고정 공용 target/pr-review를 사용했다.

WASM PNG는 `scripts/export-wasm-png-for-sweep.mjs`의 실제 Print Canvas2D 출력이다. SVG 파일을 생성해 PNG로 바꾼 결과로 표시하지 않는다. 기준/후보 모두96dpi·compat2022·print profile·전체 글꼴 공급을 적용했다. 시스템에 설치된 원본 폰트 파일을 참조하는 symlink만 로컬 검증에 사용했다. 폰트 바이너리 복사·family 위장·PR 배포는 하지 않는다.

- Native CLI SHA-256: `37d6d09029cc64a465e16c78af6bc64916af581ae30a5f3fabf9237c91db78a0`.
- fresh WASM `rhwp.js` SHA-256: `f377c6be818c64ffd20752e29271000ff5053ea8c7c984f65abb8fe74f17eddd`; Studio public JS/WASM과 일치 확인.
- fresh WASM `rhwp_bg.wasm` SHA-256: `4907d462d9bace404d5f0b11a3d8b7d1115612ed5bde92db4cbd018f0b938e62`; Studio public JS/WASM과 일치 확인.

## 입력 생성과 독립 기준

PR 테스트가 생성하는 명시적 계약 입력을 보존했다. #7663은 본문·머리말·셀의 탭과 뒤 run, #7670은150pt 탭, #7671은 구역 시작 및 각주·미주·그림 바로 뒤 번호, #7673은 새 쪽의 단 방향·구분선·비례 너비를 검증한다. HWPX와 HWP 저장본은 별개 입력이며 같은 문서라며 점수를 합산하지 않는다.

#7671의 `*-public.hwp`는 Studio와 같은 공개 `HwpDocument.exportHwp()` adapter snapshot으로 저장했다. 내부 raw HWP4개가 Print timeout인 경로를 정상 출력으로 대체 보고하지 않았다. 내부 출력 진단은 ignored output에 남고 활성 입력에서는 제외했다.

추가7개 본문 필드 HWPX는 opening 계약 입력의 XML에 PAGE/TotalPage·쪽 나눔·좁은 너비·크기20pt·부분 문단·양쪽 정렬 반례를 명시적으로 편집하고, 수동 편집으로 무효인 줄 캐시를 제거한 합성 입력이다. 정상 한컴 생성 원본으로 주장하지 않는다. 각 실제 입력을 독립 한컴 Print로 출력해 필드 값·한 em 줄 수용·숫자의 자연 전진과 후속 본문 위치의 기대값을 정했다.

#7673 width의8000/768/24000 비례 정의는 독립 한컴 HWP→HWPX SaveAs에서도 같은 값으로 관찰됐다. 본문 폭 합계에 맞춰 강제로 정규화한 임시 대조군은 가설이 반증돼 폐기했으며 활성 fixture로 추가하지 않았다.

기준 PDF는 실제 Hancom Office2020 MCP managed direct DLL host, PrintMethod0 / one_up 출력이다. PDF Creator/Producer 표기만으로 생성 경로를 추정하지 않고 job 결과·입력/PDF SHA·쪽수로 확인했다. 아래 PDF는 모두50MiB 미만이며 같은 검증 입력과 함께 보존한다.

| 입력 | SHA-256 | 독립 Print PDF | PDF SHA-256 | bytes / 쪽 |
| --- | --- | --- | --- | --- |
| [tests/fixtures/semanticist21_20261015/pr7663-body.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7663-body.hwp) | `d92446c5b20ee3de36617e8ec56b0647801a7bf5844f41a0e114e7ae1a860fa2` | [pr7663-body-hwp-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-body-hwp-v2-2020.pdf) | `66fd84c14e6d3cf00b002f5f25070b063039d775df37a279bc65ec812b0468bc` | 9968 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7663-body.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7663-body.hwpx) | `f97eeb5db36bd9c870482a626e3880ca6927770f32c9953b630e6ce4ff60d747` | [pr7663-body-hwpx-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-body-hwpx-v2-2020.pdf) | `f4c6bce1b6dc2b44e613ee7d00ca16bf073a5c0d434ede64a11a41ada9847056` | 9968 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7663-cell.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7663-cell.hwp) | `64335a100629b3a8ccbee6f6f7da9376801e94e11053a006daad2058cd6e5a95` | [pr7663-cell-hwp-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-cell-hwp-v2-2020.pdf) | `8a0f91d859a7261de6a04e841b1a1213f3f51227f78398656e037e1a5ed4b4f2` | 10100 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7663-cell.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7663-cell.hwpx) | `831b4249f3185cd471c7f43b52170faa57b1b59db6ffa10a265f02961b28c696` | [pr7663-cell-hwpx-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-cell-hwpx-v2-2020.pdf) | `95688a60f2a1f6fbbaf45d6eb0e078b59f8782be3830fd25fe01c91e7158b2a8` | 10100 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7663-header.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7663-header.hwp) | `5720e0f3eddaebe90d22c40e187185bb35caac53042679a430e1010af3af6029` | [pr7663-header-hwp-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-header-hwp-v2-2020.pdf) | `cd1ab4fa5c90933ff28d0441feacc40d98e9cdddb4368354e7b854458f88ba0d` | 9968 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7663-header.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7663-header.hwpx) | `a1edcc23ce17b0c5e0db0907b8093087595d805cc41fde82bdf1001198e7fced` | [pr7663-header-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7663-header-hwpx-2020.pdf) | `7501da62be61d2ddc1faa2a7d428cfda99c61dad90e9f9d9e080e720a1df8294` | 9968 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7670-tab-150pt.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7670-tab-150pt.hwp) | `b10737f4e477539e387ae78249805f5aeb4a2fa9c7cb21cc24877b2e87efde43` | [pr7670-tab-150pt-hwp-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7670-tab-150pt-hwp-v2-2020.pdf) | `44b70a222d307da966fd88167b96e366895ef3b6645f978040bc44ecee8aa2e5` | 9003 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7670-tab-150pt.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7670-tab-150pt.hwpx) | `2c9c3d77caa202e90a0c808c449f1134764a79bdc63862ca15fa43da8eb645c1` | [pr7670-tab-150pt-hwpx-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7670-tab-150pt-hwpx-v2-2020.pdf) | `e9b35e229bf6b2fc53c97f290a749c68d8a59600568887a9ab119ee844a8854b` | 9003 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-endnote-public.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7671-endnote-public.hwp) | `2f2dc314e752d1e0ca73164a1f098c7325e9c1bb308d85e9a6d0b12a6e9d54b8` | [pr7671-endnote-public-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-endnote-public-hwp-2020.pdf) | `5bb656f4f1f7639cc8094bbfaa405a56a854d60fd8f12678e8b83a165e6f67c2` | 10936 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-endnote.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-endnote.hwpx) | `4c9b79cd7658cd04c1ad5293c5ca7c4110d75ec8de618ada58e46d93ab573b29` | [pr7671-endnote-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-endnote-hwpx-2020.pdf) | `751b6e07db4d1de61c2fe5dadfe8cf56a679a850a87c79e1255f1ab378d0875c` | 10936 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-footnote-public.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7671-footnote-public.hwp) | `b56f2828f60d993cf5de355358edee4310e8a577376c5216ca11b2ff15adf757` | [pr7671-footnote-public-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-footnote-public-hwp-2020.pdf) | `1fd6c792a7b87ec62e01881a22ec1e46d2ebfa0e23bcfbd477c30b151261c023` | 10931 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-footnote.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-footnote.hwpx) | `71275fc13063d7fddb0f8f79e90dc18e8563160707ead57ec615d35f297154cf` | [pr7671-footnote-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-footnote-hwpx-2020.pdf) | `4bf090776522902167e1f58c0bdf09826ae6b91b639157bdcd1030d55ad3a040` | 10931 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-opening-public.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7671-opening-public.hwp) | `f1c9d30c4d45c110fd71f4980fc810d09bae6eb3d1c83400028365c429cf6d83` | [pr7671-opening-public-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-opening-public-hwp-2020.pdf) | `9d6f2801fd3b0c7c28574220d7ec86b9dc932df66f8590a331596287abdf7e24` | 10750 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-opening.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-opening.hwpx) | `76f09a943faed0ea241403d635d001b451902c6012b5f0fd905e03aa204245c5` | [pr7671-opening-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-opening-hwpx-2020.pdf) | `45bed856d39f9c727177c7f86b27332dfce5edfa80b68478d00eb1b8b32d0044` | 10744 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-picture-public.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7671-picture-public.hwp) | `bebcf898edad72222192ebca2f8a3913fe7fafde1ded34e90cb11dd9811bbe76` | [pr7671-picture-public-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-picture-public-hwp-2020.pdf) | `5f57b10d5afac95e69aca945c2daddc1e958c351d30220e47a1ef6af11353933` | 12458 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-picture.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-picture.hwpx) | `e5b8ac01bd3bc55f511f4114cf3922c8a82d97851bd5268d38551a1bb40a3ac9` | [pr7671-picture-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-picture-hwpx-2020.pdf) | `89bd2de6541db4b446448ee5d9906b10f745be1a0275be2c76c13befa09d96b2` | 12458 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7673-line.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7673-line.hwpx) | `b5be7adfc96f1daa4428006e5e80a9c478a5b346b06bc8da964d1a122c701c7a` | [pr7673-line-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-line-hwpx-2020.pdf) | `243a0b16886300a9b8e539aae5955cff6be9b568eb9e21a3b8b83ea5e5fd73a0` | 11335 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7673-mirror.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7673-mirror.hwpx) | `3b6784bd0f819a4e99bc0d233bacbf30d377565fea93289674a738cc6957e680` | [pr7673-mirror-hwpx-v2-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-mirror-hwpx-v2-2020.pdf) | `4a86f872af4e2308c463b6f882fd0fad5e72fc58e4b2c855bceffa5732809de8` | 11286 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7673-right.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7673-right.hwpx) | `c40ca7b433dcecfe63c6c9f9adaffb11742a20b5ba39539b1fa1f977718b1b8d` | [pr7673-right-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-right-hwpx-2020.pdf) | `3d5fd8d368e13fcc2587456421a2fce155f346cc0d156f4cd18a60ce00d12381` | 11286 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7673-width-hancom.hwp](../../../../tests/fixtures/semanticist21_20261015/pr7673-width-hancom.hwp) | `9f44f0f2b62db9a4b350802cae61c3efa25d92681cea90db912438817c52b841` | [pr7673-width-hancom-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-width-hancom-hwp-2020.pdf) | `9a2aabd4f6b3c5b35d355d9f7a46f485302f3fb91c6662b63d40ba01490fdc51` | 11288 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7673-width-hancom.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7673-width-hancom.hwpx) | `cee40329e513424420c78d9f869b4b727b819f7820b88f6a7b4dd76b8c1f48d4` | [pr7673-width-hancom-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-width-hancom-hwpx-2020.pdf) | `ade6d98498b7075e94eada479ffc8d55efca1ed6fe9f8e0df16bf2c7b86b8586` | 11310 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7673-width.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7673-width.hwpx) | `8f88f7e34cd205053e2d264901d0bba84d2149399b5848c4705bd13186a37a1f` | [pr7673-width-hwpx-2020.pdf](../../../../pdf/semanticist21-20261015/pr7673-width-hwpx-2020.pdf) | `a3ac12ec94f6d2007da5b8dbadb64bdda43ad6619493e0bc82e40091f317ba6e` | 11288 / 2 |
| [samples/hwp3-table-cell-order.hwp](../../../../samples/hwp3-table-cell-order.hwp) | `d9e30ed3a46e20b358c89eb1b26e14cf031f78e5192cea61752fed77f8dd939d` | [hwp3-table-cell-order-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/hwp3-table-cell-order-hwp-2020.pdf) | `a824573e75e75f19dfc5f4a2ab35df50814b2fbd2189a987741153e88681f063` | 57130 / 1 |
| [samples/issue1892_hwp3_tab_roundtrip.hwp](../../../../samples/issue1892_hwp3_tab_roundtrip.hwp) | `7c7457de8fb74f23d2305bb4870e412138494fb0aef1af2258af181ac5aec8c0` | [issue1892_hwp3_tab_roundtrip-hwp-2020.pdf](../../../../pdf/semanticist21-20261015/issue1892_hwp3_tab_roundtrip-hwp-2020.pdf) | `8e45ffbd346557202e79f5e6ad4343a62e570bc57b91aa35b15cf36b4cc4d18d` | 54566 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-fields.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-fields.hwpx) | `e8e57bf9cfb1d5bb7aae53ded67b39aa442ff2cec4ffc528b66f7d54910c2559` | [pr7671-body-page-fields-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-fields-2020.pdf) | `645a7d16abc9cda32293001e5f15f09997d14aceabf9d7488397046b03896dbd` | 12004 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-fields-narrow-restart12.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-fields-narrow-restart12.hwpx) | `74f4c1d7d0ca5bfc4c62809a4e5a8a66cb017e28eff91ed04559d5a845caf695` | [pr7671-body-page-fields-narrow-restart12-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-fields-narrow-restart12-2020.pdf) | `0e077c41e0c8d20c2232ec8d32beacc19a140ebc6e7a5c14188daacfadd516e9` | 12047 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-field12-one-cell.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-field12-one-cell.hwpx) | `a3f11bfcf6eca37a7e3cf6004bb36c33f5c13d8d853d48a80e43f4e3cf086319` | [pr7671-body-page-field12-one-cell-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-field12-one-cell-2020.pdf) | `c019afe4873a2ec60535f8201a832c222b9e05506c6e98066873b833cf03c243` | 9802 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-field-size20.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-field-size20.hwpx) | `8f540ab14bc2193e34c0ba775db1126e537bc34c4c77a0938083d9a8a4e98a1b` | [pr7671-body-page-field-size20-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-field-size20-2020.pdf) | `694e9176f074ae80e7e978f0fcc2c00f5d91d6d35f47c0d0864ea87cdcb2ea62` | 9598 / 1 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-total-page-mixed.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-total-page-mixed.hwpx) | `e5245f5aa1db970ddb250173bd95ecafeca63abaeafb28515f20f43c8c3fc956` | [pr7671-body-total-page-mixed-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-total-page-mixed-2020.pdf) | `2759f4677c6485899bed21f7f87e8196b740b8ed14f938c2c6bed1183389e1d7` | 12004 / 2 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-fields-partial-paragraph.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-fields-partial-paragraph.hwpx) | `58b67ac0c458af7458a117f8b72d16281c6f2cd5daa3f2fd0caf15990929bbd9` | [pr7671-body-page-fields-partial-paragraph-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-fields-partial-paragraph-2020.pdf) | `be1437d2039a3781a6355613d4453b530033a84c381ebba907568dc7a7b8acec` | 11632 / 3 |
| [tests/fixtures/semanticist21_20261015/pr7671-body-page-field-justification.hwpx](../../../../tests/fixtures/semanticist21_20261015/pr7671-body-page-field-justification.hwpx) | `4c5b69baf323254cfdfdf998685702e2dbe14088d5a953c744f4a95c266975bd` | [pr7671-body-page-field-justification-2020.pdf](../../../../pdf/semanticist21-20261015/pr7671-body-page-field-justification-2020.pdf) | `60ca6b1819c1cc57e71b3ca03776c73a2797e9a3d67c85ac18c544e9bc2ae30e` | 11383 / 3 |

## 전쪽 base/head 비교

2px 이웃 관용 내용 실루엣 값이다. 평균으로 미달 쪽을 상쇄하지 않았다. 원본 TSV는 ignored `output/pr-review/semanticist21-20261015/`에 두고 Git에 추가하지 않는다. PR 제출시 본문에 전쪽 TSV 원문을 함께 기록한다. 표의 `base N/W → head N/W`는 같은 입력·PDF·페이지·폰트 조건이다.

| 문서 | 쪽 | base Native / WASM | head Native / WASM | 확대 판독·px |
| --- | --- | --- | --- | --- |
| pr7663-body-hwp | 1 | 60.36745 / 60.20408 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7663-body-hwpx | 1 | 60.36745 / 60.20408 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7663-cell-hwp | 1 | 94.54823 / 94.57014 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7663-cell-hwpx | 1 | 94.54823 / 94.57014 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7663-header-hwp | 1 | 60.36745 / 60.20408 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7663-header-hwpx | 1 | 60.36745 / 60.20408 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7670-tab-150pt-hwp | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7670-tab-150pt-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-endnote-public-hwp | 1 | 98.68657 / 98.61529 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-endnote-hwpx | 1 | 98.68657 / 98.61529 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-footnote-public-hwp | 1 | 97.62419 / 97.47253 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-footnote-hwpx | 1 | 97.62419 / 97.47253 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-opening-public-hwp | 1 | 95.25773 / 94.93392 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-opening-hwpx | 1 | 95.25773 / 94.93392 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-picture-public-hwp | 1 | 96.38752 / 96.19835 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-picture-hwpx | 1 | 96.38752 / 96.19835 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-line-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-line-hwpx | 2 | 97.67055 / 97.56098 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-mirror-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-mirror-hwpx | 2 | 98.82943 / 99.14237 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-right-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-right-hwpx | 2 | 98.82943 / 99.14237 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hancom-hwp | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hancom-hwp | 2 | 42.48555 / 42.30194 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hancom-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hancom-hwpx | 2 | 42.48555 / 42.30194 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hwpx | 1 | 100.00000 / 100.00000 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7673-width-hwpx | 2 | 42.48555 / 42.30194 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| hwp3-table-cell-order-hwp | 1 | 98.99194 / 99.09383 | 98.99194 / 99.09383 | 보조 대조군 / px 자격 미부여 |
| issue1892_hwp3_tab_roundtrip-hwp | 1 | 83.00495 / 84.04071 | 94.05369 / 94.97435 | 보조 대조군 / px 자격 미부여 |
| pr7671-body-page-fields-hwpx | 1 | 91.42857 / 91.29594 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-hwpx | 2 | 88.31169 / 87.55760 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-narrow-restart12-hwpx | 1 | 75.09579 / 74.72325 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-narrow-restart12-hwpx | 2 | 58.75486 / 61.08949 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-field12-one-cell-hwpx | 1 | 70.07576 / 68.27309 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-field-size20-hwpx | 1 | 36.94084 / 37.87879 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-total-page-mixed-hwpx | 1 | 91.18199 / 91.04762 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-total-page-mixed-hwpx | 2 | 88.31169 / 87.55760 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-partial-paragraph-hwpx | 1 | 79.82063 / 80.08850 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-partial-paragraph-hwpx | 2 | 89.95984 / 92.24806 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-fields-partial-paragraph-hwpx | 3 | 83.13253 / 83.03030 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-field-justification-hwpx | 1 | 79.14230 / 77.98507 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-field-justification-hwpx | 2 | 88.16794 / 87.22222 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |
| pr7671-body-page-field-justification-hwpx | 3 | 80.14981 / 78.85305 | 100.00000 / 100.00000 | 직접 대조 / ±2px+관계 |

## 문서별 중요 위치 회귀

[`semanticist21_qualified_print_positions.rs`](../../../../tests/cases/semanticist21_qualified_print_positions.rs)에 문서별29개 테스트를 둔다. 탭8개, 단6개, 자동 번호15개 입력이다. source model/최종 run의 문단·문자 위치·셀/본문 소속, 내용 순서·누락·중복·겹침을 유지하면서 독립 PDF의96dpi 문자 원점/기준선·단 시작·구분선 끝을±2px로 검사한다. 자동 번호는26개 필드/바로 뒤 글자 쌍과 양쪽 정렬 줄 끝2곳을 보호한다.

페이지 선정은 Native/WASM 양쪽의 최솟값100%를 우선했다. 이번29문서는 전쪽100%이므로 중요한 줄·단·각주/그림 경계가 있는 페이지를 각각 포함했다. 고정 쪽은 이후에도 양쪽98% 이상을 유지해야 한다.

첫 px 실행은 구분선 끝에8px 후행 간격이 붙는 실제 결함을 검출했다. 수정 후15/15 통과했다. 추가 필드 px 첫 실행27/29는 각주 영역의 별도 번호까지 본문 수량으로 센 테스트 오류2건이었다. 소유 흐름을 분리한 후29/29 통과했다. 정상 각주 출력을 없애거나 기대 좌표를 완화하지 않았다. 본문 필드 owner의 실제 모델 위치2→1 결함도 별도 정식 검사에서 수정 전 실패/수정 후 통과를 확인했다.

## 대표 실제 PNG

아래 review/overlay는 공식 `scripts.visual_sweep` 합성 함수를 사용했고, 재렌더하지 않고 위 TSV와 같은 PNG를 입력으로 사용했다. 확대 시트는 Native/WASM 각각 전29문서42쪽 및 각주 하단2영역을 포함한다.

- [Native 확대 시트](zoom/native/01.png), [WASM 확대 시트](zoom/wasm/01.png); 각 폴더의01~15가 전체 순서다.
- [탭 Native review](native/pr7663-cell-hwpx/review.png), [WASM review](wasm/pr7663-cell-hwpx/review.png).
- [번호/그림 Native review](native/pr7671-picture-hwpx/review.png), [WASM review](wasm/pr7671-picture-hwpx/review.png).
- [PAGE12 양쪽 정렬 Native review](native/pr7671-body-page-field-justification-hwpx/review.png), [WASM review](wasm/pr7671-body-page-field-justification-hwpx/review.png).
- [단 구분선 Native overlay](native/pr7673-line-hwpx/overlay.png), [WASM overlay](wasm/pr7673-line-hwpx/overlay.png).

## 재현 명령

```bash
CARGO_BUILD_JOBS=8 cargo build --locked --profile release-test --features native-skia --bin rhwp --target-dir target/pr-review
CARGO_BUILD_JOBS=8 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_TARGET_DIR="$PWD/target/pr-review" scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp tests/fixtures/semanticist21_20261015/pr7671-body-page-field-justification.hwpx --pdf pdf/semanticist21-20261015/pr7671-body-page-field-justification-2020.pdf --key pr7671-body-page-field-justification-hwpx --rhwp-bin target/pr-review/release-test/rhwp --compat 2022 --dpi 96 --page-workers 1 --embed-fonts full --font-path <실제-시스템-글꼴-디렉터리> --out output/pr-review/semanticist21-repeat/native
# 같은 인자에 --wasm-pkg pkg를 추가하고 --out을 .../wasm으로 바꿔 fresh WASM 전쪽 TSV를 산출한다.
node scripts/run-rust-test.mjs semanticist21_qualified_print_positions -- --cargo-profile release-test --target-dir target/pr-review --features native-skia --test-threads 8 --no-fail-fast
```

공개 입력·독립 PDF와 위 해시를 먼저 확인한다. 원래 글꼴이 없는 환경의 fallback 통과를 실제 글꼴 공급과 동등한 시각 근거로 사용하지 않는다. 폰트 미설치는 px assertion을 자동 skip하거나 느슨하게 만드는 사유가 아니다. 원시 레이아웃/단위 검사는 같은 독립 기대값을 유지한다.
