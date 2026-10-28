---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-10
---

# #7700 위임전결규정 원본과 부적합한 기존 회귀의 재현 자료

[Issue #7700](https://github.com/edwardkim/rhwp/issues/7700) · 담당 jangster77. 현재 **미해결/시각 미달**이며 이 자료를 통합 PR의 정상 fixture·새 baseline으로 수용하지 않는다.

## 현재 단계

원본·독립 Print·선택 PNG·검사 본문을 바이트 동일하게 보존하고, 활성 `samples/`·일반 PDF 경로와 부적합한 검사 두 개에서 제거했다. Base 공통 103쪽 대조는 source 불변/exit 1로 종료했고 현재 미달 4쪽은 모두 base에서도 미달이다. 이관 자체는 출력 결함의 해결이나 시각 통과가 아니다. 다른 정상 문서와 같은 원본의 parser/serializer·IR 보존 계약은 유지했다. 제거 범위의 세 Clippy·workspace build·정책 검사와 관련 nextest 31개는 source 불변/exit 0이며, 최종 전체 검증과는 구분한다.

## Source와 독립 기준

- 실제 base code: `21ed64071f81f35f37618717693bd30f727392c9`, 새 Native CLI SHA-256 `b5fa531bdf40ce2b80692b11d9451b1f96ab2579260f644ddbffdff3ac5a98e0`.
- 현재 code: `9b53bd699126205b7a93fff326167f575252f1ea`에 커밋한 v86 retry2 코드. 실제 빌드는 부모 `3874bc4be037e84998d548e7755bc689af9d871b` 위 두 Rust 파일의 당시 미커밋 변경에서 수행했고 커밋 뒤 production source와 동일함을 확인했다. Native CLI SHA-256 `a41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2`. 실제 source hash와 build log는 ignored `native-build-accepted-interior-row-ledger-batch-v86-retry2-exit.json`에 있다.
- 원본은 base commit blob과 실행 입력이 바이트 동일하다. Print는 같은 원본에서 실제 한컴2020 runtime11.0.0.9136/PrintMethod0/hancom2020_pdf_driver_one_up, job `6bef4e6b-8a06-463e-8c09-539211ad7a25` success로 생성한103쪽 PDF다.
- Source와 PDF 파일명·bytes·SHA는 아래 표에 고정한다. 이미 로컬 커밋에 포함된 Print의 대응을 확인했고 새 기준/다른 정상 입력으로 교체하지 않는다.

## 현재 관측

| 범위 | Base Native | 현재 Native | 독립 Print |
| --- | --- | --- | --- |
| 전체 쪽수 | 105 | 103 | 103 |
| 12쪽 | 71.46986% | 71.48133% | 같은 Print |
| 19쪽 | 71.75853% | 74.28775% | 같은 Print |
| 27쪽 | 67.71964% | 63.15748% | 같은 Print |
| 59쪽 | 47.28116% | 87.48894% | 같은 Print |

현재 Native 전103쪽 TSV는4쪽 미달·최저27쪽63.15748%·누락0/쪽수일치다. Base Native105쪽 전체 시도는 Print103쪽과 수가 달라 TSV 전단계에서 실패했고, 공통1–103쪽 재측정도 완료했다. Base fresh WASM은 미실행이다. 이 관측을 whole fidelity 승인이나 현재 수정의 신규 회귀 면제로 쓰지 않는다.

12쪽은 첫 `조정ㆍ시행`행의 쪽 소속과 뒤 행 경계가 다르다.27쪽은 첫 `주요통계관리`가 앞쪽으로 넘어가지 않고 남아 후속 행이 아래로 밀린다.19쪽은 물품관리 후반·입찰/계약 행 높이와 병합 라벨 정렬이 다르고59쪽은 뒤 표의 일부 행·두번째 행 하단 높이가 다르다. 직접 review에서 확인했으며 글꼴 차이로 면제하지 않는다.

## 기존 검사 보존과 복원 조건

기존 #6024 함수는105쪽과8/9·66/67쪽을 고정했고, #5679 함수는11쪽·451.8px 및8/14px 한계를 고정한다. 변경 전후 source 원문은 `.rs.txt`로 보존한다. 현재 branch가 #6024 assertion을103쪽+내용 보존으로 바꾼 것 역시 전쪽90% 전의 정식 회귀 복원으로 승인하지 않는다.

이슈에서 같은 원본·Print의 관련 전체 Native/fresh WASM 최저90% 이상, 쪽수/소유·누락/중복·표와 뒤 내용·수식/미세 간격의 확대 판독을 확인한 뒤 관계형 검사를 복원한다. 현재 출력값에 pin·공차를 맞추거나 ignore/조용한return으로 숨기지 않는다. 이관은 제품 결함 해결 또는 이슈 종료가 아니다.

### 활성 경로에서 제거한 범위

- `issue_6024_band_continuation_does_not_redraw_merged_cell_content`와 `issue_5679_justified_trailing_space_does_not_take_word_distribution`: 정식 검사 두 개를 제거하고 변경 전후 본문을 아래 `.rs.txt`에 보존했다.
- 이전 `samples/issue5679/10857_delegation_rules.hwp`와 일반 PDF 경로의 대응 Print: 아래 이슈 자산 경로로 옮겼다. 바이트·SHA는 유지한다.
- `body_overflow_baseline.tsv`의 해당 원본 2건 행, `text_overlap_baseline.tsv`의 1건 행 및 일반 PDF `references.tsv`의 대응 행만 제거했다. 다른 원본의 기준값은 변경하지 않았다.
- `tests/common/preserved_format_inputs.rs`를 통해 HWP5 roundtrip 및 IR field sweep은 이 자산의 같은 바이트를 원래 논리 이름으로 계속 읽는다. IR baseline의 해당 3206건 기록은 그대로 유지한다. 이 경로는 렌더링 원본 자동 수집에 연결하지 않는다.

`#[ignore]`, 제외 조건 추가, 누락 시 조용한 return은 사용하지 않았다. 이 단계의 정식 테스트 감소는 두 개다. 파생 suite 준비 후 고정 base 대비 manifest/unit-tier 정책, fmt, 세 Clippy 및 workspace build를 모두 통과했다. `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast -E 'test(hwp5_roundtrip_baseline::) | test(ir_field_sweep_baseline::) | test(issue_6803_split_start_row_rowspan_end_cut::) | test(issue_6935)'`는 **31 PASS / 0 FAIL / 10351 skip**, 실제 검사 168.261초(2 slow)·terminal exit 0이다. 이 원본의 HWP5 roundtrip과 IR 검사가 이관 자산에서도 계속 실행됐다. 이 진단 선택 범위가 최종 전체 nextest나 모든 PR의 시각 검증을 대신하지 않는다. 전체 source hash·각 명령·시간·exit는 ignored `issue7700-relocation-validation-v90.json`에 있다.

## 파일과 SHA-256

| 보존 파일 | 이전/실행 경로 | SHA-256 |
| --- | --- | --- |
| [10857_delegation_rules.hwp](10857_delegation_rules.hwp) | `samples/issue5679/10857_delegation_rules.hwp` | `de43607f05072840097b19a294b4455905210402cc42823545125e03567df881` |
| [10857_delegation_rules-hancom2020-print.pdf](10857_delegation_rules-hancom2020-print.pdf) | `pdf/semanticist21-20261008/issue5679-10857_delegation_rules-de43607f0507-2020.pdf` | `90eda86b2b4dfeabc601cd9b583408c6528a9efabf4e626fd6f2e28e3bcb2b97` |
| [base21ed-native-review-012.png](base21ed-native-review-012.png) | `output/pr-review/semanticist21-20261008/native-upstream-base-6024-boundaries-v87/issue6024/review/review_012.png` | `b44984478e94a67b158a92dd4483c3d65a3722f11b9f079284ba5d32dc36590e` |
| [current-v86-native-review-012.png](current-v86-native-review-012.png) | `output/pr-review/semanticist21-20261008/native-logical-physical-row-boundaries-v86-retry2/issue6024/review/review_012.png` | `70d9d61fc35bf48daa620f9749f8c8e6e39b04db20037db11fad9b209c380d1c` |
| [current-v86-native-review-027.png](current-v86-native-review-027.png) | `output/pr-review/semanticist21-20261008/native-current-6024-full-failed-pages-review-v89/review/review_027.png` | `7d9ab9f2fa644162bf75268c17050a116b13d20170c511dd9cea11710bdff54f` |
| [current-v86-native-overlay-027.png](current-v86-native-overlay-027.png) | `output/pr-review/semanticist21-20261008/native-current-6024-full-failed-pages-review-v89/overlay/overlay_027.png` | `8e11922f059782f634de53296a52f795e025d4bcec608c0da983cd7308b72395` |
| [issue_6024_rowspan_continuation_blank.rs.base.rs.txt](issue_6024_rowspan_continuation_blank.rs.base.rs.txt) | `output/pr-review/semanticist21-20261008/suitability-test-source-v88/issue_6024_rowspan_continuation_blank.rs.base.rs.txt` | `ad62dd04813ccadb143bcf8e4d7efd16f1ca896eafca05226433f9fdb8738863` |
| [issue_6024_rowspan_continuation_blank.rs.current.rs.txt](issue_6024_rowspan_continuation_blank.rs.current.rs.txt) | `output/pr-review/semanticist21-20261008/suitability-test-source-v88/issue_6024_rowspan_continuation_blank.rs.current.rs.txt` | `e078db85ac2444dc98d4eb4b42c00336c9723f5d05797f15d5b25e5f29d91070` |
| [issue_5679_justify_trailing_space_width.rs.base.rs.txt](issue_5679_justify_trailing_space_width.rs.base.rs.txt) | `output/pr-review/semanticist21-20261008/suitability-test-source-v88/issue_5679_justify_trailing_space_width.rs.base.rs.txt` | `aea529ed6e87914b5ff6b10864f0bf4c688eb729612ef923cacdd82a15a50d4f` |
| [issue_5679_justify_trailing_space_width.rs.current.rs.txt](issue_5679_justify_trailing_space_width.rs.current.rs.txt) | `output/pr-review/semanticist21-20261008/suitability-test-source-v88/issue_5679_justify_trailing_space_width.rs.current.rs.txt` | `aea529ed6e87914b5ff6b10864f0bf4c688eb729612ef923cacdd82a15a50d4f` |

TSV·로그·중간 JSON은 ignored `output/pr-review/semanticist21-20261008/`에만 보존한다. 대표 PNG와 원본·PDF의 공개 URL은 검증 후 remote에 게시한 실제 SHA로 고정한다. 아직 미게시 URL을 실제 증적 링크로 표시하지 않는다.

## 고정 base 전체 대조 종료

Base 공통103쪽 중 81쪽 미달, 최저50쪽8.62142%이며base105대Print103쪽차이는유지한다. 현재전103쪽은4쪽미달·최저27쪽63.15748%이고base90% 이상→current미달인새threshold crossing은0쪽이다. 일부선택쪽의고득점이나전체개선으로남은미달을면제하지않는다.

27쪽은두출력모두`주요통계관리`가앞쪽으로넘어가지않은기존소속결함이다.현재상단96.4px는Print96.3747px와대응하고base94.5px보다정확하다.표높이339.1px는base/current같다.19쪽전체표높이는base920.9→current929.1px로Print928.744px에접근했지만내부행차이는남는다.59쪽의base단일앞표소속이current앞표+보호보상정책과표로바뀌어Print에가까워졌으나일부행높이는여전히미달이다.점수하락만으로별도회귀를면제하지않고같은원본의직접경계·소속을판독했다.

실제TSV/PNG/hash/명령/판독은ignored `native-base-current-6024-full-comparison-v89.json`과양쪽canonical run에있다.로그/TSV/중간JSON은이asset폴더에커밋하지않는다.
