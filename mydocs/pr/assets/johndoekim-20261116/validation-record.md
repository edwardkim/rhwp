# source·실행·글꼴 공급 기록

> 최신 devel 재검증 완료: 아래 수치는 **직전 후보 `105be723b2`의 역사적 기록**이다. 최종 source `4deadfa963`의 실제 13,632개·Native 4,073개·전쪽 보호 및 승인된 Native 1행 예외는 [최종 기록](latest-devel-revalidation.md)을 따른다.

최종 실행 명령·종료 코드·해시는 이 문서에 기록한다. 원시 로그·MCP 응답·중간 JSON은 Git 제외 경로에만 둔다.

## 제품 source와 실제 산출물

- 생산 source: `47177a9bfc560ee52a12ae5588f6518b7950e4b9`. 실행 당시 로컬 test source: `7607048ce6d09c6f362516328b33e244728b5dea`. 제출 코드: `e958314dc9078fdb55e7cde04227a1a22054a72d`.
- 위 두 SHA 사이의 Rust 제품·crates/vendor·Cargo·build.rs·asset·글꼴 fixture·sample·PDF·실제 캡처 스크립트 변경: **0개**.
- 제품 코드가 같은 Native·최적화 WASM을 시각 검증에 사용했다. 주석 보완 뒤 Rust test source는 새 head에서 재검증한다.

| 대상 | source | 종료 | SHA-256 |
| --- | --- | ---: | --- |
| base-native-release-test-build | `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826` | 0 | `f748c034f769e8f8469ba8669f2a870a92cf0ecad17f0559ceecbe0ca21a11b5` |
| candidate-native-build | `47177a9bfc560ee52a12ae5588f6518b7950e4b9` | 0 | `e3c1cf286be0e5a157e13eed5c3c94213576b9f87e414a1a59d7b73127d6e2ba` |
| base-wasm-build | `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826` | 0 | `6aa008c88057919f607dc7c5bfa490bebf07e75ceb9ce4619aed1e20a2a0ebb9` |
| candidate-wasm-build | `47177a9bfc560ee52a12ae5588f6518b7950e4b9` | 0 | `994e8955edb9bad3ee479bb531698568fbb2f99bcdfcc334f1a9d179cb144666` |

실제 build 명령 (공용 `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`):

```bash
cargo build --locked --profile release-test --features native-skia --bin rhwp
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 scripts/wasm-pack-locked.sh --target web --out-dir "$rhwp_review_wasm_pkg" --release
```

WASM은 wasm-opt를 포함한 완료 package를 Chromium 152.0.7946.0 (revision 1660786)에서 실행했다.

## 원 기여 보존

| PR | 파일 | 원 head와 현재의 같은 Git blob |
| --- | --- | --- |
| #9081 | `src/renderer/layout/shape_layout.rs` | `8600a9a91cff9a2f30f2c99145f485f963d32ab9` |
| #9084 | `src/serializer/body_text.rs` | `c77b2eaebc35961550f0ec844f71ea14a53276c5` |

## 독립 구형 수식 글꼴 복구

- 이전 독립 문자표와 같은 HWPX를 관리형 한컴 2020 `11.0.0.9136`에서 직접 Print했다. 출력은 4쪽이며 `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`이다.
- 대상 문서의 PDF를 글꼴 재료로 사용하지 않았으며 이동·스케일·윤곽 보정을 하지 않았다.
- 이전 컨테이너 SHA와 현재 컨테이너 SHA는 다르다. 실제 원본 77개 글리프의 outline·advance 해시는 [기존 provenance](../pr8896/font-supply/legacy-equation-arithmetic.provenance.json)의 같은 항목과 모두 일치한다.
- 독립 입력 SHA-256: `3a7ebc1d2ed90a3325c9f6f053c005508ec85f18aa19e6c11e7fc08017330e72`.
- 새 독립 Print PDF SHA-256: `065f8f3eee0b760ec2e058124c922cb40322fd639caab1c234b068a4a23d11fa`.
- 이전 글꼴 컨테이너 SHA-256: `9366877ba5d2cad2ed97963b0f127ce4c9bb72c09764783e60c6cf5c2301c375`.
- 새 실제 글꼴 SHA-256: `cea780d92c7c3f45f534d256d56e948bb2b06c04634dbc135140c8ebd2d8e13a`.
- 재사용할 로컬 설치 위치: `/home/edward/mygithub/ttfs/local-validation/legacy-equation/`.
- 사설 글꼴 프로그램·문자표 전체 PDF·서버 주소·인증 설정은 커밋하지 않는다. 원본 글리프 해시는 아래에 보존한다.

| Unicode | 원본 글리프 SHA-256 |
| --- | --- |
| U+0028 | `2dc0e94c644b95e82d765215e72b131558671aa478ac9d77dbb091015976bea6` |
| U+0029 | `5d548f35a7372320e2e73c66b2b4fb7ed1f443fa5254943e1a6ee73c81f217de` |
| U+002B | `f9aa7ad64168d903b8b158e97c776a60495f5c17664fbaad47d4bd683f02ccaf` |
| U+002C | `acff9763becfa79b755a58e6ca144fa16c59f8cda27d97556ce545dd05606717` |
| U+002D | `78b7a478e4db2f8c1c31877e5b2f71d2f8d58a2fe909053b8c980ccd5a8a0b16` |
| U+002E | `80f6df42a5f365f162d62be01da721163f3e7869cb65d23b36aa6c463e8ea129` |
| U+002F | `a7549721e4ede6ba23705b5a9f3d303692e843c400ae1377cda6a5a082c9e6a0` |
| U+0030 | `3aeea3215ae99d06a41633410a7b607d19015b730f87c9664cfc11619148e9f5` |
| U+0031 | `6f86b1f7f65fd0324818ff84434a4e4aced02045c3cda3f922c807f2e6411422` |
| U+0032 | `56b65864adc7e5db8500a9cfb1c5a0c365f6b869bddf02583b6297a20e1e5c09` |
| U+0033 | `9cb48677a17b287661a31f4759b34ff4d57a4e43467a4f9d91d213b077077aa5` |
| U+0034 | `47fba9edd3acaf0323b5b2666e387ed29f93177afcde5990c3c8c1a6ab394665` |
| U+0035 | `1f49fb7bc2f240f055c3524fa3ea909335cedd8c2bd5fb6e5a9a2429748dc71e` |
| U+0036 | `02da657a39d93714d837d04cfb14b7d49a1dc8e3f8bbe1283b50b6876f79b390` |
| U+0037 | `7d2e04cf3f0feae15875226eb2521ad0d1f33858241ee058c0db1f26f55c148c` |
| U+0038 | `a87677a4e489b1d36b55cb783d9800ad324b1ff2919e0a53ad78e7ca059bcb26` |
| U+0039 | `b42487b001d4f3621242bfe1cf5fd60e2468c8986bba6a8004a1f9e2c1ec5574` |
| U+003A | `7d11cd78fa22690cf3caf8a98821655ee8191b483a264bf1e2cf5b001aa57e97` |
| U+003C | `b53add89d66eb005074c24c7f7a94d48f2a57ea916a0f45d581f5edd98f0b821` |
| U+003D | `c34edfa3779d40e2567147a7632961498f57441ab0cc13b8395015169aa3f4b7` |
| U+003E | `96c1b923beb1b54ed1f76d7cb4682fc83a5a2b9602a85b7efa016b613534a700` |
| U+0041 | `a9a9d405431ac44167b07c742fb089c0fbfd50150cc1826e2e3b8c0061eb8aca` |
| U+0042 | `51d10d4e120b4f66c9799d80247cb19af0ad39f7e1b982c676a38e82891332de` |
| U+0043 | `0f3cc3542772a6720337e5d7fea2e7c56d632a92e8bd8c09499ad81c5d604bcc` |
| U+0044 | `ef792dbafc3c0d17c5fcf0fe9b41f58802653b82ea6d51f9b84091fe2e63db77` |
| U+0045 | `5368f666b46c9ed45d792f8887818c2028ee03041eb7570a913e026980549ce8` |
| U+0046 | `6d7c1c217b76c872ecf7975d35ca2d97f7715f7c4e627cb084711cc2ca01366b` |
| U+0047 | `472578672605714575c2ee0a17a684b82e7e5f7b14af8612cc6a73545ecc2d6e` |
| U+0048 | `fbad3d31c05d69dbbcea163feea352354985c280731e404427dfb44402ca589b` |
| U+0049 | `981ef91277ace3782bf7668a1a5412af7c89bf2f440870712f6b82d2a57dfb83` |
| U+004A | `62f393da202fcf97b96c24cf40f465dfed42a369e3547629f151d792cb88d45f` |
| U+004B | `a246c20ab427cb803dd6cbba382a01e3c35c1147f37c19f984faf48607fd50ba` |
| U+004C | `d1e5eea693bdacf1dd8bf3df1c5c8b20fe9665532ad806db68ff76755dc655aa` |
| U+004D | `2e6ad3fc99965455e45205fbaaaacbb504809a70eeadc94f3ab2bd22495b3b21` |
| U+004E | `c6c6d169a1f1181ea836e10953fc855fcfc854df8100b4bdf84aee030b9e3671` |
| U+004F | `ce465c2e82c02b24fc35b389a9243bff94f5033e75ceec3b9572f01d00877914` |
| U+0050 | `f90cb5d99f4b281a48bbbe4c9abeb038a401905d9845c5c24791fb3e16e8de0a` |
| U+0051 | `9e2a2dbd3cf0a144d34525dc50ef1477f16f76587f80024ae249686f87807fa8` |
| U+0052 | `2e58593a10dd61dc03d9be921e694d5997b02b56df735cef9a0ccfba300ab19a` |
| U+0053 | `311c84d89e9f0889cd741a119c346455bddcebbd8eededcb37e5ecc341fcd73e` |
| U+0054 | `2bf14c74a1d0a9a077a4cbc20f062ed21c92a3ec936abd521b0ba99918ca9033` |
| U+0055 | `2e402c14ac15ad9582c5c7c5120f0a19bea71d1b5bb58388be1e824f174519c5` |
| U+0056 | `8e481d5f72d19eb519805d640d182336403535baacf7a4fa8a54c476084c59e6` |
| U+0057 | `dfab2efda9b9bd1ad02f414eabeecc9d9cefd8b87a2cb32bdc97fb4a0561a90e` |
| U+0058 | `8d93f42058c99f8b9e424c75deb536f968a4745a40fac93fe856221117ae8c80` |
| U+0059 | `f800ab08fe6bdcccb2d4bcd7f2c7e7ee13f6c1d2201569ab16d414dd8a3662d7` |
| U+005A | `cb245c7279a96cad96852d2cd56e54c36cd59e6f8b1bd86b1fc7fe283526820b` |
| U+005B | `180ed57b8e069cca8b107e1b9b469c88c5bd9a1f0709bd1cccd7aad36360183d` |
| U+005D | `6d824db90f0145e13d6492ff85aa5e82df46b80fcc1218e594f26ec62978d224` |
| U+0061 | `1933e44e391ac3cc391f29ebf8224e7d28a1ad80e7a52102cf4df7575baade33` |
| U+0062 | `800ea87eb6fa83c48d3581656cf84c3d21dc1b911fbfa8f007230359fef37ef5` |
| U+0063 | `674b45cd70fda50dcd8f59b19270e357268daef4d2e2bfbc0c3a3b1bc9529a49` |
| U+0064 | `56f285236b53c896e24e6b309863b45f588733a074417f49506f1f14875c12f5` |
| U+0065 | `51a7a06063586a0939dc0f9d7a3b70285871e43ec1f9e7267957a5ab81a94783` |
| U+0066 | `14e550e7850505d7c11c23c1a15282ce33849c6a3bc893c568a400569eb68798` |
| U+0067 | `29812e20597a017e160aa54a80b73ad54d9e171505a041719adb1e7421d788bb` |
| U+0068 | `b54543438cf16ca16decdc09a8b9a7e784b7d0f0c05df4b0fc1d9f589067b884` |
| U+0069 | `28d6a0f527dd5b2cea5ee5f4de40de111af8761bf9b190277422ddc78b7afaf4` |
| U+006A | `80cd0e3842377d7c1964f455ae9d7f1ae89bf980784ba143bf28848ff0459870` |
| U+006B | `bf779c2d61b3957c21f0aa0f6671397af9411ce65b118ecba9f526a9687f1aa4` |
| U+006C | `16f0b9a2a3cbbd8b1cb721f07dee76f5a799f892dfb268144247ecd47001c9e8` |
| U+006D | `14897e4c740f0aa36b4aeb33a2d0265e99ed517c2c71a645b813d31cae7066ec` |
| U+006E | `a35e5c5351a35693fab4b227983caaf346d1f00c2b1f0a00cea928212701c8d8` |
| U+006F | `bc74abdb5d1d31c722ea8e80e740560c8dd09806f040dafbbf27623cc386c566` |
| U+0070 | `ec8c19561fa7e5ef8169b95dc3feca9f91317995bfd2bd4b741a544836b978b5` |
| U+0071 | `e8dee4872aaea79dee7930bf058033ccfedd392bff8efc980133da29fb4beb00` |
| U+0072 | `67a1bea41e92d8b69b7eb232d5cef33c6537d5f806b7c0c1ed16166070f8a4d1` |
| U+0073 | `e324b5df7992925337063484dd87e9101d6f1106f7eab213b2efcd14f873bb33` |
| U+0074 | `6a8c5606fc83d19a173237f65be6e761cbcf4b40cdac366aa8047d82fa157c6d` |
| U+0075 | `de9297abb0241c67a960695c15a9b4e32e472f92f15bc81ef3841b657a63a71a` |
| U+0076 | `ab5e372d8dea1ff9a31a88c1a6dc07b26ca5e8aef822758a3856182380c79cc2` |
| U+0077 | `a400efc6797a392f3d5ed70ba9c53eae13cb3f0ef8217b5d3eef4a3086720260` |
| U+0078 | `93fdab4ac6540f7f5d1a7771b18f137c6cf588a82aa57d51e50e1aa3d7120cfe` |
| U+0079 | `adb38305e6849f09583ef74c533fbdb17e43b830f50f93b1701dd550ac790bc5` |
| U+007A | `8821a364ea3e723fc4f6e0bb21a1559491e21dfa480053d8bdf09d85e3ce5fab` |
| U+00D7 | `ce80ce249d3c694461bb2394459e4d7787281c4955d7e9cdf2619ec673f0db61` |
| U+00F7 | `01f109ae2b1dd2e5a7a7dabf66c3e57d446c63b15504ec84b6b82fe9de85080f` |

## focused 최소 글꼴 실행

`bwrap`의 파일 시스템에서 사용자 홈 글꼴·Windows/HFT 경로를 노출하지 않고 `/usr/share/fonts`·`/usr/local/share/fonts`를 tmpfs로 가렸다. `RHWP_FONT_PATH`는 unset했다. Fontconfig은 저장소 `ttfs/opensource`만 가리키며 실제 공개 글꼴 5개를 확인했다.

- source `47177a9bfc560ee52a12ae5588f6518b7950e4b9`: focused 57 passed / 0 failed. 지정한 suite의 1,664개는 필터 제외이며 글꼴 때문에 skip한 검사는 없다.
- 범위: 신규 #9076/#9082, #9021·#9005, 필드 슬롯·인라인 이동·전체 IR baseline.

## 편람·공개 공통 검사

- 실제 Native `--handbook-gate`: 383쪽, passed, 하락 0건. [전쪽 TSV](handbook-gate/silhouette.tsv).
- 새 공개 공통 검사: 사설 글꼴을 숨긴 환경의 4문서 × Native/WASM 8개 실행, 모두 passed·raw 최저 100.00%.
- 최신 devel #9102의 공통 gate 단위 검사: `python -m unittest scripts.tests.test_visual_sweep_contributor`, 9 passed / 0 failed, 종료 0.
- 실제 review·overlay의 도형 외곽·그림 crop/테두리·영문 face와 줄바꿈·좁은 한글 줄을 직접 판독했다. 각 문서의 정식 하위 경로에 전쪽 TSV를 보존한다.

## 최신 base·PR 상태

- `upstream/devel=67b4ff8d3a55e5bb50c46a44315de749a4ef07a7`, 반영한 #9102는 실행 엔진 입력 변경 없이 문서·공개 gate를 추가했다.
- 원 PR 두 head와 reviewer `edwardkim`을 재확인했다. 두 원 CI는 성공했으며 이 새 후보의 CI를 대신한다고 기록하지 않는다.
- 2026-11-16 조회한 본인 `edwardkim` 열린 PR은 Draft 포함 0건이다. 다른 기존 PR의 미해결 CI 또는 병합 시뮬레이션 대기 항목은 없다.

## 실제 저장 검증 재현

세 입력은 #9084의 한 글자 값 fixture, 공개 #9058의 네 글자 값 원본, #9081의 안내문 fixture다.
각각 아래와 같이 첫 구역의 문단을 실제로 바꾼 뒤 두 번 저장했다. 값·이름 조회와 별도로 CFB의
`BodyText/Section0`을 해제하고 PARA_TEXT의 8-unit FIELD_BEGIN/FIELD_END와 table 토큰을 순서대로 대조했다.
저장 순서 수치는 [증적 본문](README.md)의 원본/base/후보 표에 있다.

```bash
"$rhwp_review_native" edit insert-text "$rhwp_review_input" --text x --section 0 --para 0 --offset 0 -o "$rhwp_review_first" --json
"$rhwp_review_native" edit delete-text "$rhwp_review_first" --count 1 --section 0 --para 0 --offset 0 -o "$rhwp_review_second" --json
"$rhwp_review_native" fields "$rhwp_review_second" --json
"$rhwp_review_native" export-hwpx "$rhwp_review_second" "$rhwp_review_hwpx"
```

HWPX는 재열기·재저장 뒤에도 XML의 begin/value/end/table 순서를 다시 확인했다.
모든 실제 CLI 명령은 종료 코드 0이며, 값만 유지되는 base의 잘못된 FIELD_END 순서는 별도로 검출됐다.

## 구현의 소비 위치

- #9081: `shape_layout.rs:510`의 적용 판정 → `:3492`의 기존 `textbox_reflow::in_frame` →
  공통 vpos 재계산 → `:4231`의 `control_line_indices`와 인라인 표 줄 계획·실제 배치.
- #9084: `body_text.rs:943/950`의 공통 stream 슬롯 → `:1390`의 이미 지난 필드 닫기 위치를 정렬·방출 →
  `:1467`의 뒤 컨트롤 루프. `Storage::collect`와 오류 전달 계약을 유지한다.
- 분할 컷·rowspan 예약·이어받기 알고리즘은 변경하지 않는다. 저장기 구조 검토와 #9058의 별도 줄 소속 원인을 구분한다.

## 최종 공개 증적 해시

검토·후속 댓글에 사용하는 실제 파일이다. `guide-table-zoom.png`와 각 입력의 Native/WASM review·overlay를 보존한다.

| 파일 | SHA-256 |
| --- | --- |
| `guide-table-zoom.png` | `b5815020a5828810cadacaac136dc2de6f5f3670d3dd49d12fa31a5c2024dab0` |
| `visual/candidate-native/guide/overlay.png` | `b9481c25695169a8344c05701a0526eaafe586ab42963600fa5444033157c1ed` |
| `visual/candidate-native/guide/review.png` | `05f2dfabbf6ff18a48de94ba2392044d1281e2b57c695247a0769d3d2fd10808` |
| `visual/candidate-native/no-field/overlay.png` | `3991a2f3d9fc80793c2deacb98c0f0f9959952d9dff0c32cdf88983bbe813030` |
| `visual/candidate-native/no-field/review.png` | `eaf33fb3d98e40e75b7f2f300fa323e5c23049dd1fcc710d4a31a14d567aabae` |
| `visual/candidate-wasm/guide/overlay.png` | `b1dec31ffe62c23fb3705407a386ea319467aaa1cfd4a54a8dcbe3833e338c3c` |
| `visual/candidate-wasm/guide/review.png` | `53895e86c4924294ab004e84b5bfe35a7dbdae4caad3b05f41adbab04a453924` |
| `visual/candidate-wasm/no-field/overlay.png` | `49c09dd7083bbb89f6131760c21e38cea35c4c6c5e644470af32a03392477def` |
| `visual/candidate-wasm/no-field/review.png` | `d4625184967efc8b2f2c915cd88a7118d78e5bde4355349e5dde2eb8bbe74d9d` |

#9058의 공개 진단 입력·독립 Print는 아래 해시로 고정했다. 값 한 글자 HWP는 기존 #9084 fixture를 재사용한다.

| 파일 | SHA-256 |
| --- | --- |
| `issue9058/field_end_slot_textboxWitnoneText-hwp2022-print.pdf` | `c8007709140c9ee62e25d8a9ec218e0643caedcfc1a8c10513e278d1036f5c6d` |
| `issue9058/field_end_slot_textbox_with_cursor-hwp2022-print.pdf` | `616af761237655c3ebfd793a36153f032cb884bf325b88e72340b40b8da66514` |
| `issue9058/field_end_slot_textbox_with_cursor.hwp` | `384c46a75ea7a7f80e7baf94cc0d446c7c18717aafb45b729c3571e81e4101f5` |
| `issue9058/field_end_slot_textbox_with_text-hwp2022-print.pdf` | `9eef0a599a7042ca8e6e63ef51da1458df34e6b4cb09f8da159a1fed85c30b42` |
| `issue9058/field_end_slot_textbox_with_text.hwp` | `4fe0ecb6c7723f752097c43696557eea02d986649bce12c300926108aa42dea2` |

## 복구 후 수식 직접 판독

2026-11-16, sample16 HWP 2022의 6·43쪽을 독립 Print·복구 후 Native·복구 후 WASM의 같은 영역에서
2배 nearest 확대해 대조했다. 96dpi의 6쪽 `(100,790)-(710,985)`와 43쪽 `(90,370)-(730,830)`을
사용했다. 분수·괄호·대괄호의 윤곽과 크기·기준선, ×·÷·+·− 기호와 앞뒤 본문의 간격을 확인했다.
주석이나 PNG 점수로만 완료 처리하지 않았으며 사설 글꼴이 포함된 추가 crop은 공개 증적으로 커밋하지 않는다.

| 쪽 | Native raw | WASM raw |
| --- | ---: | ---: |
| 6 | 100.00% | 100.00% |
| 43 | 100.00% | 100.00% |

<!-- FINAL_LOCAL_CHECKS_START -->
## 최종 test head 로컬 검사

- 실제 명령 실행 source: `7607048ce6d09c6f362516328b33e244728b5dea` (로컬 임시 commit). 제출 코드 `e958314dc9078fdb55e7cde04227a1a22054a72d`와 모든 Rust·test·Cargo·원본·PDF·캡처 스크립트 입력의 Git blob이 같다.
- `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`, Rust `1.93.1`.
- 회귀·Native 공식 검사는 [최소 글꼴 격리 절차](../../../manual/pr_review/local_validation.md#회귀-테스트의-최소-글꼴-환경)의 bwrap 파일 시스템 안에서 실행했다.

| 검사 | 종료 코드 | 실행 시간(초) |
| --- | ---: | ---: |
| suite-prepare | 0 | 2.2 |
| format | 0 | 27.3 |
| clippy-native | 0 | 138.5 |
| clippy-wasm | 0 | 136.6 |
| workspace-build | 0 | 190.4 |
| clippy-all-targets | 0 | 169.6 |
| full-regressions | 0 | 1080.0 |
| native-lib | 0 | 322.0 |
| native-placeholder | 0 | 263.7 |
| native-direct-pdf | 0 | 20.8 |

- release-test 전체: **13,614 passed / 0 failed / 45 skipped**. 기존 ignored 45개는 변경하지 않았다. 신규 #9076·#9082 6개도 모두 실제 통과했다.
- 실제 테스트 실행 499.092초, 컴파일·테스트 전체 1080.0초. Native 공식 3종도 모두 통과했다: lib 4035개·placeholder 2개·직접 PDF 36개, 합계 **4,073 passed / 0 failed**. lib의 기존 ignored 12개와 focused 필터 제외 349/292개를 구분한다.
- suite manifest: `--check --base-ref 67b4ff8d3a55e5bb50c46a44315de749a4ef07a7`, 종료 0. 2,371 sources·28 suites와 20 exceptions, 전체 integration targets 48/48.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir "$rhwp_review_target_dir" -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir "$rhwp_review_target_dir" -- -D warnings
cargo build --locked --workspace --target-dir "$rhwp_review_target_dir"
cargo clippy --locked --workspace --all-targets --target-dir "$rhwp_review_target_dir" -- -D warnings
cargo nextest run --locked --cargo-profile release-test --target-dir "$rhwp_review_target_dir" --tests --test-threads 14 --no-fail-fast
cargo test --locked --profile release-test --target-dir "$rhwp_review_target_dir" --features native-skia --lib -- --test-threads 14
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir "$rhwp_review_target_dir" --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir "$rhwp_review_target_dir" --features native-skia
```

최소 환경에서 실제 노출된 프로젝트 공개 글꼴:

- `ttfs/opensource/NotoSansKR-ExtraLight.ttf`
- `ttfs/opensource/NotoSansKR-Regular.ttf`
- `ttfs/opensource/NotoSansSymbols2-Regular.ttf`
- `ttfs/opensource/RHWPOldHangul-Regular.otf`
- `ttfs/opensource/RHWPPublicBatang-Regular.ttf`

<!-- FINAL_LOCAL_CHECKS_END -->

## 제출 이력과 검증 입력 동일성

원시 실행 로그·MCP 응답·중간 JSON을 공개 이력에 포함하지 않도록 미게시 로컬 checkpoint를 정리했다.
실제 실행 당시 `7607048ce6d09c6f362516328b33e244728b5dea`와 제출 코드 `e958314dc9078fdb55e7cde04227a1a22054a72d` 사이의
Rust·test·Cargo·build.rs·sample·PDF·asset·실제 캡처 스크립트 Git blob 변경은 **0개**다.
원 기여 head 2개와 접수 계획·최신 base merge는 보존했다. 최종 문서·전쪽 TSV·대표 PNG만 다음 문서 commit에 담는다.

## 제출 직전 확인

2026-11-16 최종 fetch에서 devel은 `67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` 그대로다.
원 PR 2개의 실제 `refs/pull/<번호>/head`를 Git transport로 다시 조회해 접수 SHA와 같음을 확인했다.
본인 `edwardkim`의 열린 PR은 Draft 포함 0건이다. 신규 후보와 함께 검사할 미지정 본인 PR은 없다.

```bash
git merge-tree --write-tree 67b4ff8d3a55e5bb50c46a44315de749a4ef07a7 HEAD
git diff --check
```

병합 시뮬레이션은 종료 0이며 실제 merge tree가 후보 tree와 같다. 공백 검사와 변경 Markdown 7개의
내부 링크 검사도 통과했다. 최종 후속 문서 commit은 mydocs만 바꾸며, 원 기여 SHA·제품·test 입력을 보존한다.
원격 push·PR 생성·새 head CI·공개 리뷰·merge는 아직 수행하지 않았다.
