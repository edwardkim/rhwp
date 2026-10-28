# 개별 PR 변경과 증적 대응

검증 code head: `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`. 고정 base: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`. 그림은 D86의 동일 source byte로 만든 실제 Native/fresh WASM 출력입니다. 이후 MCP 기대값 수정은 테스트 한 파일뿐이며 렌더러 변경이 없습니다.

| PR | 입력·쪽 | 직접 입증하는 내용 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- | --- | --- |
| #8111 | `onsaemiro-control` p36 | KoPubWorld의 ㉠·∼·㉤ 전진폭과 이후 본문 | [review](images/pr-8111/native/onsaemiro-control-p036-review.png) · [overlay](images/pr-8111/native/onsaemiro-control-p036-overlay.png) | [review](images/pr-8111/wasm/onsaemiro-control-p036-review.png) · [overlay](images/pr-8111/wasm/onsaemiro-control-p036-overlay.png) |
| #8130 | `field-reset` p1 | 닫힌 중복 PR의 자리차지 앵커 정상 대조군 | [review](images/pr-8130/native/field-reset-p001-review.png) · [overlay](images/pr-8130/native/field-reset-p001-overlay.png) | [review](images/pr-8130/wasm/field-reset-p001-review.png) · [overlay](images/pr-8130/wasm/field-reset-p001-overlay.png) |
| #8141 | `symbol-slot-2020` p1 | 2020 기호 슬롯의 수학 연산자 전진폭 | [review](images/pr-8141/native/symbol-slot-2020-p001-review.png) · [overlay](images/pr-8141/native/symbol-slot-2020-p001-overlay.png) | [review](images/pr-8141/wasm/symbol-slot-2020-p001-review.png) · [overlay](images/pr-8141/wasm/symbol-slot-2020-p001-overlay.png) |
| #8141 | `symbol-slot-2024` p1 | 2024 기호 슬롯의 수학 연산자 전진폭 | [review](images/pr-8141/native/symbol-slot-2024-p001-review.png) · [overlay](images/pr-8141/native/symbol-slot-2024-p001-overlay.png) | [review](images/pr-8141/wasm/symbol-slot-2024-p001-review.png) · [overlay](images/pr-8141/wasm/symbol-slot-2024-p001-overlay.png) |
| #8157 | `titled-host` p1 | 공동 앵커 host 표의 바깥 여백과 본문 흐름 | [review](images/pr-8157/native/titled-host-p001-review.png) · [overlay](images/pr-8157/native/titled-host-p001-overlay.png) | [review](images/pr-8157/wasm/titled-host-p001-review.png) · [overlay](images/pr-8157/wasm/titled-host-p001-overlay.png) |
| #8159 | `saved-host` p4 | 양수 오프셋 자리차지 표의 문단 원점 | [review](images/pr-8159/native/saved-host-p004-review.png) · [overlay](images/pr-8159/native/saved-host-p004-overlay.png) | [review](images/pr-8159/wasm/saved-host-p004-review.png) · [overlay](images/pr-8159/wasm/saved-host-p004-overlay.png) |
| #8163 | `omitted-after` p2 | 저장 사다리의 문단 아래 간격 생략과 후속 문단 | [review](images/pr-8163/native/omitted-after-p002-review.png) · [overlay](images/pr-8163/native/omitted-after-p002-overlay.png) | [review](images/pr-8163/wasm/omitted-after-p002-review.png) · [overlay](images/pr-8163/wasm/omitted-after-p002-overlay.png) |
| #8185 | `offset-min` p1 | 오프셋 표 첫 조각과 후속 쪽 경계 | [review](images/pr-8185/native/offset-min-p001-review.png) · [overlay](images/pr-8185/native/offset-min-p001-overlay.png) | [review](images/pr-8185/wasm/offset-min-p001-review.png) · [overlay](images/pr-8185/wasm/offset-min-p001-overlay.png) |
| #8185 | `rowspan-header` p2 | 반복 제목행 rowspan과 본문 소유 | [review](images/pr-8185/native/rowspan-header-p002-review.png) · [overlay](images/pr-8185/native/rowspan-header-p002-overlay.png) | [review](images/pr-8185/wasm/rowspan-header-p002-review.png) · [overlay](images/pr-8185/wasm/rowspan-header-p002-overlay.png) |
| #8185 | `policy1790387-actual` p6 | 정책연구 원본의 반복 제목·표 본문 이어받기와 범례 | [review](images/pr-8185/native/policy1790387-actual-p006-review.png) · [overlay](images/pr-8185/native/policy1790387-actual-p006-overlay.png) | [review](images/pr-8185/wasm/policy1790387-actual-p006-review.png) · [overlay](images/pr-8185/wasm/policy1790387-actual-p006-overlay.png) |
| #8159 | `saved-host` p1 | 앞 빈 host와 양수 오프셋 표를 함께 둔 경우의 본문 원점 | [review](images/pr-8159/native/saved-host-p001-review.png) · [overlay](images/pr-8159/native/saved-host-p001-overlay.png) | [review](images/pr-8159/wasm/saved-host-p001-review.png) · [overlay](images/pr-8159/wasm/saved-host-p001-overlay.png) |
| #8185 | `rowspan-header` p3 | 이어받기 조각에서 제목행 0·1 반복과 뒤 본문 보존 | [review](images/pr-8185/native/rowspan-header-p003-review.png) · [overlay](images/pr-8185/native/rowspan-header-p003-overlay.png) | [review](images/pr-8185/wasm/rowspan-header-p003-review.png) · [overlay](images/pr-8185/wasm/rowspan-header-p003-overlay.png) |
| #8152 | `curriculum-control` p48 | 선택 span 행의 Center 정렬 | [review](images/native/curriculum-control-p048-review.png) · [overlay](images/native/curriculum-control-p048-overlay.png) | [review](images/wasm/curriculum-control-p048-review.png) · [overlay](images/wasm/curriculum-control-p048-overlay.png) |
| #8152 | `curriculum-control` p87 | 변경하지 않은 이웃 셀의 선언 행 띠 | [review](images/native/curriculum-control-p087-review.png) · [overlay](images/native/curriculum-control-p087-overlay.png) | [review](images/wasm/curriculum-control-p087-review.png) · [overlay](images/wasm/curriculum-control-p087-overlay.png) |
| #8152 | `curriculum-control` p178 | 절대 쪽 프레임과 첫 이어받기 본문 | [review](images/native/curriculum-control-p178-review.png) · [overlay](images/native/curriculum-control-p178-overlay.png) | [review](images/wasm/curriculum-control-p178-review.png) · [overlay](images/wasm/curriculum-control-p178-overlay.png) |
| #8152 | `regulatory86712` p27 | 이어받기 셀의 닫는 선과 아래 표 | [review](images/native/regulatory86712-p027-review.png) · [overlay](images/native/regulatory86712-p027-overlay.png) | [review](images/wasm/regulatory86712-p027-review.png) · [overlay](images/wasm/regulatory86712-p027-overlay.png) |
| #8152 | `cbta1613` p48 | 기존 정상 대조군의 최저쪽·잔여 농도 차이 | [review](images/native/cbta1613-p048-review.png) · [overlay](images/native/cbta1613-p048-overlay.png) | [review](images/wasm/cbta1613-p048-review.png) · [overlay](images/wasm/cbta1613-p048-overlay.png) |

## 화면·비시각 계약

- #8170: mock SSE E2E의 AI 읽기·변경 적용·새 문서 작성 화면입니다. 실제 유료 API나 Print 일치율의 증거로 쓰지 않습니다.
- #8176: 글상자 비트 생성/부착/제거 및 HWP 저장·재열기 계약입니다. 무관한 문서 그림을 첨부하지 않고 정식 테스트와 실행 결과를 연결합니다.
- #8193: LF/CRLF·심볼릭 링크 checkout와 SFNT fixture 바이트 계약입니다. 화면 배치 변경의 증적으로 대체하지 않습니다.
- #8130: 작성자가 미병합 종료한 중복 구현이며 그림은 기존 정상 대조군의 유지 확인입니다.

## 입력·독립 PDF와 해시

| 조건 | 실행 입력 / SHA-256 | 독립 Print PDF / SHA-256 |
| --- | --- | --- |
| `field-reset` | `samples/stored_reset_float/field_float_reset.hwp` / `9dfe5e3fcdb1041de3e68c38ea4ebd95c3d51b858d7907607cc4c0b12ca13a14` | `pdf/stored_reset_float/field_float_reset-2020.pdf` / `2c7f2f14f3a60d9636e1d61507f3318eccc2fcced01852c360ed773dcd51f861` |
| `field-plain` | `samples/stored_reset_float/field_float_reset_plain.hwp` / `5eefa29cbad1ba0dbd369581e89da31cacae64e2b873f92353a2700ce00a1813` | `pdf/stored_reset_float/field_float_reset_plain-2020.pdf` / `a0f4e1806669482b587b0c511018fb6f17238e88909e4c3a702ce70120c2e4e7` |
| `symbol-slot-2020` | `samples/lang_slot/symbol_slot_bullets.hwpx` / `388dd0f84f46301803d5072d21585988b969adf75dce4eecaa9463145080d1fa` | `pdf/lang_slot/symbol_slot_bullets-2020.pdf` / `cc1c03ca618cdcf32cd924f85eed8ae32728d52613e699a2cefbcdd031654cab` |
| `symbol-slot-2024` | `samples/lang_slot/symbol_slot_bullets.hwpx` / `388dd0f84f46301803d5072d21585988b969adf75dce4eecaa9463145080d1fa` | `pdf/lang_slot/symbol_slot_bullets-2024.pdf` / `6e20a8f4d78a89fa024644c354ab484f9fdcf316c1179effcd2021cbd80ae03a` |
| `titled-host` | `samples/issue7687/titled-host.hwpx` / `7647230e6b7194bda394b8f9771523708104be1b48f54064e7243f2c5446dd74` | `pdf/issue7687/titled-host-2020.pdf` / `ac0b81e37e957bd923dd7b7cb0c9ab243ff0e2039ca9385880e9f73fe87c0521` |
| `saved-host` | `samples/issue8156/titled-host-hancom2020-saved.hwp` / `0fe65f118d8617552af12f1ef9ebeb3083cce0a6b5537fab43646fc30a0fd776` | `pdf/issue8156/titled-host-hancom2020-saved-2020.pdf` / `5ce45d9da28bd05627fbf4e084da3de380da990b145d7c1475f54fe1df005df8` |
| `omitted-after` | `samples/issue7874/omitted-spacing-after.hwpx` / `b1905e369c93f4086047e3c68b77474eab0468491fae7da97bcbd4d944ac0cbf` | `pdf/issue7874/omitted-spacing-after-2024.pdf` / `8a15a05c3f7c7f43e999e4bfa75971255a1892e03643f54f0ffc7cf848da2bce` |
| `hwp5-control` | `samples/HWP5-nopassword-123456.hwp` / `a34ecb8cde85b6db49c64a954cb7fa5d23b5f49367bc4753c90bfe89a075b50d` | `pdf/issue7994/HWP5-nopassword-123456-source-2024.pdf` / `69378cafd87877cb09813350e8bb83130a5dd9ef8239e4a0a79250ec7a7a928c` |
| `onsaemiro-control` | `samples/[2027] 온새미로 1 본교재.hwp` / `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce` | `pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf` / `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7` |
| `offset-min` | `samples/issue4599/1790387_offset_float_split_min.hwpx` / `a4f2f5fc2b3b68148100121344783964be75f11092a985d89317b963cdda1156` | `pdf/1790387_offset_float_split_min-2020.pdf` / `53ed89bf9c3b937a909eec7f43e215747ef47e43691abe9d61637c1e8b1c8d70` |
| `rowspan-header` | `samples/issue4599/1790387_rowspan_header_repeat.hwpx` / `59b8c232f433ecf04e2374505e4c6b18a9aac919a347e0f3a9b8f4d791e004a4` | `pdf/1790387_rowspan_header_repeat-2020.pdf` / `4e6bf98adea24d257accf50fdfdc5bd386736f3623e6d2a96cceb63d94c7d097` |
| `tac-successor` | `samples/issue4599/1790387_tac_successor_after_split.hwpx` / `1b2505d993e8948ba2cfd649a7403781488286aa2bcf45eacaa2285ebbee0942` | `pdf/1790387_tac_successor_after_split-2020.pdf` / `10e37138faf2bdb841275ad730dc9e844d6fdaf862b9fcb822f533712211b0b2` |
| `policy1790387-actual` | `samples/issue4599/1790387_policy_progress.hwpx` / `947e51913c561611397c4f7d88feaad7112da4adc658b21cbfd9471e00192411` | `pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf` / `4c0d9d0876410766d99f4589916971f97b751a3d71901068637c7e2cc1061f95` |
| `briefing1487` | `samples/issue6924/148751598-briefing.hwp` / `03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610` | `pdf/148751598-briefing-2020.pdf` / `98ce52ec0a6ed25ba73070b22c743cb72d129113456ae3019f417d1b53ee9dd3` |
| `franchise` | `samples/issue5966/1130000-202100008_franchise_review_report.hwp` / `a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55` | `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` / `4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009` |
| `regulatory86712` | `samples/86712_regulatory_analysis.hwp` / `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88` | `pdf/issue7382-regression-review/86712_regulatory_analysis-font-supplied-2024.pdf` / `ac34bb60492e3749a53183cfb63c814f86379895937dd1309d3bc3d2d1b8126a` |
| `cbta1613` | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` / `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7` | `pdf/issue6764/1613000-202200037-air-traffic-controller-cbta-hwp-2020.pdf` / `1e15e26100e3b4c4cd9737a0d2b116a449c5895efec9765680d79baa3b8f4380` |
| `task1749x` | `samples/task1749/saved_bounds_cumulative_page_break.hwpx` / `2fd531b3e235a714c3fef1a59ddcd9d23225f68f5c492adb17500d23f5cbfe2f` | `pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf` / `798e06db6946edcbb05e7399d3aec08d847a57afc375d9d9e67b435e9e1f40d8` |
| `task1749h` | `samples/task1749/saved_bounds_cumulative_page_break.hwp` / `49478ec2a01315594bcc5bf0bbbdbc43b86c134cefba3c979d617180bfd43f81` | `pdf/task1749/saved_bounds_cumulative_page_break-hwp-2020.pdf` / `0b219907dc9fcaf34175ee70b59c3124c1fcd76bd0613e49fcf7529cec6e6dc9` |
| `near-top-reset` | `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` / `18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a` | `pdf/issue5941-1480000-201900698-2020.pdf` / `fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc` |
| `curriculum-control` | `mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp` / `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e` | `mydocs/pr/assets/issue7445/education-curriculum-map/existing-hwp-hancom2020-print.pdf` / `bc16ee92640644307fd1ce92ab8ffe663088210caa7740bef620daaa0187e197` |
| `math1342000-actual` | `samples/issue8141/1342000_math_textbook_field_review.hwp` / `1c2d27c5f431141b81052ea308039aa43c2899488b54b48a1d1ff2f6c487047e` | `pdf/issue8141/1342000_math_textbook_field_review-hwp-2020.pdf` / `956ede8003ccd245d5dd7021a0e4af4bd9264a1d95eac77dda134f698b230972` |
| `roadmap-control` | `samples/issue5941/1490000-201600081_roadmap_research.hwp` / `a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9` | `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf` / `133f79cad7b756ebef777d253befb14f9407faaf7fe04eeea66036f3e049bbe7` |
| `onsaemiro-converted-control` | `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp` / `c9ef761953944665e7f9886f5414955c9d52291f1b2e2fd12f62277aafdafa3e` | `pdf/hwpx/hancom-hwp/[2027] 온새미로 1 본교재-hwp-2020.pdf` / `58ff2fdc358d38c503d61a1e35a78320099b0671860047f1ac40151787247f3d` |
| `source8152-d2176` | `samples/hwpx/21761835_jeonjik_exemption_table.hwpx` / `53b33bfca8d8a80534f27564ecee86403c6d1e3dc83cf405c7d6d158ae9625d2` | `pdf/task2146/21761835_jeonjik_exemption_table-2020.pdf` / `d24191e125ce03290fdd55414ff480d8e00ade177d2b51352a61dbafbedd9ca0` |
| `source8152-d17253` | `samples/issue6756/17253153-traffic-safety-designated-routes.hwp` / `0bb29b403355463d16c8dab2107431cb258ec73f8ada8ec564f2200fd8aa2721` | `pdf/17253153-traffic-safety-designated-routes-2020.pdf` / `deb6aad07fde6198443c95360a2dcbaea7781dce66e769c4754c0fb8fca8f6be` |
| `source8152-d80168` | `samples/80168_regulatory_analysis.hwp` / `c8ad10fe9f07be5119cd804278017aefa46e555bbee4f05f0f5132fe4f591a22` | `pdf/80168_regulatory_analysis-hwp-2024.pdf` / `b233df7098547df43455f49c13ba5b63f1f06113c853a4773f86a94d6b34a982` |
| `source8152-d1853` | `samples/issue1853_caption_precedes_body_split.hwpx` / `46bd0142f7fe2f68dffb9768e08d65488feeb19f5fd6916f9ad90ae5d72a96a9` | `pdf/issue1853_caption_precedes_body_split-hwpx-2020.pdf` / `6577db6936337f4003553c1b8c492cfac42f06554b7dddca640b865ce5a100a8` |
| `source8152-d1880` | `samples/issue1880_takeplace_host_before.hwpx` / `9807766f5b4b2ad352362b5384f3bc0611a89cfdb1b65bd3d6682d7d571dca2c` | `pdf/issue1880/issue1880_takeplace_host_before-2024.pdf` / `662735e5f28fd423ffd158335cea1abfee1ed92b19c125efbea20c46dceca446` |
| `source8152-d5169` | `samples/issue5169_viewtext_changetracking.hwp` / `c003cdb0dd9aac5fda9c82d4f3be20a327522cf2deb680dc25134c7c6c8c3462` | `pdf/issue7674/issue5169_viewtext_changetracking-2020.pdf` / `affa9d0f6325d77317b4d206feec9c3cd5f62f9181a11f39efffc41fbda88f18` |
| `source8152-hfine` | `samples/issue7702/humanmyeongjo_condense_end_fine.hwpx` / `18e7f00b1a1714e0afdea4b41ee7d87fbcdb1c290e74575082fce574d91f0675` | `pdf/issue7702/humanmyeongjo_condense_end_fine-2024.pdf` / `2ac26de6d0bf79e486a4ac882bfe8aca751d6121dc028a251e0fb205b3328fb0` |
| `source8152-ha10m` | `samples/issue7702/humanmyeongjo_ascii_10pt_m10.hwpx` / `367300d79682b5fd3945c6a0a7e40c2591ee8ae71cdaf9397b8a0f3b15ec021b` | `pdf/issue7702/humanmyeongjo_ascii_10pt_m10-2024.pdf` / `756902b7490316bf603b0dbf9f9d92b31aed988e5a2b4a23337d0d203dfca06d` |
| `source8152-dband` | `samples/issue8105/declared_row_band.hwpx` / `3ce5fc24400e5647921538aea20dbfb6999e4fd6b198d044b3f269992d23af91` | `pdf/issue8105/declared_row_band-2024.pdf` / `13c60cdc8f3fba6fe052a0bc29fbf9cd2c46298c091854ccd5e2eda39f7919d8` |
| `source8152-hs2` | `samples/hwpx_sample2.hwp` / `ca66d521d070ac8442e09fd3f283e17611eb9efb45eb4bf6c3a8f65e604f6b5f` | `pdf/hwpx_sample2-hwp-2024.pdf` / `1b2e13e966787f6b91f590c8901dcb988e73ac7063d5e1cb95c8952400fd9a47` |
| `source8152-d80168x` | `samples/issue1891/80168_regulatory_analysis.hwpx` / `1e00b54e1139515258260a657217e0b3baa0b1b652a2cd28dd4b9bc48aa262fe` | `pdf/issue1891/80168_regulatory_analysis-hwpx-2020.pdf` / `88996e41ed3e68b15a3cd0eb4eda98845d34225c5dcd2ca547c80ab6e1242744` |

## 증적 바이트 해시

| 파일 | SHA-256 |
| --- | --- |
| [base/native/briefing1487.tsv](base/native/briefing1487.tsv) | `df7ede25b379f17fdcd7ef5572bb32d5fe28ed4182dbdc73f2e84cb2979027e6` |
| [base/native/cbta1613.tsv](base/native/cbta1613.tsv) | `49ac7ab1906361fe3578021981539efa7bf0efc378f65006a409ba862685482c` |
| [base/native/curriculum-control.tsv](base/native/curriculum-control.tsv) | `9566cbbfb2dc587ad7b06020ced9d79b0dd6e4f2f3d245282cf2d421134c56eb` |
| [base/native/field-plain.tsv](base/native/field-plain.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/native/field-reset.tsv](base/native/field-reset.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/native/franchise.tsv](base/native/franchise.tsv) | `a693d91cf0bcd9db99f1e291c7d185f7f6169fea4f789ee88944165d75a0bc67` |
| [base/native/hwp5-control.tsv](base/native/hwp5-control.tsv) | `42e2df2c542051e7aca6ed03e426a5189213a0d8e1bccdb416da74e782924048` |
| [base/native/math1342000-actual.tsv](base/native/math1342000-actual.tsv) | `92aa52099d4d22f0707797cc3b57b380f9ee37329d1b83579241dc4ca7747649` |
| [base/native/near-top-reset.tsv](base/native/near-top-reset.tsv) | `6cdbb1d6e933608d3a45e788d6add659b450d721ed0884d849ddf12ca5b97828` |
| [base/native/omitted-after.tsv](base/native/omitted-after.tsv) | `d127ce924572f6909c52c12af5c5a31332165f0a241df247b51315931664afc9` |
| [base/native/onsaemiro-control.tsv](base/native/onsaemiro-control.tsv) | `bf847fa93da46ceb782280505c2718fa2492e3c88686dfa433cdd6ba3be580fa` |
| [base/native/regulatory86712.tsv](base/native/regulatory86712.tsv) | `d80241b30ef2f8b08e3eb109b26d5c0787e6ec17faae0531a61a0c740ab3e1db` |
| [base/native/roadmap-control.tsv](base/native/roadmap-control.tsv) | `ac8769922bafb302130e572562d081a23f84a245d61d4e4d8e80434d995ee495` |
| [base/native/rowspan-header.tsv](base/native/rowspan-header.tsv) | `563cb18f88868920f408b62b53af3a55481e7e9d3f41bfbf48dfcb3f0de1c31a` |
| [base/native/saved-host.tsv](base/native/saved-host.tsv) | `147344947dc62caad6a9849ff325a88b496a1be00faded6a66bfa8cf2a58d106` |
| [base/native/source8152-d17253.tsv](base/native/source8152-d17253.tsv) | `7703a59cc2df3855518686c972b53feffa5e586895734e87d4e4bf3347cea61f` |
| [base/native/source8152-d1853.tsv](base/native/source8152-d1853.tsv) | `31ffc9844c0d2bf7afb00e6c651dd2cafdc648784915193e4a1a3d22320aa2b5` |
| [base/native/source8152-d1880.tsv](base/native/source8152-d1880.tsv) | `e2b3aa1a3c0bf7cf22451b9fc157273205b9648c9ced505f67991055a9f5ebc7` |
| [base/native/source8152-d2176.tsv](base/native/source8152-d2176.tsv) | `70666a04ba45bb775a023b1f5f8384ee30635498d111f48d98f3e4786afeb497` |
| [base/native/source8152-d5169.tsv](base/native/source8152-d5169.tsv) | `3766af30e09141443ff48aca3acc71167044aef27337bb005cfa72b2ae6dd95a` |
| [base/native/source8152-d80168.tsv](base/native/source8152-d80168.tsv) | `46ecbbfce62f9e3a0459138a62e609aa71ef2df126e1870444638631f06fb9ff` |
| [base/native/source8152-d80168x.tsv](base/native/source8152-d80168x.tsv) | `1b167c67ec4863b7082d80e24911b4bb065fa568f12397860be3f123a1bf406d` |
| [base/native/source8152-dband.tsv](base/native/source8152-dband.tsv) | `321106bb285e4af6569375e75fb4045e5aff3977e8a293b49a3c0f1431dc603b` |
| [base/native/source8152-ha10m.tsv](base/native/source8152-ha10m.tsv) | `9e83e432d3f2300ef54a44fa7d98ebd345b4830e43bb1ebefd13e639b05b27a9` |
| [base/native/source8152-hfine.tsv](base/native/source8152-hfine.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/native/source8152-hs2.tsv](base/native/source8152-hs2.tsv) | `01b6ddec0d06c57551b41959457d17449b2334f64c78837c10d0613499f105c8` |
| [base/native/symbol-slot-2020.tsv](base/native/symbol-slot-2020.tsv) | `232cca86ba18845c118b6ee541bc230fcb6a09e249324ae0e4f6b9b2adb033bb` |
| [base/native/symbol-slot-2024.tsv](base/native/symbol-slot-2024.tsv) | `232cca86ba18845c118b6ee541bc230fcb6a09e249324ae0e4f6b9b2adb033bb` |
| [base/native/tac-successor.tsv](base/native/tac-successor.tsv) | `6956b2207d5f71d1741896a0ff2127837586fdaeeff3e82408fe476f0499eef9` |
| [base/native/task1749h.tsv](base/native/task1749h.tsv) | `08e646c8e29594c39d5168286a4dd36435a947798a637162ccebd49040b96459` |
| [base/native/task1749x.tsv](base/native/task1749x.tsv) | `71127b1683c174004d79e1d1d1bb792a793f55c7d4b8b6e910380596f5935e3c` |
| [base/native/titled-host.tsv](base/native/titled-host.tsv) | `cc970a771357df620c17317b8a64973de3ec01c2205a77212db73e5014b98efa` |
| [base/wasm/briefing1487.tsv](base/wasm/briefing1487.tsv) | `4d6e21c8af4224a15c02e8efdca85288cc933124af1c02adccca50676068ef17` |
| [base/wasm/cbta1613.tsv](base/wasm/cbta1613.tsv) | `8396e45b41a6998566ff800e8238c71acf7d711c85b63f87e9da92feb3d25a9b` |
| [base/wasm/curriculum-control.tsv](base/wasm/curriculum-control.tsv) | `bad496eec00aae3de46c6b9a724832fe78e6147a84b3f44c9c626db3057b1902` |
| [base/wasm/field-plain.tsv](base/wasm/field-plain.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/wasm/field-reset.tsv](base/wasm/field-reset.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/wasm/franchise.tsv](base/wasm/franchise.tsv) | `75cc6fe277d0e3333c497f61278967c00b5871e9faf70688d302c2a94567505e` |
| [base/wasm/hwp5-control.tsv](base/wasm/hwp5-control.tsv) | `310a8f812b85393cdb07a1627faee1dfc2035c9ee9e69a8e1f94038b9777c8a8` |
| [base/wasm/math1342000-actual.tsv](base/wasm/math1342000-actual.tsv) | `4bce1598e2e6871a4ddfb09f3cb97252e1f7e49e048e1ef0b2ed9c978c51960d` |
| [base/wasm/near-top-reset.tsv](base/wasm/near-top-reset.tsv) | `ec6580946f1bec055c9f943400112b8d42a8841f559cc58f645ca027cd229024` |
| [base/wasm/omitted-after.tsv](base/wasm/omitted-after.tsv) | `bfe281814efda18c1d4aeacbdfcaec01a1bd9e384023c5332e4c402ece01703c` |
| [base/wasm/onsaemiro-control.tsv](base/wasm/onsaemiro-control.tsv) | `d3028a1be6f5c870bb4f81bb820cf14ee78f4fa69e9fe108cc4c0c53835e18d0` |
| [base/wasm/regulatory86712.tsv](base/wasm/regulatory86712.tsv) | `0ad71cf629fda5e53cd8fd2e735ecfd603bbd2693887fd2826cd59bd157b06d4` |
| [base/wasm/roadmap-control.tsv](base/wasm/roadmap-control.tsv) | `a6dda17f5b30bd09368942da4076428b266c84eb3b64ac25c2a29776edcb8859` |
| [base/wasm/rowspan-header.tsv](base/wasm/rowspan-header.tsv) | `80ef763b48ddf44e0cf5b85402e0d113834f24ff1999837f77442ea0f0aacd6a` |
| [base/wasm/saved-host.tsv](base/wasm/saved-host.tsv) | `f2d3250468e6c29a63254e2b55d30a5a80fbfb99b2803c133f48082b0aa682be` |
| [base/wasm/source8152-d17253.tsv](base/wasm/source8152-d17253.tsv) | `7703a59cc2df3855518686c972b53feffa5e586895734e87d4e4bf3347cea61f` |
| [base/wasm/source8152-d1853.tsv](base/wasm/source8152-d1853.tsv) | `0df546ff53ab188c478c8c4cfd51abc18522949542ae522f5dafdcf1ad14f324` |
| [base/wasm/source8152-d1880.tsv](base/wasm/source8152-d1880.tsv) | `69008f5aa3f93b5841d6a3eab9d0be62f95d8ecab14353b6ea69470320a6d202` |
| [base/wasm/source8152-d2176.tsv](base/wasm/source8152-d2176.tsv) | `6337cbc30db6091b0e137bce4e33cc641f7a41ffd825969cdb6a94726ca96534` |
| [base/wasm/source8152-d5169.tsv](base/wasm/source8152-d5169.tsv) | `3209002ba048d3fed4a18bc3fe6b31827c429b0d88802527f23a2f21318198f7` |
| [base/wasm/source8152-d80168.tsv](base/wasm/source8152-d80168.tsv) | `73d18a8edd317c0db643f86d31253e1144c755f62fe28d3fdac18d1ec671862f` |
| [base/wasm/source8152-d80168x.tsv](base/wasm/source8152-d80168x.tsv) | `2bcd0edb0065570cde8d5e0afcf216d629128297f6abc2b109c5fad2a2a37e63` |
| [base/wasm/source8152-dband.tsv](base/wasm/source8152-dband.tsv) | `3370cea6ade5a2f8bef72bfa0aa136f14e363410ccfc375d7347b0f04901ae2b` |
| [base/wasm/source8152-ha10m.tsv](base/wasm/source8152-ha10m.tsv) | `9e83e432d3f2300ef54a44fa7d98ebd345b4830e43bb1ebefd13e639b05b27a9` |
| [base/wasm/source8152-hfine.tsv](base/wasm/source8152-hfine.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [base/wasm/source8152-hs2.tsv](base/wasm/source8152-hs2.tsv) | `9378b36159274f7b7fcc034a5941e291845e7fe4ae326e3cac5a68e45e0b10dd` |
| [base/wasm/symbol-slot-2020.tsv](base/wasm/symbol-slot-2020.tsv) | `9ed060c9596f70efa33a5218fb4995d77c437593aea065844e2a71903605b671` |
| [base/wasm/symbol-slot-2024.tsv](base/wasm/symbol-slot-2024.tsv) | `9ed060c9596f70efa33a5218fb4995d77c437593aea065844e2a71903605b671` |
| [base/wasm/tac-successor.tsv](base/wasm/tac-successor.tsv) | `f616ba64ebf4591d7e62ec16e20510cb9452dc97d9f1d9d20ce8a91165717eff` |
| [base/wasm/task1749h.tsv](base/wasm/task1749h.tsv) | `22d1a5fe0c86ff358b445034cd0196278a04aacbfb4642b9730ef22da08b1020` |
| [base/wasm/task1749x.tsv](base/wasm/task1749x.tsv) | `b60739a1a46e36d92fec74705c072d07b4de08f8353c5ed9951bdbd9e5cb070b` |
| [base/wasm/titled-host.tsv](base/wasm/titled-host.tsv) | `8a80d245b35dd8c4f4d77a4a3bd2e48c7db9e87240f691268e85bedd7b82d367` |
| [head/native/briefing1487.tsv](head/native/briefing1487.tsv) | `4ab9db5c7e4eefd0702f5de0dba44f42af08229e8690b4e1175ad4645a251d5b` |
| [head/native/cbta1613.tsv](head/native/cbta1613.tsv) | `1d3e64d2a0a6a492a3dd0e3229592943881cd916f18d4d68448e2668d4a38cb5` |
| [head/native/curriculum-control.tsv](head/native/curriculum-control.tsv) | `abaad9d0614f4c46c9ac53b4dcde6e1b2cf8fd846ff591a301a64a6f41efd173` |
| [head/native/field-plain.tsv](head/native/field-plain.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/native/field-reset.tsv](head/native/field-reset.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/native/franchise.tsv](head/native/franchise.tsv) | `1105e76ef1230b1ac83ebd2008ed239362d9749215b1a4587694a87799a51d37` |
| [head/native/hwp5-control.tsv](head/native/hwp5-control.tsv) | `42e2df2c542051e7aca6ed03e426a5189213a0d8e1bccdb416da74e782924048` |
| [head/native/math1342000-actual.tsv](head/native/math1342000-actual.tsv) | `f816725fa284a67c77367d780ef17f89dea219d775d1aae8300c7f22841c87bb` |
| [head/native/near-top-reset.tsv](head/native/near-top-reset.tsv) | `6cdbb1d6e933608d3a45e788d6add659b450d721ed0884d849ddf12ca5b97828` |
| [head/native/offset-min.tsv](head/native/offset-min.tsv) | `787261d0a96917ee938f831d3813a3d946493114afd1ec79b08c1520ced3f6e3` |
| [head/native/omitted-after.tsv](head/native/omitted-after.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/native/onsaemiro-control.tsv](head/native/onsaemiro-control.tsv) | `ab7f497efb45a1d3a7098051ed12c49d995b47d01b0805df3120f0d0b3224cef` |
| [head/native/onsaemiro-converted-control.tsv](head/native/onsaemiro-converted-control.tsv) | `3bbe7184c1baafd93890f7844af7684641cf222a50a9293ef0894a940484d92e` |
| [head/native/policy1790387-actual.tsv](head/native/policy1790387-actual.tsv) | `3d06d738266697c18132f72ab0d2f8a31c1dbf8a45a8ea104f10236bf0918ba7` |
| [head/native/regulatory86712.tsv](head/native/regulatory86712.tsv) | `185709f91df2c77b05d541559b5737551660babbb70c84cfff5e30c01ada9c36` |
| [head/native/roadmap-control.tsv](head/native/roadmap-control.tsv) | `461bf60183a8b6158667eddd2c25d4a36aeb34f2c9ab6d4708f3bcc6e3fcc64c` |
| [head/native/rowspan-header.tsv](head/native/rowspan-header.tsv) | `df63f7ddc9b486623657065ff6fd2883bd552279328e29d034daf7204452b52b` |
| [head/native/saved-host.tsv](head/native/saved-host.tsv) | `460b8c7a74507a4137b2e78424a939982ca2e5c39beb4dc0fec3dd9c45ff5fc9` |
| [head/native/source8152-d17253.tsv](head/native/source8152-d17253.tsv) | `a3b50d6cb18d044185cfa5a9c2af975cb1d1239658dc7d0ce11ff09a915a76d3` |
| [head/native/source8152-d1853.tsv](head/native/source8152-d1853.tsv) | `31ffc9844c0d2bf7afb00e6c651dd2cafdc648784915193e4a1a3d22320aa2b5` |
| [head/native/source8152-d1880.tsv](head/native/source8152-d1880.tsv) | `20f9c1b0f470283fc7f6f032ba62c0f1b4f7278119558d307d70b30d8c773dec` |
| [head/native/source8152-d2176.tsv](head/native/source8152-d2176.tsv) | `70666a04ba45bb775a023b1f5f8384ee30635498d111f48d98f3e4786afeb497` |
| [head/native/source8152-d5169.tsv](head/native/source8152-d5169.tsv) | `3766af30e09141443ff48aca3acc71167044aef27337bb005cfa72b2ae6dd95a` |
| [head/native/source8152-d80168.tsv](head/native/source8152-d80168.tsv) | `46ecbbfce62f9e3a0459138a62e609aa71ef2df126e1870444638631f06fb9ff` |
| [head/native/source8152-d80168x.tsv](head/native/source8152-d80168x.tsv) | `b92c4f371c0a4b4c5897d6951e23994374248653e5fc1a202fcf4574d55ab061` |
| [head/native/source8152-dband.tsv](head/native/source8152-dband.tsv) | `321106bb285e4af6569375e75fb4045e5aff3977e8a293b49a3c0f1431dc603b` |
| [head/native/source8152-ha10m.tsv](head/native/source8152-ha10m.tsv) | `9e83e432d3f2300ef54a44fa7d98ebd345b4830e43bb1ebefd13e639b05b27a9` |
| [head/native/source8152-hfine.tsv](head/native/source8152-hfine.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/native/source8152-hs2.tsv](head/native/source8152-hs2.tsv) | `3b23d58e46258f63a936e8f68a087e70d5f47c414323814a96738fa3f47d2ff3` |
| [head/native/symbol-slot-2020.tsv](head/native/symbol-slot-2020.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [head/native/symbol-slot-2024.tsv](head/native/symbol-slot-2024.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [head/native/tac-successor.tsv](head/native/tac-successor.tsv) | `7c0ad234663845d73a637a1b1e94f5ce4b9f5b7936dcd7a1763d91450a51e0dc` |
| [head/native/task1749h.tsv](head/native/task1749h.tsv) | `08e646c8e29594c39d5168286a4dd36435a947798a637162ccebd49040b96459` |
| [head/native/task1749x.tsv](head/native/task1749x.tsv) | `427886707d75c6178b6a733056e1807d297ed7bdcf8993780c5793d757a1530f` |
| [head/native/titled-host.tsv](head/native/titled-host.tsv) | `cc970a771357df620c17317b8a64973de3ec01c2205a77212db73e5014b98efa` |
| [head/wasm/briefing1487.tsv](head/wasm/briefing1487.tsv) | `82c79efd17cf3583506b091f98d01d4d6d14436fe437e24df90672363b7336e8` |
| [head/wasm/cbta1613.tsv](head/wasm/cbta1613.tsv) | `36f7e1d04fbb6ed181db816e23f281795fe55b4cded43e4ee14fdefed93b711b` |
| [head/wasm/curriculum-control.tsv](head/wasm/curriculum-control.tsv) | `01b103d19854d0e1d6326a87157fad6ffc85bd78299de52947ada6956ac60640` |
| [head/wasm/field-plain.tsv](head/wasm/field-plain.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/wasm/field-reset.tsv](head/wasm/field-reset.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/wasm/franchise.tsv](head/wasm/franchise.tsv) | `aa4261a0ccb2dd4f006705d0d5b07d68e478bf1e3c95b5b85fa5a08c11d4f257` |
| [head/wasm/hwp5-control.tsv](head/wasm/hwp5-control.tsv) | `310a8f812b85393cdb07a1627faee1dfc2035c9ee9e69a8e1f94038b9777c8a8` |
| [head/wasm/math1342000-actual.tsv](head/wasm/math1342000-actual.tsv) | `d0619367822cd6b6ca6b3f7da6ff3afb0c4e34b80191a756fe0c6392b5148a22` |
| [head/wasm/near-top-reset.tsv](head/wasm/near-top-reset.tsv) | `ec6580946f1bec055c9f943400112b8d42a8841f559cc58f645ca027cd229024` |
| [head/wasm/offset-min.tsv](head/wasm/offset-min.tsv) | `2da481b5b289f98faae8e30bf5dba0771802fb2f724d980460b706bc03454835` |
| [head/wasm/omitted-after.tsv](head/wasm/omitted-after.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/wasm/onsaemiro-control.tsv](head/wasm/onsaemiro-control.tsv) | `c7035e1564922cbf2614623ba97bc969d8d85c407c5c95aa2a5bafb66bd6d728` |
| [head/wasm/onsaemiro-converted-control.tsv](head/wasm/onsaemiro-converted-control.tsv) | `92a0c973ebc2b421603aa4242e83c403d4c04265bcb12d775940438a5567d75b` |
| [head/wasm/policy1790387-actual.tsv](head/wasm/policy1790387-actual.tsv) | `261560e4026cf08a9b58a2e28fa2d62d4cdc8741f6dea6cc72a54f6208d92cee` |
| [head/wasm/regulatory86712.tsv](head/wasm/regulatory86712.tsv) | `18f51c36859ce5e284939dbe57644daa337adc31dbf11568242e1d4840f862ab` |
| [head/wasm/roadmap-control.tsv](head/wasm/roadmap-control.tsv) | `a6a9f19f2499184b2d9779eeb1aa05a4b5ed6a3cc2f25f658db3551cfb8a2d95` |
| [head/wasm/rowspan-header.tsv](head/wasm/rowspan-header.tsv) | `e469a7f7ef9ededd86d33355c49ca96e2ca2ac80ca27f0abeb31882d9fb23787` |
| [head/wasm/saved-host.tsv](head/wasm/saved-host.tsv) | `460b8c7a74507a4137b2e78424a939982ca2e5c39beb4dc0fec3dd9c45ff5fc9` |
| [head/wasm/source8152-d17253.tsv](head/wasm/source8152-d17253.tsv) | `a3b50d6cb18d044185cfa5a9c2af975cb1d1239658dc7d0ce11ff09a915a76d3` |
| [head/wasm/source8152-d1853.tsv](head/wasm/source8152-d1853.tsv) | `0df546ff53ab188c478c8c4cfd51abc18522949542ae522f5dafdcf1ad14f324` |
| [head/wasm/source8152-d1880.tsv](head/wasm/source8152-d1880.tsv) | `4847fc536a4aa18eb91bfa1d4d35f5e5cbaad57fecc34b67b734464e130e5ce1` |
| [head/wasm/source8152-d2176.tsv](head/wasm/source8152-d2176.tsv) | `6337cbc30db6091b0e137bce4e33cc641f7a41ffd825969cdb6a94726ca96534` |
| [head/wasm/source8152-d5169.tsv](head/wasm/source8152-d5169.tsv) | `3209002ba048d3fed4a18bc3fe6b31827c429b0d88802527f23a2f21318198f7` |
| [head/wasm/source8152-d80168.tsv](head/wasm/source8152-d80168.tsv) | `73d18a8edd317c0db643f86d31253e1144c755f62fe28d3fdac18d1ec671862f` |
| [head/wasm/source8152-d80168x.tsv](head/wasm/source8152-d80168x.tsv) | `70a68f9563699c8791477851a0c2d6fac8b43d233fbffb6a9d4f768ab5a08f9b` |
| [head/wasm/source8152-dband.tsv](head/wasm/source8152-dband.tsv) | `3370cea6ade5a2f8bef72bfa0aa136f14e363410ccfc375d7347b0f04901ae2b` |
| [head/wasm/source8152-ha10m.tsv](head/wasm/source8152-ha10m.tsv) | `9e83e432d3f2300ef54a44fa7d98ebd345b4830e43bb1ebefd13e639b05b27a9` |
| [head/wasm/source8152-hfine.tsv](head/wasm/source8152-hfine.tsv) | `888e36110d95d3e030cb550c68ce509614b6865ab466222d207b01234183ba5c` |
| [head/wasm/source8152-hs2.tsv](head/wasm/source8152-hs2.tsv) | `1776a06cf2002dc2ab34d7dde73bb77bf1f1107f7b504653d8f7b676a897496d` |
| [head/wasm/symbol-slot-2020.tsv](head/wasm/symbol-slot-2020.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [head/wasm/symbol-slot-2024.tsv](head/wasm/symbol-slot-2024.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [head/wasm/tac-successor.tsv](head/wasm/tac-successor.tsv) | `956b170fa88debbb6d46f713f1991341bd239a1dece41c4911dabdfd30349b38` |
| [head/wasm/task1749h.tsv](head/wasm/task1749h.tsv) | `22d1a5fe0c86ff358b445034cd0196278a04aacbfb4642b9730ef22da08b1020` |
| [head/wasm/task1749x.tsv](head/wasm/task1749x.tsv) | `22d1a5fe0c86ff358b445034cd0196278a04aacbfb4642b9730ef22da08b1020` |
| [head/wasm/titled-host.tsv](head/wasm/titled-host.tsv) | `8a80d245b35dd8c4f4d77a4a3bd2e48c7db9e87240f691268e85bedd7b82d367` |
| [images/actual/canvas2d-80168x-p137.png](images/actual/canvas2d-80168x-p137.png) | `e034df97f5d9cba57b8ffd80c8f62561b7afd924a6e4aa1a93c9ecd7bc2f0f18` |
| [images/actual/canvas2d-80168x-p138.png](images/actual/canvas2d-80168x-p138.png) | `125c0e72e5f2db0e28cb6ec374b13164bd9cfb741c22d125ea7b6805b62bea01` |
| [images/actual/canvaskit-declared-row-band-p012.png](images/actual/canvaskit-declared-row-band-p012.png) | `6f9ef0a81fa84a79a33957da1a3516a1b54a06bbd46430e3fb99b7092f802a49` |
| [images/actual/canvaskit-symbol-slot-p001.png](images/actual/canvaskit-symbol-slot-p001.png) | `ddde4c5453f18df82b2971aa4b8655ddf628f6c151d969c799ea1da210e973e5` |
| [images/native/cbta1613-p048-overlay.png](images/native/cbta1613-p048-overlay.png) | `69c37127f9ce70423c7205463878c8f950f3e83cc7962cdc70fe156d896be391` |
| [images/native/cbta1613-p048-review.png](images/native/cbta1613-p048-review.png) | `c10b1dd71f743abfcb8e776c5466facf64040f395c3313808c047d4b8abadc9d` |
| [images/native/curriculum-control-p048-overlay.png](images/native/curriculum-control-p048-overlay.png) | `8a9c98eb2df4d4eeec117193388223df7bacbf133350f6b4d9caad24fe0ce06a` |
| [images/native/curriculum-control-p048-review.png](images/native/curriculum-control-p048-review.png) | `c2e380ae4c6db1ca375bb2ef10a33443fd5c9e7758143289c6d868bede1d5788` |
| [images/native/curriculum-control-p087-overlay.png](images/native/curriculum-control-p087-overlay.png) | `7fdae73c3761b9f325215a7039d25484b0cb9e97d77fe9880406edb0ccbd2177` |
| [images/native/curriculum-control-p087-review.png](images/native/curriculum-control-p087-review.png) | `40ba55cdd4c2068f3eee82dd0e0961d49a3f89b86ef699fb5fd5fc2bec037b71` |
| [images/native/curriculum-control-p178-overlay.png](images/native/curriculum-control-p178-overlay.png) | `38ac9b1fb2a4b3887230db3dca0c42d6d9cf7fa736fe17b070e029a1cb7b357f` |
| [images/native/curriculum-control-p178-review.png](images/native/curriculum-control-p178-review.png) | `56f9e014c4c7ed622345b1aff335596d3b2a278c4115e799e84fb4f4d0d02116` |
| [images/native/regulatory86712-p027-overlay.png](images/native/regulatory86712-p027-overlay.png) | `f87be802a4f6d27a1da3a50310d6c738cec903149aa049b99445bc705c7be3de` |
| [images/native/regulatory86712-p027-review.png](images/native/regulatory86712-p027-review.png) | `9530fddf5b5c457883ae16ceca109e697d567f8bb6ed8c87431b10e42d1e1faa` |
| [images/pr-8111/native/onsaemiro-control-p036-overlay.png](images/pr-8111/native/onsaemiro-control-p036-overlay.png) | `cade66dedea0614da89bf79c5b531a771dd2d91ee63c54c7a2db100c1e07c247` |
| [images/pr-8111/native/onsaemiro-control-p036-review.png](images/pr-8111/native/onsaemiro-control-p036-review.png) | `d85794e0f62dcf0a455542bfcc4f62358df995d5581dfc42ad73f0aa16439498` |
| [images/pr-8111/wasm/onsaemiro-control-p036-overlay.png](images/pr-8111/wasm/onsaemiro-control-p036-overlay.png) | `3f6a377447354d0ebe7fa41a277e0d9e6a227c0c82ecd73a2f25b06b9d1d7ffc` |
| [images/pr-8111/wasm/onsaemiro-control-p036-review.png](images/pr-8111/wasm/onsaemiro-control-p036-review.png) | `59ee98091f2bf0089d3d1c02ca81147d91c945b6341358ea9bb87f1bc20c95bb` |
| [images/pr-8130/native/field-reset-p001-overlay.png](images/pr-8130/native/field-reset-p001-overlay.png) | `f943cdaabede7e26e119481c050f635b7c167ddc7a857486699ee6bdaecf90e4` |
| [images/pr-8130/native/field-reset-p001-review.png](images/pr-8130/native/field-reset-p001-review.png) | `31114c91c88ef134613abe85056cd2e663fbc6b2317200b3f24755f18aaedadf` |
| [images/pr-8130/wasm/field-reset-p001-overlay.png](images/pr-8130/wasm/field-reset-p001-overlay.png) | `27464ee863a5fe4ea51364a680f4f5a1e1863651f5b78f0f86d0fd16039c035c` |
| [images/pr-8130/wasm/field-reset-p001-review.png](images/pr-8130/wasm/field-reset-p001-review.png) | `eea9e913bcf3231727b65833dda843706ac3150f002ad953282709842a6fb2d7` |
| [images/pr-8141/native/symbol-slot-2020-p001-overlay.png](images/pr-8141/native/symbol-slot-2020-p001-overlay.png) | `5583a379894e61635e377d81af4a760f23a7dbb1111bfdc258f12b326a79af6f` |
| [images/pr-8141/native/symbol-slot-2020-p001-review.png](images/pr-8141/native/symbol-slot-2020-p001-review.png) | `4288b56c7487b4bf76c64140667c37637395a3fb2dc868c385f955e9c87b8d1c` |
| [images/pr-8141/native/symbol-slot-2024-p001-overlay.png](images/pr-8141/native/symbol-slot-2024-p001-overlay.png) | `11be1fc17c0b4d444811d93341da128def43755814e93a2772a8fa2e498a8812` |
| [images/pr-8141/native/symbol-slot-2024-p001-review.png](images/pr-8141/native/symbol-slot-2024-p001-review.png) | `f984f31d33a1c333b04733a235083b9ba88b6f270fe48c48d9896330cc7b07a4` |
| [images/pr-8141/wasm/symbol-slot-2020-p001-overlay.png](images/pr-8141/wasm/symbol-slot-2020-p001-overlay.png) | `d777616990b17ecfceeeeea5fe5ffe900a5da05ae399e7fde33a6384f26a8d2c` |
| [images/pr-8141/wasm/symbol-slot-2020-p001-review.png](images/pr-8141/wasm/symbol-slot-2020-p001-review.png) | `095689d20874b98fce4efa374d172abc9dc4533450f80f1ce8c57a7edd0c47d6` |
| [images/pr-8141/wasm/symbol-slot-2024-p001-overlay.png](images/pr-8141/wasm/symbol-slot-2024-p001-overlay.png) | `21cc4bf050d6690f68d2b42cfc77a7a7195af5ac2edd26b4d50d2220d8a02307` |
| [images/pr-8141/wasm/symbol-slot-2024-p001-review.png](images/pr-8141/wasm/symbol-slot-2024-p001-review.png) | `4862d98d2ede4c6bf26068612e82cdce400a842f48fe308e2d8834289e3e51ed` |
| [images/pr-8157/native/titled-host-p001-overlay.png](images/pr-8157/native/titled-host-p001-overlay.png) | `a6afce5b35600dd4a7ed793eb8e6ad4a603b70d642ad230158d870e95319ba93` |
| [images/pr-8157/native/titled-host-p001-review.png](images/pr-8157/native/titled-host-p001-review.png) | `c328cd63909150bbd79fe8d82456f2b81781e90ab8c68beccd3ae68d51dcdacf` |
| [images/pr-8157/wasm/titled-host-p001-overlay.png](images/pr-8157/wasm/titled-host-p001-overlay.png) | `e4d78ba6110b1a808fe9c42484ac456573e5e272d0bc1e3177a34337107a51fb` |
| [images/pr-8157/wasm/titled-host-p001-review.png](images/pr-8157/wasm/titled-host-p001-review.png) | `1f22509d841f93f4119c880984eb7af6a73c7331461725d5644815887f4a1033` |
| [images/pr-8159/native/saved-host-p001-overlay.png](images/pr-8159/native/saved-host-p001-overlay.png) | `fac5639c60917c96d519de7cbca06e1f8f9aa2fb1cdbf387873abe3436bb213f` |
| [images/pr-8159/native/saved-host-p001-review.png](images/pr-8159/native/saved-host-p001-review.png) | `5e665f5dd44f59914533ac83dd7e5a00ddba1ae97e2f47481b397c0518bd313f` |
| [images/pr-8159/native/saved-host-p004-overlay.png](images/pr-8159/native/saved-host-p004-overlay.png) | `875635bd66f5fe6abee5838c3dfec2e01ff642c581df327569331c817398dd5a` |
| [images/pr-8159/native/saved-host-p004-review.png](images/pr-8159/native/saved-host-p004-review.png) | `b054d764363db6f1520ab9d390106effb6de4b3b9bb176e256d3acfb443c1144` |
| [images/pr-8159/wasm/saved-host-p001-overlay.png](images/pr-8159/wasm/saved-host-p001-overlay.png) | `2a4f7ee9af97f67b4f76a2ba0e290da932a80bbb6802aac3d016376ae754edd2` |
| [images/pr-8159/wasm/saved-host-p001-review.png](images/pr-8159/wasm/saved-host-p001-review.png) | `71545bd51056d1d8a0a3e79d2d369e720c6197828f9d49701898fc6902e45a58` |
| [images/pr-8159/wasm/saved-host-p004-overlay.png](images/pr-8159/wasm/saved-host-p004-overlay.png) | `5dccc02dbb77610f1a64bcd8abaf14ccb5dcc0cc3c730d0a43c604e4d848bfce` |
| [images/pr-8159/wasm/saved-host-p004-review.png](images/pr-8159/wasm/saved-host-p004-review.png) | `d0f6b54011e519cb958ad0b670a0d160000c4022f0ee0a9bf4e7a4fc7c4ca822` |
| [images/pr-8163/native/omitted-after-p002-overlay.png](images/pr-8163/native/omitted-after-p002-overlay.png) | `d7a9c9be4b86c2b36716404e5d375400c7a393c9e28a7f41b4a7ef1d7bcc24af` |
| [images/pr-8163/native/omitted-after-p002-review.png](images/pr-8163/native/omitted-after-p002-review.png) | `0f6ea9721faf8e5ad1663b02c5351025b840496d0b4808ec3c4898dcf33bfa89` |
| [images/pr-8163/wasm/omitted-after-p002-overlay.png](images/pr-8163/wasm/omitted-after-p002-overlay.png) | `be4440b5b9512f600baa57544bdfa558caca068db93f68e4e2df7fc797bd8868` |
| [images/pr-8163/wasm/omitted-after-p002-review.png](images/pr-8163/wasm/omitted-after-p002-review.png) | `b20f9713dfbd24638a672a5cf10819b7f532c4eb1aa3a7bf1d8bd3cb7245ee8a` |
| [images/pr-8170/ai-panel-answer.png](images/pr-8170/ai-panel-answer.png) | `099554b2dbd9e775148bb6a0c56451592e7b45af8ab5168ceb39c2a24d605a5e` |
| [images/pr-8170/ai-panel-change.png](images/pr-8170/ai-panel-change.png) | `92896cc3b588a9eb39cbf79f1c8054854157ba42ee5aa23798e2c80bd85aba0d` |
| [images/pr-8170/ai-panel-new-document.png](images/pr-8170/ai-panel-new-document.png) | `727de02dd824cf73272b878eca8d2b1b621fcdf070f47ec146dee1126b450074` |
| [images/pr-8185/native/offset-min-p001-overlay.png](images/pr-8185/native/offset-min-p001-overlay.png) | `83bf07a900f7f3e11040e5a47a926c401d69586941df690f4c512e02903e679b` |
| [images/pr-8185/native/offset-min-p001-review.png](images/pr-8185/native/offset-min-p001-review.png) | `c6202517f4579ddddd64dd484007b6aea08d9156c0e3a0c4456eaf77d1aaac95` |
| [images/pr-8185/native/policy1790387-actual-p006-overlay.png](images/pr-8185/native/policy1790387-actual-p006-overlay.png) | `d36727a23d93d2b8eea3f405822af4bf496605303e5303b6f58da8d0e8fb58f0` |
| [images/pr-8185/native/policy1790387-actual-p006-review.png](images/pr-8185/native/policy1790387-actual-p006-review.png) | `9056abfef5ee8c60524792f0bd4538f7cbfb9bda767db7552de4a6e0e2f29bfb` |
| [images/pr-8185/native/rowspan-header-p002-overlay.png](images/pr-8185/native/rowspan-header-p002-overlay.png) | `80528cbaa34aa3e2eac726d340eccd7eca2e790e259e4ec2a2bff18b9b68ec55` |
| [images/pr-8185/native/rowspan-header-p002-review.png](images/pr-8185/native/rowspan-header-p002-review.png) | `ba94a9920199c60cbe2ee96a747c97a70e7d8af079058d247937f545860e42d0` |
| [images/pr-8185/native/rowspan-header-p003-overlay.png](images/pr-8185/native/rowspan-header-p003-overlay.png) | `b990b348e95d5d19bc76e44ea5d3297afdcaf6add52ca136baec772423d82ee9` |
| [images/pr-8185/native/rowspan-header-p003-review.png](images/pr-8185/native/rowspan-header-p003-review.png) | `4cac6640df5691278f34ccdbdbd2e0f278b598d7347b9401dce10b38102ee688` |
| [images/pr-8185/wasm/offset-min-p001-overlay.png](images/pr-8185/wasm/offset-min-p001-overlay.png) | `c698d19cd5358636a4ab337b56375231cb846f520ca242b4191776ff794acede` |
| [images/pr-8185/wasm/offset-min-p001-review.png](images/pr-8185/wasm/offset-min-p001-review.png) | `47351f2a4f2745afe836f81a43c521fda5c96deedda485ed9f1f21e0f84838c2` |
| [images/pr-8185/wasm/policy1790387-actual-p006-overlay.png](images/pr-8185/wasm/policy1790387-actual-p006-overlay.png) | `9abf9aeadb2973ef7f14b07006b5ddf84ea73d7691ecf726552acfabb68a8e80` |
| [images/pr-8185/wasm/policy1790387-actual-p006-review.png](images/pr-8185/wasm/policy1790387-actual-p006-review.png) | `09febb8f4ba067c9047265f7163056b4c703c1e22022e21dfe14435e5cc7d2e7` |
| [images/pr-8185/wasm/rowspan-header-p002-overlay.png](images/pr-8185/wasm/rowspan-header-p002-overlay.png) | `d28c190bede32cd66a8ba50667c4bb0e0f6464d1c3d34e42ab0a02312804a625` |
| [images/pr-8185/wasm/rowspan-header-p002-review.png](images/pr-8185/wasm/rowspan-header-p002-review.png) | `992d6b1f7ab047eeea26765d17d9253ebf4fb792c670000b6a48391f3f2cbb09` |
| [images/pr-8185/wasm/rowspan-header-p003-overlay.png](images/pr-8185/wasm/rowspan-header-p003-overlay.png) | `99678a9dd496208e9c810d90698b80da6778614937c8d7f321e5363b91df687c` |
| [images/pr-8185/wasm/rowspan-header-p003-review.png](images/pr-8185/wasm/rowspan-header-p003-review.png) | `ae92634cc14b039eeb552c87021ac75f0a6f58cbb4bf1895931b34668e736035` |
| [images/wasm/cbta1613-p048-overlay.png](images/wasm/cbta1613-p048-overlay.png) | `42c673663686cf22134cfe49a74cff0e39859e00ad989ba945df321394999e6f` |
| [images/wasm/cbta1613-p048-review.png](images/wasm/cbta1613-p048-review.png) | `5cca68364f173efb219f363a5f34f8f9ddffa5938559c26f0b7dc30cb6bde527` |
| [images/wasm/curriculum-control-p048-overlay.png](images/wasm/curriculum-control-p048-overlay.png) | `329bab0ce8c310f712fdffab593d88e25d0e7c5ebaea4997421867cc04871d8c` |
| [images/wasm/curriculum-control-p048-review.png](images/wasm/curriculum-control-p048-review.png) | `72a5402053020434cd5fae93e57dc1c8a268ebae41d4144fcd6d0c9ed62471d5` |
| [images/wasm/curriculum-control-p087-overlay.png](images/wasm/curriculum-control-p087-overlay.png) | `1b94bbf13e10482671d97717c4ebc72c8309fcccfe21ccb45cfd9a8a5107af71` |
| [images/wasm/curriculum-control-p087-review.png](images/wasm/curriculum-control-p087-review.png) | `75ec61bf106a38e6eeee5377ac65876416e5756ef8ec7f3f86711c6f8d50e711` |
| [images/wasm/curriculum-control-p178-overlay.png](images/wasm/curriculum-control-p178-overlay.png) | `f088f7985507c149eb17739f619d3d170976f5807c6455b0737ddcb877a19cbc` |
| [images/wasm/curriculum-control-p178-review.png](images/wasm/curriculum-control-p178-review.png) | `4427f25323286ce914612ec7c65844e44251a59aded376d528395e8d8258a6f1` |
| [images/wasm/regulatory86712-p027-overlay.png](images/wasm/regulatory86712-p027-overlay.png) | `2ba0067ced9f650c1ed24f0fdd78e14f35fef3c243dca8413030a1aa7052b292` |
| [images/wasm/regulatory86712-p027-review.png](images/wasm/regulatory86712-p027-review.png) | `575ae12b071076fe4fe931e811e2bd72e8b9422cc761eb4583544204d41b5a98` |
