# PR #8475 리뷰 — fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)

## 최종 판정

**메인터너 보정 후 수용 가능**

초기 직접 판독에서 발견한 `임용전경력`의 마지막 글자 소실은 공통 글줄 축·합성 줄 보정으로 복구했습니다.
현재 통합 후보에서 두 문제 입력의 Native/fresh WASM 전 5쪽이 각각 100.00%이며,
같은 두 쪽의 실제 Studio Canvas2D·CanvasKit 화면도 직접 확인했습니다.
같은 assertion의 수정 전 2 FAIL → 수정 후 2 PASS와 정상 대조 5 PASS를 보존했습니다.
필수 실행 41개는 모두 통과했습니다. 새 base Native/fresh WASM 대조도 완료했으며 최종 제출 감사 뒤 통합 PR을 생성합니다.
원 contributor head를 그대로 승인하거나 원격 조치를 수행했다는 뜻이 아닙니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 최신 리베이스 Native·fresh WASM 확인

최신 제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`에서 두 문제 입력은
**Native/fresh WASM 각각 전 5쪽 100.00%**다. 마지막 `력`까지 칸 내부 한 세로줄에 보존됨을
[동일 입력·5쪽의 양쪽 review와 별도 overlay](../assets/planet6897_20261103/rebase8609/vertical-cell-claims.md)에서
실제로 확인했다. 문제 글자 소실의 보정과 통합 공통 검증은 완료했습니다.

정상 저장 줄·실물·개요·표 경계까지 포함한 [Native 10입력·43쪽](../assets/planet6897_20261103/rebase8609/native-own-validation.md)과
[fresh WASM 10입력·43쪽](../assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 모두
90% 이상이며 직전 후보 대비 정밀 raw 0.1%p 초과 하락 0쪽이다.
필수 실행 41개와 Native/fresh WASM base 대조까지 완료했습니다. 아래 증거로 제출 조건을 확인합니다.

## 초기 세로쓰기 보정의 실제 증거

보정 소스 `37e5bba3007614d9304c3dbf863503a8c7340405`의 Native 제품을 별도로
빌드해 실제 컴파일 의존성에 새 공통 helper가 포함됐음을 확인했습니다.
[제품·명령·SHA와 전쪽 대조](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/README.md),
[주장 쪽 직접 판독](../assets/planet6897_20261103/direct-reading-current.md)을 보존했습니다.

| 입력 | 보정 후 Native 전쪽 TSV · 최저 raw | 직접 확인한 주장 쪽 |
| --- | --- | --- |
| `vertical_fit.hwpx` · 5쪽 | [TSV](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-silhouette.tsv) · 100.00% | 5쪽 [review](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-p005-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-00-p005-overlay.png) |
| `vertical_oversize.hwpx` · 5쪽 | [TSV](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-silhouette.tsv) · 100.00% | 5쪽 [review](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-p005-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-01-p005-overlay.png) |
| `42565_입법예고문.hwp` · 16쪽 | [TSV](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-silhouette.tsv) · 94.92448% | 10쪽 [review](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p010-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p010-overlay.png), 11쪽 [review](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p011-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/case-02-p011-overlay.png) |

두 축약 입력의 5쪽에서 `임용전경력` 다섯 글자가 한 세로줄의 셀 안에 표시되며, 마지막
글자의 소실이 복원됐습니다. 앞뒤 행·표 끝·쪽 소속을 함께 확인했습니다. 정상 대조군을
포함한 6입력·29쪽 중 27쪽은 이전 `c183a1ac61` 출력과 PNG 바이트가 동일하고,
두 문제 쪽만 개선됐습니다. 원본의 유효한 저장 세로줄은 유지합니다.

실물 10쪽의 기존 저장 세로줄 글자 간격과 정상 대조군의 연결 화살표 등에는 이전부터
Print와 차이가 남습니다. 해당 페이지 PNG가 직전 후보와 동일함을 대조했으며, 이번
합성 줄의 누락 보정을 원본 전체의 완전 일치로 확대하지 않습니다.

[같은 보정 소스의 fresh WASM 전쪽 TSV](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/wasm/README.md)도
6입력·29쪽을 완료했습니다. 두 문제 입력의 각 5쪽은 모두 100.00%이며, Native와 마찬가지로
두 문제 쪽만 바뀌고 나머지 27쪽의 PNG는 해당 경로의 직전 후보와 같습니다.
두 문제 쪽·실물 10/11쪽·저장 줄 정상 대조 1쪽의 Native/WASM review와 별도
overlay를 모두 직접 확인했습니다. 현재 통합 브랜치에 같은 보정을 적용했고 최신 source의 필수 실행 41개도 완료했습니다.
아래 초기 측정 결과는 원래 producer의 이력으로 보존합니다.
아래 이전 producer의 TSV·PNG는 보정 전 증거로 유지합니다.

## 필수 편람·환경 복구 결과

[보정 소스의 Native 편람 383쪽 게이트](../assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/handbook-validation.md)는
최저 99.96%, 기준 표시값 하락 0쪽·fallback 0건·exit 0으로 완료했습니다. 첫 실행의
ENOSPC는 성공으로 집계하지 않았으며 동일 제품·완료 구간을 검증해 재사용한 복구 결과입니다.

같은 코드에서 doctest의 동시 링크만 1개로 제한한 재검증은 8 PASS·3 IGNORE·exit 0입니다.
이 절은 초기 보정 source의 결과입니다. 현재 통합 source의 전체 nextest·Skia·CI Native 모듈·doctest는
아래 필수 41개 실행 기록에서 모두 완료로 확인했습니다. 공통 fresh WASM base 대조도 아래 기록에서 완료로 확인했습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8475](https://github.com/edwardkim/rhwp/pull/8475) · `planet6897` → `devel` |
| 원 head | `c01f286aca82a191bca56e3df89defddeeda81a0` |
| 현재 통합 적용 커밋 | `1301f3e9da34f7cc12cad712f83adbde49c64baf, c06485dcb9b2ede3e9f66841234e96e72b06d90f` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 초기 대표 PNG producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8470](https://github.com/edwardkim/rhwp/issues/8470) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 현재 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [vertical_fit.hwpx](<../../../samples/issue8470/vertical_fit.hwpx>) · [vertical_fit-2020.pdf](<../../../pdf/issue8470/vertical_fit-2020.pdf>) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/native/pr8475-vertical_fit-8a319181.tsv>) · 100.00% (5쪽) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/wasm/pr8475-vertical_fit-8a319181.tsv>) · 100.00% (5쪽) |
| [vertical_oversize.hwpx](<../../../samples/issue8470/vertical_oversize.hwpx>) · [vertical_oversize-2020.pdf](<../../../pdf/issue8470/vertical_oversize-2020.pdf>) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/native/pr8475-vertical_oversize-30055ea5.tsv>) · 100.00% (5쪽) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/wasm/pr8475-vertical_oversize-30055ea5.tsv>) · 100.00% (5쪽) |
| [42565_입법예고문.hwp](<../../../samples/issue8470/42565_입법예고문.hwp>) · [42565_입법예고문-2020.pdf](<../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/native/pr8475-42565_입법예고문-2279178b.tsv>) · 94.92% (16쪽) | [TSV · ba2](<../assets/planet6897_20261103/rebase8609/candidate/wasm/pr8475-42565_입법예고문-2279178b.tsv>) · 96.96% (16쪽) |

최신 TSV의 producer는 ba2이며 Native/fresh WASM 두 입력 전 5쪽의 직접 판독을 마쳤습니다.

## 보정 전 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [vertical_fit.hwpx](../../../samples/issue8470/vertical_fit.hwpx) · [vertical_fit-2020.pdf](../../../pdf/issue8470/vertical_fit-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-native.tsv) · 98.75% (5쪽) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-wasm.tsv) · 98.72% (5쪽) |
| [vertical_oversize.hwpx](../../../samples/issue8470/vertical_oversize.hwpx) · [vertical_oversize-2020.pdf](../../../pdf/issue8470/vertical_oversize-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-native.tsv) · 98.94% (5쪽) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-fresh-wasm.tsv) · 98.92% (5쪽) |
| [42565_입법예고문.hwp](../../../samples/issue8470/42565_입법예고문.hwp) · [42565_입법예고문-2020.pdf](../../../pdf/issue8470/42565_입법예고문-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-native.tsv) · 94.92% (16쪽) | [TSV](../assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-wasm.tsv) · 96.96% (16쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
위 전쪽 TSV는 각 링크에 표시한 실제 producer c183 또는 ba2의 완료 증거입니다. 대표 PNG는 해당 이미지의 원래 producer를 유지하며 최신 재출력과 구분합니다.
원본·PDF SHA·Print 절차·전쪽 누락·쪽수 대조는 [증거 목록](../assets/planet6897_20261103/README.md)에 연결합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

세로쓰기 칸을 가진 표는 저장 page_break 값을 보존하면서 조판에서는 나누지 않는 표로 취급한다.

model/table.rs의 Table::layout_page_break가 측정·typeset·layout 공통 소비 지점이다. has_vertical_cell 캐시는 셀 구성·글 방향 편집 뒤 무효화하며 저장 attr 자체는 바꾸지 않는다.

작은 표·본문보다 긴 표·걸침 위치가 다른 표를 독립 Print와 대조한다. 가로쓰기 표의 기존 분할과 편집·저장 왕복을 대조군으로 유지한다. 추가한 42565 실물은 devel 17쪽·본문 넘침 4건에서 Print와 같은 16쪽·5건으로 바뀐다. 추가 1건은 10쪽의 한 쪽 세로쓰기 표이며 Native 99.26%·fresh WASM 99.25%와 직접 판독으로 확인했다. 기존 기준선 행·공차는 유지하고 신규 입력의 정상 관측값 5건을 등록한다. [본문 경계와 독립 Print 대조 근거](../assets/planet6897_20261103/body-overflow-42565-proof.md)를 연결한다.

관련 정식 회귀 파일: [issue_8470_vertical_cell_table_no_split.rs](../../../tests/cases/issue_8470_vertical_cell_table_no_split.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](../assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 실제 검증 파일과 현재 Git 파일의 바이트가 일치함을 다시 확인했습니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 보정 전 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `vertical_fit.hwpx` 5쪽 · 세로쓰기 표 경계 | 98.75% · [review](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-overlay.png) | 98.72% · [review](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-overlay.png) |
| `vertical_oversize.hwpx` 5쪽 · 세로쓰기 넘침 경계 | 98.94% · [review](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-overlay.png) | 98.92% · [review](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-overlay.png) |
| `42565_입법예고문.hwp` 5쪽 · 구역 첫쪽 표 위 간격 | 99.27% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-overlay.png) | 99.24% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-overlay.png) |
| `42565_입법예고문.hwp` 10쪽 · 세로쓰기 표 한쪽 수용 | 99.26% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-overlay.png) | 99.25% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-overlay.png) |
| `42565_입법예고문.hwp` 11쪽 · 병합 칸 끝 행 | 100.00% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-overlay.png) | 100.00% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-overlay.png) |
| `42565_입법예고문.hwp` 13쪽 · 구역 첫쪽 표 위 간격 | 99.88% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-overlay.png) | 99.84% · [review](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-overlay.png) |


<!-- planet6897-20261103-generated-review -->

## 최신 통합 검증 상태

현재 base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`입니다. 직전 문서 갱신 뒤 PR #8618의 이슈 라벨링 workflow·계약 검사·운영 문서 3개가 추가됐습니다.
184개 패치가 충돌 0건·range-diff 모두 `=`로 리베이스됐습니다. 제품·테스트·fixture·검증 도구의 내용은
리베이스 전과 같으므로 완료 증거를 재사용합니다. 실제 측정 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`이며 문서 commit을 새 렌더 제품으로 표시하지 않습니다.
[현재 적용 SHA 39개](../assets/planet6897_20261103/integration-commits.md)를 확인합니다.

- [최소 글꼴 환경의 필수 실행 41개](../assets/planet6897_20261103/rebase8609/rust-regression-validation.md)는 모두 exit 0입니다.
  전체 nextest 12,583 PASS·45 SKIP, Native Skia lib 4,040 PASS·12 IGNORE,
  CI Native 모듈 21개와 doctest 8 PASS·3 IGNORE를 포함합니다.
- [최신 Native 영향 10입력·43쪽](../assets/planet6897_20261103/rebase8609/native-own-validation.md)과
  [fresh WASM 같은 43쪽](../assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 전쪽 90% 이상,
  직전 후보 대비 정밀 raw 0.1%p 초과 하락·누락·쪽수 불일치 0건입니다.
- [최신 필수 편람 Native 383쪽](../assets/planet6897_20261103/rebase8609/handbook-native-validation.md)은
  최저 표시 raw 99.96%, 기준 표시값 하락·fallback 0건·exit 0입니다.
- [최신 실제 API·브라우저](../assets/planet6897_20261103/rebase8609/runtime-validation.md)는 실제
  271 assertion·17 Worker·12쪽 렌더/저장과 가져오기·편집 산출물 대조를 완료했습니다.
- 기존 c183의 [Native 80입력·406쪽](../assets/planet6897_20261103/current-c183a1ac61/native-validation.md)과
  [fresh WASM 전406쪽](../assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md)은 양쪽 90% 이상입니다.
  충돌 없는 리베이스 뒤 전쪽 측정을 반복하지 않는 지시에 따라 원래 producer를 명시해 재사용합니다.
- 보호 PR #8389·#8397의 [c183 Native](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/native-validation.md)와
  [c183 fresh WASM](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/wasm-validation.md)은
  각 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락·누락 0행입니다.

원 PR별 주장에 대응하는 입력·쪽의 대표 review/overlay는 아래 원래 출처를 유지합니다.
최신 영향 입력의 그림은 최신 source로 별도 표시하며, 다른 문서의 대표 그림으로 대체하지 않습니다.
[직접 판독 원장](../assets/planet6897_20261103/direct-reading-current.md)과
[입력/PDF·compat·글꼴·명령](../assets/planet6897_20261103/visual-reproduction.md)을 연결합니다.
회귀 측정 주석 갱신은 실제 assertion·helper 본문과 구분했고 주석만 바뀐 전체 검사는 반복하지 않았습니다.

[새 base Native 대조](../assets/planet6897_20261103/rebase8609/native-base-preservation.md)와
[fresh WASM 대조](../assets/planet6897_20261103/rebase8609/wasm-base-preservation.md)는 각각 14입력·41쪽을 완료했습니다.
양쪽 모두 raw 하락 자체가 0쪽이며 90% 미만·누락·쪽수 차이·제품 해시 오류 0건입니다.
문서-only 리베이스의 [내용 동일성](../assets/planet6897_20261103/latest-rebase.md)을 확인해 완료 결과를 재사용했습니다.
추가 전체 회귀는 필요하지 않습니다. 제출 전 필수 검증은 완료했고 통합 PR #8622의 정확한 code head CI가 모두 성공했으며 이번 trailing head는 별도로 확인합니다.
원 PR 39개는 원격 재조회에서 head 불변·Open·non-Draft·reviewer 요청을 확인했습니다.
통합 PR 생성 전 본인 계정의 우선 처리할 열린 PR은 0건이었습니다. 원격 push·통합 PR 생성과 code head CI는 완료했습니다. 이 후속 문서 작성 시점에는 병합·원 PR 댓글·닫기를 아직 수행하지 않았습니다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다. 문서-only trailing commit의 병합·링크·기존 오늘할일 보존을 확인한 뒤 push하고 새 head CI를 확인합니다.

## 실제 Studio 출력의 소비 확인

최신 WASM을 실제 Studio의 Canvas2D·CanvasKit compat/default software에 연결한 screen/print 12개 캡처는 실제 backend 불일치·fallback·replay hard failure 0건, exit 0입니다.
[같은 두 원본의 5쪽 review·별도 overlay와 실제 화면](../assets/planet6897_20261103/rebase8609/studio-consumers.md)을 직접 확인했습니다. print 4개 영역의 raw는 99.99761~100%이고 마지막 글자와 셀 내부 포함·행 순서·표 종료를 보존합니다.
엄격 RGB parity의 보고용 차이는 남아 있으며 점수와 별도로 기록했습니다. 설치 글꼴 바이트는 로컬 host API로만 공급하고 배포하지 않았습니다.

## Merge 후 contributor PR comment 계획

[통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)의 실제 merge SHA와 devel 반영을 확인한 뒤 원 PR을 통합 완료로 정리합니다.
[이슈별 처리 범위](../assets/planet6897_20261103/source-followup-scope.md)에 따라 해결 이슈와 남은 별도 범위를 구분합니다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고, 위 전쪽 TSV와 같은 원본·PDF·주장 쪽의 review 및 별도 overlay를 사용합니다.
실제 그림은 직접 판독 원장의 producer·페이지·해시를 유지하며 다른 문서의 대표 그림으로 대신하지 않습니다.
원 PR의 기여와 메인터너 보정 이유·검증 결과·남은 차이를 한국어 존댓말로 설명합니다.
이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/...` 형식으로 고정하고 게시 전 응답의 이미지 형식·SHA-256을 원본과 대조합니다.
기존 같은 검토 댓글이 있으면 수정하고 없을 때만 처음 게시합니다. 게시 뒤 API 본문과 이미지 표시를 재확인하며 contributor fork branch는 삭제하지 않습니다.

[최종 code CI와 오탐 처리](../assets/planet6897_20261103/code-head-ci-completion.md)를 함께 기록했습니다. 현재 기록 시점에는 merge·원 PR close/comment를 아직 실행하지 않았으며, 실제 원격 완료 사실은 원 PR 타임라인과 통합 PR에 남깁니다.

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8475)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
