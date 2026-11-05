# PR #8433 리뷰 — fix(layout): 번호 정의 없는 개요의 기본 머리 모양에 본문과의 거리 50%·자동 내어쓰기를 합성한다 (#8425)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
제출 전 필수 검증의 완료 근거를 아래에 기록했습니다. 작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 기본 개요 예약 폭의 추가 보정 — 양쪽 실측·필수 회귀 완료

공통 예약 폭 보정 `c183a1ac61c2aa4188989fd93032cae68d3538c1`을 적용했습니다.
번호 정의가 없는 내장 `^N`의 참조 형식 폭과 실제 글리프 폭을 분리해 측정·줄 나눔·배치가
같은 영역을 소비합니다. 명시적 번호 정의나 글머리표에 이 합성 폭을 부여하지 않습니다.

| 같은 원본·독립 Print | 보정 후 Native TSV | 직접 확인한 review / overlay |
| --- | --- | --- |
| 수준 1~7 탐침 · 한컴 2020 | [전 1쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-native.tsv) · 100.00% | [review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-native-overlay.png) |
| 같은 탐침 · 한컴 2024 | [전 1쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-native.tsv) · 100.00% | [review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-native-overlay.png) |
| 원 회귀 issue3307 · 한컴 2020 | [전 9쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-native.tsv) · 최저 99.99%, 7쪽 100.00% | [7쪽 review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-native-overlay.png) |

세 영역의 review와 별도 overlay를 직접 읽어 제목 시작·긴 제목의 내어쓰기와 원 회귀의
개요·리터럴 번호·내용 순서를 확인했습니다. 탐침은 이전 Native 91.85%·91.97%에서
100.00%로 개선됐습니다. [13개 입력·Print 조합 73쪽의 Native 대조](../assets/planet6897_20261103/intermediate-evidence-summary.md)는
모두 90% 이상이고 이전 표시값 하락은 0쪽입니다. [실제 Native 제품](../assets/planet6897_20261103/intermediate-evidence-summary.md)의
소스·실행 파일 해시를 연결했습니다. 글머리표 대조군 전 4쪽과 실제 NUMBER 문단을 쓰는 별도 대조군 전 2쪽은
보정 전·후 Native·WASM 모두 100.00%입니다. 번호 정의의 존재와 실제 문단의 사용을 구분했습니다.

[fresh WASM의 같은 13개 조합·73쪽 대조](../assets/planet6897_20261103/intermediate-evidence-summary.md)도
완료했습니다. 전쪽 최저는 Native 95.98% / fresh WASM 96.12%이며 이전 표시값 하락은 양쪽 모두 0쪽입니다.
[글머리표·명시적 번호 대조군](../assets/planet6897_20261103/intermediate-evidence-summary.md)의
실제 사용 경로와 각 전쪽 TSV를 보존했습니다. 새 보정의 필수 회귀 41개 실행은 모두 완료했습니다. 남은 제출 대조는 아래에서 구분합니다. 아래 기존 측정은 보정 전
제품의 결과로 보존하며 새 보정의 완료나 최종 승인으로 표시하지 않습니다.

### 같은 코드의 fresh WASM 직접 대조

| 원본·기준 | fresh WASM 전쪽 TSV·최저 | 직접 확인한 review / overlay |
| --- | --- | --- |
| 수준 1~7 탐침 · 2020 | [전 1쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-wasm.tsv) · 100.00% | [review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2020-wasm-overlay.png) |
| 같은 탐침 · 2024 | [전 1쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-wasm.tsv) · 100.00% | [review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/levels-2024-wasm-overlay.png) |
| issue3307 · 2020 | [전 9쪽](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-wasm.tsv) · 최저 99.94%, 7쪽 99.99% | [7쪽 review](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/reference-width-c183a1ac61/issue3307-p007-wasm-overlay.png) |

세 영역의 fresh WASM review와 별도 overlay를 직접 판독했습니다. 제목 시작점과 긴 제목의
이어지는 줄, 실물 7쪽의 번호·본문·리터럴 번호가 보존됩니다. [최적화 제품과 PNG 연결](../assets/planet6897_20261103/intermediate-evidence-summary.md)을
기록했으며 PDF를 재산출하지 않았습니다.

## 새 보정의 최소 글꼴 집중 회귀

원본·독립 Print의 양쪽 전쪽 측정 뒤 실제 최소 글꼴 환경에서 정식 검사를 실행했습니다.
기본 개요 4개와 저장 줄 재시작 표 2개, 총 6개가 통과했습니다.
[실행 명령·종료 코드·검사 파일 해시](../assets/planet6897_20261103/rust-validation-c183a1ac61.md)를
연결합니다. 원본 장평을 누락한 첫 검사 실패와 수정 근거도 같은 기록에 구분했습니다.
전체 Rust·Skia·lint·doctest의 필수 실행 41개는 모두 통과했습니다. 새 base Native/fresh WASM 대조도 아래 공통 기록에서 완료로 확인했습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8433](https://github.com/edwardkim/rhwp/pull/8433) · `planet6897` → `devel` |
| 원 head | `70aff23228747919c3be80d773737864b8b8a649` |
| 현재 통합 적용 커밋 | `baa415907b1e25aac728f8d9d32b60dddd29474a, 9d3f0bcb2b04d5cb76e63460ae0e708b80d4499f` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 초기 대표 PNG producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8425](https://github.com/edwardkim/rhwp/issues/8425) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(layout): 번호 정의 없는 개요의 기본 머리 모양에 본문과의 거리 50%·자동 내어쓰기를 합성한다 (#8425)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 보정 전 검증 입력과 전쪽 결과 — 4b149b6cdf

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [levels.hwpx](../assets/issue_8425/probe/levels.hwpx) · [levels-2024.pdf](../../../pdf/issue-8425/levels-2024.pdf) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-levels-d014d88d-native.tsv) · 91.97% (1쪽) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-levels-d014d88d-wasm.tsv) · 92.21% (1쪽) |
| [levels.hwpx](../assets/issue_8425/probe/levels.hwpx) · [levels-2020.pdf](../../../pdf/issue-8425/levels-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-levels-493f24c1-native.tsv) · 91.85% (1쪽) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-levels-493f24c1-wasm.tsv) · 92.03% (1쪽) |
| [issue3307_outline_number.hwpx](../../../samples/task3307/issue3307_outline_number.hwpx) · [issue3307_outline_number-2020.pdf](../../../pdf/task3307/issue3307_outline_number-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-issue3307_outline_number-8704d61b-native.tsv) · 99.99% (9쪽) | [TSV](../assets/planet6897_20261103/pr_8433/pr8433-issue3307_outline_number-8704d61b-wasm.tsv) · 99.94% (9쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
위 전쪽 TSV는 각 링크에 표시한 실제 producer c183 또는 ba2의 완료 증거입니다. 대표 PNG는 해당 이미지의 원래 producer를 유지하며 최신 재출력과 구분합니다.
원본·PDF SHA·Print 절차·전쪽 누락·쪽수 대조는 [증거 목록](../assets/planet6897_20261103/README.md)에 연결합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

번호 정의가 없는 기본 개요는 본문과의 거리 50%와 자동 내어쓰기를 함께 합성한다. 정상 번호 정의의 저장값은 독립 계약으로 보존한다.

src/renderer/layout/utils.rs의 resolve_numbering_id·default_outline_numbering이 만든 기본 속성을 개요 머리와 본문의 실제 배치가 소비한다.

levels probe와 issue3307 원본의 대응 줄에서 번호·본문의 상대 거리와 후속 줄을 대조한다. 2020·2024 Print를 같은 파일명이라는 이유로 혼용하지 않고 각각 연결한다.

관련 정식 회귀 파일: [issue_8425_default_outline_head_distance.rs](../../../tests/cases/issue_8425_default_outline_head_distance.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](../assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 실제 검증 파일과 현재 Git 파일의 바이트가 일치함을 다시 확인했습니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 보정 전 원 PR 주장 쪽의 시각 증적 — 4b149b6cdf

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `levels.hwpx` 1쪽 · 기본 개요 머리 모양 | 91.97% · [review](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-07bae7-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-07bae7-native-overlay.png) | 92.21% · [review](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-07bae7-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-07bae7-wasm-overlay.png) |
| `levels.hwpx` 1쪽 · 개요 머리 probe의 2020 Print 최저 페이지 | 91.85% · [review](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-d891f2-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-d891f2-native-overlay.png) | 92.03% · [review](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-d891f2-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-claim-p001-levels-d891f2-wasm-overlay.png) |
| `issue3307_outline_number.hwpx` 7쪽 · 개요 번호 본문 거리 | 100.00% · [review](../assets/planet6897_20261103/pr_8433/pr8433-outline-claim-p007-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-outline-claim-p007-native-overlay.png) | 99.99% · [review](../assets/planet6897_20261103/pr_8433/pr8433-outline-claim-p007-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8433/pr8433-outline-claim-p007-wasm-overlay.png) |

## 보정 전 차이와 해소 기록

2026-11-05 KST에 2020 Print 탐침의 1쪽 Native/fresh WASM review와 별도 overlay를
직접 대조했습니다. 수준 1~7의 번호·본문 및 넘친 줄은 보존되지만, 제목 글자 시작점과
굵기에 가시 차이가 남습니다. 90% 통과만으로 해당 차이를 정상으로 판정하지 않습니다.
원 PR 본문과 원본 HWPX의 글자 속성·PDF의 실제 글꼴 및 번호 폭을 대조했습니다.
이 차이는 위 `c183a1ac61`의 참조 폭 보정으로 해소했으며, 같은 두 탐침은 Native·WASM 모두 100.00%입니다.
[직접 판독 기록](../assets/planet6897_20261103/direct-reading-current.md)에 해당 파일의 해시를
연결했습니다. 원 PR이 주로 검증한 `issue3307_outline_number.hwpx` 7쪽의 개선과 탐침의
보정 전 차이를 구분합니다. 최종 공통 검증은 별도로 완료해야 합니다.

기존 증적 표의 2020 Print 탐침 행은 같은 HWPX의 2024 PDF 점수를 잘못 가져왔습니다.
이미지와 TSV는 올바른 2020 자료였으며, 이미지의 입력·PDF 해시 기록으로 점수 연결을
수정했습니다. [전체 주장 행 대조](../assets/planet6897_20261103/direct-reading-image-sha256.md)는
96개 행을 확인하고 실제 수치 정정 1개 행을 기록합니다. PDF나 엔진 출력을 재생성하지 않았습니다.


<!-- 수동 보정 기록을 포함하므로 일괄 생성으로 덮어쓰지 않는다. -->

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

## Merge 후 contributor PR comment 계획

[통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)의 실제 merge SHA와 devel 반영을 확인한 뒤 원 PR을 통합 완료로 정리합니다.
[이슈별 처리 범위](../assets/planet6897_20261103/source-followup-scope.md)에 따라 해결 이슈와 남은 별도 범위를 구분합니다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고, 위 전쪽 TSV와 같은 원본·PDF·주장 쪽의 review 및 별도 overlay를 사용합니다.
실제 그림은 직접 판독 원장의 producer·페이지·해시를 유지하며 다른 문서의 대표 그림으로 대신하지 않습니다.
원 PR의 기여와 메인터너 보정 이유·검증 결과·남은 차이를 한국어 존댓말로 설명합니다.
이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/...` 형식으로 고정하고 게시 전 응답의 이미지 형식·SHA-256을 원본과 대조합니다.
기존 같은 검토 댓글이 있으면 수정하고 없을 때만 처음 게시합니다. 게시 뒤 API 본문과 이미지 표시를 재확인하며 contributor fork branch는 삭제하지 않습니다.

[최종 code CI와 오탐 처리](../assets/planet6897_20261103/code-head-ci-completion.md)를 함께 기록했습니다. 현재 기록 시점에는 merge·원 PR close/comment를 아직 실행하지 않았으며, 실제 원격 완료 사실은 원 PR 타임라인과 통합 PR에 남깁니다.

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8433)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
