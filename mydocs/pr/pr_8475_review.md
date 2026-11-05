# PR #8475 리뷰 — fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)

## 최종 판정

**머지 보류 — 메인터너 보정 진행**

2026-11-05 추가 직접 판독에서 `vertical_fit.hwpx` 5쪽의 표 한 쪽 소속은 확인했으나,
병합 칸 `임용전경력`의 줄 나눔이 독립 Print와 다르고 마지막 글자가 보이지 않았습니다.
Native/fresh WASM 양쪽에서 확인했으며 98%대 점수만으로 면제하지 않습니다.
[소유권·합성 줄 경로 분석](../working/task_m100_8470_vertical_synthetic_frame.md)에 따라
이 주장 칸의 글자 보존은 최신 Native/fresh WASM에서 보정·재검증했습니다. 남은 통합 공통 검증을 마친 뒤 최종 판정을 갱신합니다.

별도 보정 소스 `37e5bba300`에서 [같은 관계 검사의 수정 전후 실행](assets/planet6897_20261103/pr_8475/vertical-axis-focused-37e5bba300.md)을 확인했습니다.
검출 assertion을 바꾸지 않고 수정 전 2 FAIL에서 수정 후 2 PASS로 바뀌었으며,
기존 표 나눔 4개와 저장 세로줄 정상 대조 1개도 최소 글꼴 환경에서 통과했습니다.
이 소스는 별도 검증 worktree에 있으며 아직 통합 브랜치의 제품 코드에 적용하지 않았습니다.
Native/fresh WASM 전쪽·확대 판독과 최종 공통 검증 전까지 보류를 유지합니다.

메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 최신 리베이스 Native·fresh WASM 확인

최신 제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`에서 두 문제 입력은
**Native/fresh WASM 각각 전5쪽 100.00%**다. 마지막 `력`까지 칸 내부 한 세로줄에 보존됨을
[동일 입력·5쪽의 양쪽 review와 별도 overlay](assets/planet6897_20261103/rebase8609/vertical-cell-claims.md)에서
실제로 확인했다. 문제 글자 소실의 보정 검증은 완료했으며 남은 조건은 통합 공통 검증이다.

정상 저장 줄·실물·개요·표 경계까지 포함한 [Native 10입력·43쪽](assets/planet6897_20261103/rebase8609/native-own-validation.md)과
[fresh WASM 10입력·43쪽](assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 모두
90% 이상이며 직전 후보 대비 정밀 raw 0.1%p 초과 하락 0쪽이다.
새 base 자체 대조와 전체 공통 검증 전에는 최종 수용으로 바꾸지 않는다.

## 세로쓰기 글자 보존 보정의 현재 증거

보정 소스 `37e5bba3007614d9304c3dbf863503a8c7340405`의 Native 제품을 별도로
빌드해 실제 컴파일 의존성에 새 공통 helper가 포함됐음을 확인했습니다.
[제품·명령·SHA와 전쪽 대조](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/README.md),
[주장 쪽 직접 판독](assets/planet6897_20261103/direct-reading-current.md)을 보존했습니다.

| 입력 | 보정 후 Native 전쪽 TSV · 최저 raw | 직접 확인한 주장 쪽 |
| --- | --- | --- |
| `vertical_fit.hwpx` · 5쪽 | [TSV](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-silhouette.tsv) · 100.00% | 5쪽 [review](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-p005-review.png) / [overlay](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-p005-overlay.png) |
| `vertical_oversize.hwpx` · 5쪽 | [TSV](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-silhouette.tsv) · 100.00% | 5쪽 [review](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-p005-review.png) / [overlay](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-p005-overlay.png) |
| `42565_입법예고문.hwp` · 16쪽 | [TSV](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-silhouette.tsv) · 94.92448% | 10쪽 [review](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p010-review.png) / [overlay](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p010-overlay.png), 11쪽 [review](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p011-review.png) / [overlay](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p011-overlay.png) |

두 축약 입력의 5쪽에서 `임용전경력` 다섯 글자가 한 세로줄의 셀 안에 표시되며, 마지막
글자의 소실이 복원됐습니다. 앞뒤 행·표 끝·쪽 소속을 함께 확인했습니다. 정상 대조군을
포함한 6입력·29쪽 중 27쪽은 이전 `c183a1ac61` 출력과 PNG 바이트가 동일하고,
두 문제 쪽만 개선됐습니다. 원본의 유효한 저장 세로줄은 유지합니다.

실물 10쪽의 기존 저장 세로줄 글자 간격과 정상 대조군의 연결 화살표 등에는 이전부터
Print와 차이가 남습니다. 해당 페이지 PNG가 직전 후보와 동일함을 대조했으며, 이번
합성 줄의 누락 보정을 원본 전체의 완전 일치로 확대하지 않습니다.

[같은 보정 소스의 fresh WASM 전쪽 TSV](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/wasm/README.md)도
6입력·29쪽을 완료했습니다. 두 문제 입력의 각 5쪽은 모두 100.00%이며, Native와 마찬가지로
두 문제 쪽만 바뀌고 나머지 27쪽의 PNG는 해당 경로의 직전 후보와 같습니다.
두 문제 쪽·실물 10/11쪽·저장 줄 정상 대조 1쪽의 Native/WASM review와 별도
overlay를 모두 직접 확인했습니다. 필수 전체 Rust 묶음과 통합 브랜치 적용·최신 base
확인은 아직 완료 전입니다.
아래 이전 producer의 TSV·PNG는 보정 전 증거로 유지합니다.

## 필수 편람·환경 복구 결과

[보정 소스의 Native 편람 383쪽 게이트](assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/handbook-validation.md)는
최저 99.96%, 기준 표시값 하락 0쪽·fallback 0건·exit 0으로 완료했습니다. 첫 실행의
ENOSPC는 성공으로 집계하지 않았으며 동일 제품·완료 구간을 검증해 재사용한 복구 결과입니다.

같은 코드에서 doctest의 동시 링크만 1개로 제한한 재검증은 8 PASS·3 IGNORE·exit 0입니다.
Rust lint와 workspace 빌드도 완료했습니다. 전체 nextest·Skia·CI 통합 모듈은 이어 실행 중이며,
통합 브랜치 적용과 최신 base 확인까지 끝나기 전에는 최종 수용 판정을 바꾸지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8475](https://github.com/edwardkim/rhwp/pull/8475) · `planet6897` → `devel` |
| 원 head | `c01f286aca82a191bca56e3df89defddeeda81a0` |
| 통합 적용 커밋 | `b8dfcb09b48e6bd56a15df3ab691b2fa85857a31, 4a12af9ea34b0a7e2641927aadeab3e139f466a7` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8470](https://github.com/edwardkim/rhwp/issues/8470) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 보정 전 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [vertical_fit.hwpx](../../samples/issue8470/vertical_fit.hwpx) · [vertical_fit-2020.pdf](../../pdf/issue8470/vertical_fit-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-native.tsv) · 98.75% (5쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-wasm.tsv) · 98.72% (5쪽) |
| [vertical_oversize.hwpx](../../samples/issue8470/vertical_oversize.hwpx) · [vertical_oversize-2020.pdf](../../pdf/issue8470/vertical_oversize-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-native.tsv) · 98.94% (5쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-fresh-wasm.tsv) · 98.92% (5쪽) |
| [42565_입법예고문.hwp](../../samples/issue8470/42565_입법예고문.hwp) · [42565_입법예고문-2020.pdf](../../pdf/issue8470/42565_입법예고문-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-native.tsv) · 94.92% (16쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-wasm.tsv) · 96.96% (16쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

세로쓰기 칸을 가진 표는 저장 page_break 값을 보존하면서 조판에서는 나누지 않는 표로 취급한다.

model/table.rs의 Table::layout_page_break가 측정·typeset·layout 공통 소비 지점이다. has_vertical_cell 캐시는 셀 구성·글 방향 편집 뒤 무효화하며 저장 attr 자체는 바꾸지 않는다.

작은 표·본문보다 긴 표·걸침 위치가 다른 표를 독립 Print와 대조한다. 가로쓰기 표의 기존 분할과 편집·저장 왕복을 대조군으로 유지한다. 추가한 42565 실물은 devel 17쪽·본문 넘침 4건에서 Print와 같은 16쪽·5건으로 바뀐다. 추가 1건은 10쪽의 한 쪽 세로쓰기 표이며 Native 99.26%·fresh WASM 99.25%와 직접 판독으로 확인했다. 기존 기준선 행·공차는 유지하고 신규 입력의 정상 관측값 5건을 등록한다. [본문 경계와 독립 Print 대조 근거](assets/planet6897_20261103/body-overflow-42565-proof.md)를 연결한다.

관련 정식 회귀 파일: [issue_8470_vertical_cell_table_no_split.rs](../../tests/cases/issue_8470_vertical_cell_table_no_split.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 보정 전 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `vertical_fit.hwpx` 5쪽 · 세로쓰기 표 경계 | 98.75% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-overlay.png) | 98.72% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-overlay.png) |
| `vertical_oversize.hwpx` 5쪽 · 세로쓰기 넘침 경계 | 98.94% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-overlay.png) | 98.92% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-overlay.png) |
| `42565_입법예고문.hwp` 5쪽 · 구역 첫쪽 표 위 간격 | 99.27% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-overlay.png) | 99.24% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-overlay.png) |
| `42565_입법예고문.hwp` 10쪽 · 세로쓰기 표 한쪽 수용 | 99.26% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-overlay.png) | 99.25% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-overlay.png) |
| `42565_입법예고문.hwp` 11쪽 · 병합 칸 끝 행 | 100.00% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-overlay.png) |
| `42565_입법예고문.hwp` 13쪽 · 구역 첫쪽 표 위 간격 | 99.88% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-overlay.png) | 99.84% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-overlay.png) |

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->

## 최신 리베이스 이후 통합 검증 상태

최신 base는 `f529dc92d4d172bf32390d7749743a45d9e41bfc`, 검증 시작 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`다. 원 PR 39개와 메인터너 보정을 담은
168개 커밋을 충돌 0건으로 리베이스했고 range-diff 전 행이 `=`로 대응했다.
[원 head와 현재 적용 SHA](assets/planet6897_20261103/integration-commits.md)를 구분한다.

최신 source의 최소 글꼴 Rust·Native Skia·lint·최적화 WASM 및 영향 경계 검증은 진행 중이다.
완료 판정이나 최종 승인으로 표시하지 않는다. 검증 중 소스 해시를 고정하고 문서 갱신과 구분한다.
충돌 없는 리베이스 뒤 기존 전쪽 비교를 반복하지 않는 작업지시자의 범위를 따른다.
새 base 기능은 같은 시스템 글꼴 환경의 실제 base/candidate 출력과 영향 경계로 대조한다.
새 base 대비 허용 하락은 0.1%p이며, 보호 PR #8389·#8397은 원 TSV의 소수 둘째 자리
표시값 하락을 허용하지 않는다. 기존 TSV·PNG의 producer는 원래 출처대로 유지한다.

직전 c183 제품에서는 [최소 글꼴 Rust 12,563 PASS와 필수 40개 실행](assets/planet6897_20261103/rust-validation-c183a1ac61.md),
[Native 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/native-validation.md),
[fresh WASM 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md),
[실제 브라우저/API 실행](assets/planet6897_20261103/runtime-validation-c183a1ac61.md)을 완료했다.
양쪽 전쪽 최저는 각각 92.45232%·92.87288%다. 이 결과를 최신 리베이스 제품의 재출력으로
취급하지 않는다. 보호 범위 Native는 17문서·1,770쪽·원 TSV 2,153행의 하락·누락 0행이며,
같은 c183 WASM 보호 범위의 남은 집계를 진행 중이다. [공식 증거 목록](assets/planet6897_20261103/README.md)과
[재실행 명령·입력·Print PDF·조판 세대·글꼴 계약](assets/planet6897_20261103/visual-reproduction.md)을 연결한다.

원 PR별 주장 쪽의 직접 판독·회귀 주석·최종 본문 감사를 마친 뒤 수용 판정을 확정한다.
통합 PR 생성·push·원격 review·comment·close는 아직 수행하지 않았다. PR 생성 전 승인을 기다린다.

## 실제 Studio 출력의 소비 확인

최신 WASM을 실제 Studio의 Canvas2D·CanvasKit compat/default software에 연결한 screen/print 12개 캡처는 실제 backend 불일치·fallback·replay hard failure 0건, exit 0입니다.
[같은 두 원본의 5쪽 review·별도 overlay와 실제 화면](assets/planet6897_20261103/rebase8609/studio-consumers.md)을 직접 확인했습니다. print 4개 영역의 raw는 99.99761~100%이고 마지막 글자와 셀 내부 포함·행 순서·표 종료를 보존합니다.
엄격 RGB parity의 보고용 차이는 남아 있으며 점수와 별도로 기록했습니다. 설치 글꼴 바이트는 로컬 host API로만 공급하고 배포하지 않았습니다.
