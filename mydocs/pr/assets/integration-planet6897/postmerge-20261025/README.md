# PR #8228 통합 후 #7445 종료 범위 대조

## 판정

사용자는 CI 성공 후 병합과 옵션 2 기록 보완, 완전 해결된 하위 이슈의 취소선·종료를 승인했습니다. source/test/workflow나 새 문서의 출력을 변경하지 않았습니다. 기존 35조건의 전쪽 TSV·PNG에서 등록 원본과 범위를 대조했습니다.

| 하위 이슈 | 동일 원본의 전체 결과 | 판정 |
| --- | --- | --- |
| #7734 | HWPX5쪽 Native99.99836%/fresh WASM99.99835%; base71.73078%/71.92415% | 정책제안·사회기여 표와 다음 본문 소속 복구, 문서 반영 뒤 완료 종료 |
| #7783 | HWP157쪽 Native95.44852%/WASM95.44144%; HWPX156쪽94.79496%/94.79506% | HWP 정상 대조군 유지; HWPX base2.04077%/2.00397%에서 개선, 문서 반영 뒤 완료 종료 |
| #7766 |52쪽 Native95.71982%/WASM95.70173% |13/14쪽 흐름은 대응하지만44쪽 캡션 상자·글줄의 수직 차이가 남아 OPEN 유지 |

종료 기준은 전쪽90%·쪽수 일치·본문/셀 소유·관련 정상 회귀·직접 판독입니다. 픽셀 완전 동일을 뜻하지 않으며 미세 자형·래스터 차이는 남습니다. #7734와 #7783만 종료 후보로 결정했습니다. #7743의 기존 이관 회귀·잔여 경계, #7761의 trace 계약, HWPX 미검증인 #7762/#7765, 등록 HWP가 아닌 HWPX 대조군만 있는 #7748, 다른 원본인 #7823 및 이전 기준·회귀 복원 범위가 남은 #7830은 이 결과로 종료하지 않습니다. sample16·편람 제외와 대표 #7445 OPEN 상태를 유지합니다.

## 생산 근거와 전쪽 자료

- 병합: [#8228](https://github.com/edwardkim/rhwp/pull/8228), `3e02fe2749866a8453af3eb3901b9c1a674cccaf`; 최종 CI head `b1e93d8de2d4842f725415aee19bd5b98ecfb8b3`, [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40196953497).
- 로컬 검증 source `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`의 runtime은 최종 CI head와 동일합니다. D86 부모+diff의 정확한 소스와 Native/fresh WASM producer 해시는 [기존 원장](../final-validation-20261025/README.md), 입력/PDF·TSV 해시는 [EVIDENCE](../final-validation-20261025/EVIDENCE.md)를 따릅니다.
- 아래 패널은 기존 전쪽 PNG의 입력·PDF 해시를 다시 확인하고 canonical visual_sweep.py의 compare/overlay/review로 구성했습니다. 새 렌더링·캡처·새 문서가 아니며 full TSV의 원래 raw 값과 PDFium·Print·96 CSSpx·2px 조건을 유지합니다.
- #7734 등록 입력 SHA `2fd531b3e235a714c3fef1a59ddcd9d23225f68f5c492adb17500d23f5cbfe2f`와 정확히 같습니다. 현재 올바른 저장 버전 대조는 기존 `pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf` SHA `798e06db6946edcbb05e7399d3aec08d847a57afc375d9d9e67b435e9e1f40d8`입니다. 과거2024 PDF와 같은 바이트라고 주장하지 않습니다.
- #7783은 등록한 HWP/HWPX 및 각각의 한컴2024/2020 Print를 재사용했습니다. HWP는 이미 base에서 전쪽90% 이상인 정상 대조군이며 HWPX의 전체 흐름 회복과 구분합니다.

| 대상 | Native 전쪽 TSV | fresh WASM 전쪽 TSV |
| --- | --- | --- |
| `task1749x` | [Native](../final-validation-20261025/head/native/task1749x.tsv) | [WASM](../final-validation-20261025/head/wasm/task1749x.tsv) |
| `source8152-d80168` | [Native](../final-validation-20261025/head/native/source8152-d80168.tsv) | [WASM](../final-validation-20261025/head/wasm/source8152-d80168.tsv) |
| `source8152-d80168x` | [Native](../final-validation-20261025/head/native/source8152-d80168x.tsv) | [WASM](../final-validation-20261025/head/wasm/source8152-d80168x.tsv) |
| `source8152-d1853` | [Native](../final-validation-20261025/head/native/source8152-d1853.tsv) | [WASM](../final-validation-20261025/head/wasm/source8152-d1853.tsv) |

## 등록 증상과 대응하는 review·overlay

#7734는4/5쪽 정책제안·사회기여 표와 청정아트·파일럿 프로그램·정책홍보·후속 제목을 확인했습니다. #7783은 HWP6/120/121쪽, HWPX4/6/120/121/137/138쪽의 조문 대비표·마지막 문장·내용 소유·표 경계·뒤 내용을 각각 확인했습니다. #7766의44쪽은 잔여 캡션 위치 차이를 보존합니다.

| 이슈·형식·쪽 | review | overlay |
| --- | --- | --- |
| #7766 `source8152-d1853` native p44 | [review](images/issue7766-source8152-d1853-native-p044-review.png) | [overlay](images/issue7766-source8152-d1853-native-p044-overlay.png) |
| #7783 `source8152-d80168` native p6 | [review](images/issue7783-source8152-d80168-native-p006-review.png) | [overlay](images/issue7783-source8152-d80168-native-p006-overlay.png) |
| #7783 `source8152-d80168` native p120 | [review](images/issue7783-source8152-d80168-native-p120-review.png) | [overlay](images/issue7783-source8152-d80168-native-p120-overlay.png) |
| #7783 `source8152-d80168` native p121 | [review](images/issue7783-source8152-d80168-native-p121-review.png) | [overlay](images/issue7783-source8152-d80168-native-p121-overlay.png) |
| #7783 `source8152-d80168x` native p4 | [review](images/issue7783-source8152-d80168x-native-p004-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p004-overlay.png) |
| #7783 `source8152-d80168x` native p6 | [review](images/issue7783-source8152-d80168x-native-p006-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p006-overlay.png) |
| #7783 `source8152-d80168x` native p120 | [review](images/issue7783-source8152-d80168x-native-p120-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p120-overlay.png) |
| #7783 `source8152-d80168x` native p121 | [review](images/issue7783-source8152-d80168x-native-p121-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p121-overlay.png) |
| #7783 `source8152-d80168x` native p137 | [review](images/issue7783-source8152-d80168x-native-p137-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p137-overlay.png) |
| #7783 `source8152-d80168x` native p138 | [review](images/issue7783-source8152-d80168x-native-p138-review.png) | [overlay](images/issue7783-source8152-d80168x-native-p138-overlay.png) |
| #7766 `source8152-d1853` wasm p44 | [review](images/issue7766-source8152-d1853-wasm-p044-review.png) | [overlay](images/issue7766-source8152-d1853-wasm-p044-overlay.png) |
| #7783 `source8152-d80168` wasm p6 | [review](images/issue7783-source8152-d80168-wasm-p006-review.png) | [overlay](images/issue7783-source8152-d80168-wasm-p006-overlay.png) |
| #7783 `source8152-d80168` wasm p120 | [review](images/issue7783-source8152-d80168-wasm-p120-review.png) | [overlay](images/issue7783-source8152-d80168-wasm-p120-overlay.png) |
| #7783 `source8152-d80168` wasm p121 | [review](images/issue7783-source8152-d80168-wasm-p121-review.png) | [overlay](images/issue7783-source8152-d80168-wasm-p121-overlay.png) |
| #7783 `source8152-d80168x` wasm p4 | [review](images/issue7783-source8152-d80168x-wasm-p004-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p004-overlay.png) |
| #7783 `source8152-d80168x` wasm p6 | [review](images/issue7783-source8152-d80168x-wasm-p006-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p006-overlay.png) |
| #7783 `source8152-d80168x` wasm p120 | [review](images/issue7783-source8152-d80168x-wasm-p120-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p120-overlay.png) |
| #7783 `source8152-d80168x` wasm p121 | [review](images/issue7783-source8152-d80168x-wasm-p121-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p121-overlay.png) |
| #7783 `source8152-d80168x` wasm p137 | [review](images/issue7783-source8152-d80168x-wasm-p137-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p137-overlay.png) |
| #7783 `source8152-d80168x` wasm p138 | [review](images/issue7783-source8152-d80168x-wasm-p138-review.png) | [overlay](images/issue7783-source8152-d80168x-wasm-p138-overlay.png) |
| #7734 `task1749x` native p4 | [review](images/issue7734-task1749x-native-p004-review.png) | [overlay](images/issue7734-task1749x-native-p004-overlay.png) |
| #7734 `task1749x` native p5 | [review](images/issue7734-task1749x-native-p005-review.png) | [overlay](images/issue7734-task1749x-native-p005-overlay.png) |
| #7734 `task1749x` wasm p4 | [review](images/issue7734-task1749x-wasm-p004-review.png) | [overlay](images/issue7734-task1749x-wasm-p004-overlay.png) |
| #7734 `task1749x` wasm p5 | [review](images/issue7734-task1749x-wasm-p005-review.png) | [overlay](images/issue7734-task1749x-wasm-p005-overlay.png) |

## 실제 실행된 회귀

원래 전체 nextest11,481 PASS 로그에서 다음 검사의 실제 PASS를 대조했습니다. 인위적으로 과거 잘못된 컷 숫자를 복원하지 않고 source 소유·끝 줄·원래 여백·물리 프레임의 관계를 보호합니다.

### `issue_1749_synthetic_cell_spacing` — 2 PASS

- `rhwp::regression_suite_017 issue_1749_synthetic_cell_spacing::centered_synthetic_stack_owns_first_paragraph_before_space_once`
- `rhwp::regression_suite_017 issue_1749_synthetic_cell_spacing::opening_rowbreak_fragment_keeps_source_first_paragraph_before_space`

### `issue_1891_legacy_hwpx_current_frame` — 8 PASS

- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::a_blank_cell_line_stays_with_the_closing_frame_and_the_next_title_restarts`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::a_visible_tail_line_cannot_start_outside_its_body_page`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::adjustable_single_row_uses_current_lines_and_source_inner_padding`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::completed_table_edges_follow_independent_print_within_two_css_pixels`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::legacy_latin_without_a_true_type_substitute_keeps_its_source_dot_boundary`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::nested_current_height_and_following_empty_line_share_the_parent_frame`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::previous_margin_cache_does_not_move_the_last_sentence_to_another_page`
- `rhwp::regression_suite_011 issue_1891_legacy_hwpx_current_frame::the_last_small_row_stays_before_the_nested_explanation_continuation`

### `regulatory_analysis_completed_pages` — 9 PASS

- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::absolute_nested_table_keeps_width_and_text_inside_owner_on_completed_page139`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::bold_decoration_preserves_hancom_line_partition_on_completed_page133`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::continuation_keeps_final_row_frame_and_plan_on_completed_page90`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::complete_wrapper_preserves_following_blank_heading_and_table_on_completed_page93`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::carried_rowspan_label_keeps_its_print_center_and_page_owner`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::empty_line_cut_keeps_amendment_and_existing_label_aligned_on_completed_page109`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::merged_grid_shares_declared_column_boundaries_on_completed_page17`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::single_cell_opening_fragment_preserves_successor_sentence_owner`
- `rhwp::regression_suite_018 regulatory_analysis_completed_pages::partial_row_cut_keeps_independent_physical_frame_on_completed_page121`

### `issue_7412_cell_content_width_quantum` — 2 PASS

- `rhwp::regression_suite_016 issue_7412_cell_content_width_quantum::issue_7412_cell_reflow_breaks_on_the_4hu_grid_hwp`
- `rhwp::regression_suite_016 issue_7412_cell_content_width_quantum::issue_7412_cell_reflow_breaks_on_the_4hu_grid_hwpx`

## 새 보존 PNG의 SHA-256·바이트

| 경로 | SHA-256 | 바이트 |
| --- | --- | ---: |
| `images/issue7766-source8152-d1853-native-p044-review.png` | `36e1110f993589c5cdd1b60ab2867dbfbf80ca7dc2ffd832ad288143b736d85e` | 549446 |
| `images/issue7766-source8152-d1853-native-p044-overlay.png` | `eb75b59133d135de191ed45b62ef8d39b168fc17f4e6235e3745ef88f941f8ca` | 178760 |
| `images/issue7783-source8152-d80168-native-p006-review.png` | `db8dd4b5052772c148a4b7bb49e2835954d7291b3453db5c1e5722c8de707aaa` | 694083 |
| `images/issue7783-source8152-d80168-native-p006-overlay.png` | `d927b625ef0466f8250b901c49218c5c1638e19511b48dc1f6f9c5ba9e0cd724` | 234338 |
| `images/issue7783-source8152-d80168-native-p120-review.png` | `3e5610e392c0259f7d5caf4c0364d2098aff768f5cdf9eee926701ac204a839d` | 533489 |
| `images/issue7783-source8152-d80168-native-p120-overlay.png` | `2b32eb165a2736927fa8920cf91b4d73f42843d55cdf4e17e16e94cc48aefd4f` | 174891 |
| `images/issue7783-source8152-d80168-native-p121-review.png` | `da63cc21e6eda8670552bb2e77bffcd0f56e3352e3e9714f0a13643290fb0087` | 642546 |
| `images/issue7783-source8152-d80168-native-p121-overlay.png` | `7d668bf6b14282573d7140d15d36e8f12d619969295d103be6d9e1f5a00e5757` | 209885 |
| `images/issue7783-source8152-d80168x-native-p004-review.png` | `dc96eac2acf55c77445a4b4cbe72957b43e964d1bdf4ca930864e6d3cbc6f74b` | 461131 |
| `images/issue7783-source8152-d80168x-native-p004-overlay.png` | `49a339dfa8c98f5536f98fa8211d6717a4b4dba06ff09aff9040643307d45a46` | 155275 |
| `images/issue7783-source8152-d80168x-native-p006-review.png` | `5fe4bff8872bc78a3c2cf0044ea81e72f4955135065f5695d09add4556fdc670` | 142274 |
| `images/issue7783-source8152-d80168x-native-p006-overlay.png` | `e7beee57ada9832553d5bde3a099444734c7908a4749a97f2db971fc1d765576` | 61844 |
| `images/issue7783-source8152-d80168x-native-p120-review.png` | `386093f63ce860cb487f2de017044999825494231b628f8809c278317e961cd7` | 650344 |
| `images/issue7783-source8152-d80168x-native-p120-overlay.png` | `5545e8d74370be0bc8f01c8f5f9ef8441aacd90314026d8ad7c621c1e7f8cf99` | 220602 |
| `images/issue7783-source8152-d80168x-native-p121-review.png` | `261dd91c39f95a91865163245126863e600b585e1d186926f559df5ae34a0d8b` | 540396 |
| `images/issue7783-source8152-d80168x-native-p121-overlay.png` | `8c658084c7eb57dfa1728b024df901bd51c4f46dc71e659032c8830aa00fc179` | 185833 |
| `images/issue7783-source8152-d80168x-native-p137-review.png` | `5e0b58b2b6b56101d4a657507fc10bb936a96da3e7ac30c834dbde909f343958` | 537909 |
| `images/issue7783-source8152-d80168x-native-p137-overlay.png` | `946c37e14f3a0b126cfda56626bc56cf057799a1d563a5023ba75ca5702f4e6a` | 182829 |
| `images/issue7783-source8152-d80168x-native-p138-review.png` | `879dd083d96bfdcd85771395d5d070385fd97b8b9301f5727f1e2510f628e7dd` | 111242 |
| `images/issue7783-source8152-d80168x-native-p138-overlay.png` | `51cb5f5bfa7eb0f879ae0571a490808412062f2baf2e9bfd4b71a0419d155792` | 50873 |
| `images/issue7766-source8152-d1853-wasm-p044-review.png` | `71989b4dc45dab9ec287bfcd61824f252c59e5bf0cdde4795dd04de42ab22ee0` | 547462 |
| `images/issue7766-source8152-d1853-wasm-p044-overlay.png` | `b92682656c5494cce31a92e47493ec3d5b9f6f6272fddd0d035ec0ce15a8b39a` | 177718 |
| `images/issue7783-source8152-d80168-wasm-p006-review.png` | `7602eacb2dae0f482533cee8b5d71ffa32c5dea0bb7cf7981b75189514109d78` | 690917 |
| `images/issue7783-source8152-d80168-wasm-p006-overlay.png` | `ecb19482db63e2717ec322c0a775d4956c4640937cd483af7b29e784f7bcb306` | 231203 |
| `images/issue7783-source8152-d80168-wasm-p120-review.png` | `c64c914a13481d0c7e2dbbbfede2fe31c49f465bd3a917fb0d301ede9330c4c9` | 532125 |
| `images/issue7783-source8152-d80168-wasm-p120-overlay.png` | `e4bdfdf90cc5f1b7d308dbb9c346efc9513b8dcb8fd3d76d64aa29a2ae1fda83` | 173217 |
| `images/issue7783-source8152-d80168-wasm-p121-review.png` | `24e1071cf88ed62a18c25ebe212b5469d44a109dd30b7b420a387a87e456a0dc` | 638858 |
| `images/issue7783-source8152-d80168-wasm-p121-overlay.png` | `5ee18096c5acea53f2ffaddab7d828d1482f6237d21c4d498914f2537687c956` | 207388 |
| `images/issue7783-source8152-d80168x-wasm-p004-review.png` | `f2179dc0c0a9db0527a16915c4bfa676767e7ad0b6d1a2561655734672e88010` | 461414 |
| `images/issue7783-source8152-d80168x-wasm-p004-overlay.png` | `0198caca8accd086f8432185e5174e2e8a1ec4364a34ea5295aea5c6f962e7cb` | 155378 |
| `images/issue7783-source8152-d80168x-wasm-p006-review.png` | `677849a975b90071f9a48fff74a400a0096178a02f8d012c649255c82e2e4e20` | 142272 |
| `images/issue7783-source8152-d80168x-wasm-p006-overlay.png` | `729d8bccbf44b068058ffe6ee91249b5739ac8c42fe5c1e1c5a8e4e4316b5ef4` | 61844 |
| `images/issue7783-source8152-d80168x-wasm-p120-review.png` | `6b7151ded60260b99c3a32d8cb9d43e326eaef7118da6f358bad2b26580cc95d` | 647418 |
| `images/issue7783-source8152-d80168x-wasm-p120-overlay.png` | `beb5812949092d920e84700a4dad94680010dadcaada1e1e79fca19c024fe76a` | 218349 |
| `images/issue7783-source8152-d80168x-wasm-p121-review.png` | `785d8ada1fbbc60fdaa4f9bb2cf7b102c9c4c50aaa586f90fa82ff1a93d76980` | 532830 |
| `images/issue7783-source8152-d80168x-wasm-p121-overlay.png` | `6343f4089ac44870b771b9f2713f1222cdc22e36d184ccfcc5d6ef32816896c7` | 182608 |
| `images/issue7783-source8152-d80168x-wasm-p137-review.png` | `891094e7c821eddc67af47f236e42ab04f91f6dd608d393fdf9a8a9217a17640` | 536686 |
| `images/issue7783-source8152-d80168x-wasm-p137-overlay.png` | `28c840533e4339c6be8393d928eb0c6c9b839ca695d8166f10a000b11d68490a` | 181580 |
| `images/issue7783-source8152-d80168x-wasm-p138-review.png` | `178d7691d4b06189f8853e6c75246cd9a3cd59b60df39476e783bce29ffb7622` | 111223 |
| `images/issue7783-source8152-d80168x-wasm-p138-overlay.png` | `6a7b12cda4005bbbc1831d65ed8a833f67c28fe5ac8dd71bb3dea947cc2e5a5d` | 50916 |
| `images/issue7734-task1749x-native-p004-review.png` | `6592962791f80fff8af85a227c3ea006389e5fe232e59160018d9da6fcd79431` | 504403 |
| `images/issue7734-task1749x-native-p004-overlay.png` | `f18ae841113f301563ee7dcc1e2f43dd4a97aa13962985dc1a7617ff5aa58a82` | 172545 |
| `images/issue7734-task1749x-native-p005-review.png` | `e1acaec838b925b29e11b658d21e394540c1c9726777d2b21b8771cc5c214bc6` | 469360 |
| `images/issue7734-task1749x-native-p005-overlay.png` | `c063169622c888a55423f6d3cbe89c4b8fff36fbba8f69ff753f54a1329e60a4` | 159309 |
| `images/issue7734-task1749x-wasm-p004-review.png` | `a464cafad6581d1a258a6bc9d2ab881343dcbf83269efcc64613958ed4f2a666` | 503377 |
| `images/issue7734-task1749x-wasm-p004-overlay.png` | `31d72f86fe646145ea74396a02d2db4a4c6b738918a1b84879eea69c668950cd` | 171202 |
| `images/issue7734-task1749x-wasm-p005-review.png` | `92e87b6cc598394d4b7dfe8dcc1800dfe6984a8a073526fd044eaa21f36f3a53` | 467351 |
| `images/issue7734-task1749x-wasm-p005-overlay.png` | `aab07e8d196991c09aaca8f53eecd8af61cd3e5cb2e38da4e409c96a7c951635` | 157709 |

- 병합 후 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/40200061667)은 `successful-pr-worker-measurements`, source run40196953497/attempt1, B·C·D artifact `12601231532`·`12601517458`·`12601121704`를 확인하고 metrics commit `56537b8ce5162d47b9679f6355aef0044937eb2b`로 발행했습니다. 검증 CI를 병합 뒤 재실행하지 않았습니다.
