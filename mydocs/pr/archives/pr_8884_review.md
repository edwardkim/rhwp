# PR #8884 self-review — main 보안 백포트

## 최종 판정

**승인·병합 완료.** code head와 문서-only trailing head의 CI·self-review를 확인하고, 승인된 리뷰 개수 요건의 이번 PR 한정 예외로 main에 병합했다. main·devel 전체 CodeQL에서 원 경고 8건이 fixed가 됐으며, 별도 승인된 숫자-only 경고 #623 분류 후 두 보호 브랜치의 OPEN은 모두 0건이다. 아래 최종 종료 확인이 병합 전 조건 기록보다 우선한다.

## 최종 종료 확인 — 2026-11-10T10:13:45.330083+09:00

- 작업지시자가 이번 PR의 승인 리뷰 1건 요건만 명시적으로 예외 승인했다. 최종 head `e44630a7b1b08e2433a96e95b8d5068aedd64da9`의 CI와 self-review·최신 main 병합 시뮬레이션이 통과한 뒤 정확한 head를 지정한 admin merge를 수행했다. 보호 설정·CI 기준은 변경하지 않았다.
- 실제 main merge는 `7213ed1f96266b95e22cd5645a07851c9c18298e`이며 병합 시각은 `2026-11-10T00:27:50Z`다. [trailing head CI 원장](../assets/code-scanning-closure-20261110/main-trailing-success.json)의 모든 workflow가 성공했고 실제 fast-pass를 확인했다.
- 별도로 승인된 [main 전체 CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42728536362)은 같은 merge source의 네 언어 모두 성공했다. Rust analysis `2131629953`의 full 초기화·규칙 26개와 query pack이 유지됐다. SARIF 144→137건, qcms invalid-pointer 7→0건이다.
- 기존 [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)과 [#616](https://github.com/edwardkim/rhwp/security/code-scanning/616)–[#622](https://github.com/edwardkim/rhwp/security/code-scanning/622)는 main·devel에서 모두 fixed다. 원래 경고를 오탐 분류로 대신 닫지 않았다.
- 새 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)의 4개 flow는 main 전체 분석에서도 `page_index: usize` 숫자 진단으로만 이어지며 devel의 전체 topology와 같았다. 이 조건을 확인한 뒤 승인 범위에 따라 해당 번호 한 건만 `false positive`로 분류했다. [실제 source·flow](../assets/code-scanning-closure-20261110/main-full-verification.json).
- 최종 조회의 OPEN은 default·main·devel 모두 0건이다. default 분류 수는 dismissed 184건·fixed 48건이며, 기존 dismissed 183건의 번호·사유가 그대로다. [분류 후 API 검증](../assets/code-scanning-closure-20261110/final-security-state.json).
- #8745의 기존 CLOSED 상태를 유지했다. main push의 [Pages](https://github.com/edwardkim/rhwp/actions/runs/42728493675)는 `success`다. duration 갱신 workflow는 devel 전용이므로 main 대상에는 적용되지 않는다. 별도 제품 CI를 중복 실행하지 않았다.
- 최종 기록은 maintainer 직접 반영 범위인 archive·asset·오늘할일만 devel에 남긴다. source·test·workflow·golden·기존 sample은 포함하지 않는다. [원격 완료·source 동일성](../assets/code-scanning-closure-20261110/remote-completion.json).
- 전용 branch/worktree·루트 output·공유 debug의 실제 종료 정리는 `target/pr-review/cleanup-security-closure-20261110.json`에서 확인한다. 다른 작업 #8863의 공유 target 사용이 끝나지 않았으면 해당 사용 근거를 남기고 debug를 보존하며, 이를 정리 완료로 표시하지 않는다.

## 대상과 승인

- PR: https://github.com/edwardkim/rhwp/pull/8884. 작성자 edwardkim, base main.
- 제출 code head `9558a0b0213f56181f2a98646d472d41f1fca52f`.
- base `c9095c5f67632e6721e800ccc7fb43ea49d26682`, 검증 source `7bbe651832c6fd0dab9c2fe381f5a5d0752ecef7`.
- 작업지시자 “순서대로 진행합니다.”에 따른 보호 브랜치 종료 확인과 제한 main 반영이다. 일반 devel 기반 기능 제출과 구분하며 이미 수용한 #8867 source만 백포트한다.
- base route `collaborator_self_merge`, modifiers intake_and_review, local_validation, review_only_fast_pass, post_merge와 대형 증적의 rework_and_exceptions. 모 문서·선택표·해당 자식 문서를 확인했다. 본인 OPEN PR 0건에서 제출했고 신규 후보의 exact main merge tree가 충돌 없이 후보 tree와 같음을 확인했다.

## source와 선행 근거

완료된 #8745·#8750과 최초/현재 flow·source/sink 도입, 실제 독립 대조는 #8867의 정식 원장을 따른다. source `913713dbfd`를 -x cherry-pick하여 PDF 진단의 payload 두 곳과 qcms 테스트 nullable pointer 일곱 곳을 보정했다. 선택 C LUT adapter는 기존 자원 예산 실패를 전파하면서 호출자의 배열을 보존한다. 제품 색 변환·조판 계산·인증 정책·기본 feature를 바꾸지 않는다.

Rust 보정 코드와 qcms·PDF backend 전체 tree, codec-resource·toolchain이 수용된 devel source와 같다. main의 기존 cfb·web-sys와 workflow는 유지하고 root dev-dependency log만 추가했다. 소스 8개 파일에 더해 대부분의 diff는 정규화된 검증 JSON이다. 새 테스트는 실제 로그와 공개 control PDF·캐시를 확인하며 양성 이벤트까지 검사한다. null helper의 기존 SIGSEGV와 budget/정상 C LUT의 독립 실행 증거도 같은 소스 근거로 재사용했다. 추가 vendor Clippy의 기존 125개 진단과 정식 root lint 통과는 원 증적에서 구분한다.

## 로컬 검증과 입력

- 최소 글꼴 격리 전체 release-test: 12,999 PASS·0 FAIL·기존 45 skip.
- PDF 로그 비노출·내용/캐시 보존 2 PASS, 기존 인증 경계 6 PASS.
- fmt·native Clippy·WASM32 Clippy·workspace build·workspace all-target Clippy·suite 정책 PASS.
- 라이선스 확인 공개 글꼴 5개만 노출했다. 실제 사용한 nextest 0.9.137과 기존 0.9.140 권고/미사용 관찰 프로필의 report-skipped 경고는 실행 원장에 기록했다.
- qcms 선택 기능 30 PASS는 동일 standalone source·manifest, 자원 crate·toolchain의 독립 실행을 재사용한다. 이번 main에서 새로 실행한 30건으로 표시하지 않는다.
- 입력 커밋 확인: 충족. 공개 PDF 2개와 HWPX 2개의 실제 bytes가 Git bytes와 같고 LFS pointer가 아님을 확인했다. 동작만 합성한 공개 PDF는 source에 보존됐다.

근거: [새 검증 원장](https://github.com/edwardkim/rhwp/blob/7213ed1f96266b95e22cd5645a07851c9c18298e/mydocs/tech/investigations/code-scanning-closure-20261110/README.md), [명령·시각·결과](https://github.com/edwardkim/rhwp/blob/7213ed1f96266b95e22cd5645a07851c9c18298e/mydocs/tech/investigations/code-scanning-closure-20261110/main-local-validation.json), [소스·입력 동일성](https://github.com/edwardkim/rhwp/blob/7213ed1f96266b95e22cd5645a07851c9c18298e/mydocs/tech/investigations/code-scanning-closure-20261110/main-source-identity.json).

## devel 전체 CodeQL과 실제 경고 번호

[수동 전체 분석](https://github.com/edwardkim/rhwp/actions/runs/42714977872)은 exact devel `b5064fae1e`에서 네 언어 모두 성공했다. Rust analysis 2131136443의 실제 full 초기화와 동일한 CodeQL 2.27.4·query pack·규칙 26개를 확인했다. SARIF는 144→137건, qcms 포인터는 7→0건이며 [보안 경고 #616](https://github.com/edwardkim/rhwp/security/code-scanning/616)–[보안 경고 #622](https://github.com/edwardkim/rhwp/security/code-scanning/622)와 기존 [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)의 devel instance는 모두 fixed다.

숫자-only 진단 4개 flow는 GitHub가 새 **[보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)**으로 등록했다. 모든 끝점이 `page_index: usize`를 거쳐 길이·쪽 번호 로그로 이어진다. 현재 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)은 OPEN이며 이전 번호별 분류 승인과 구분한다. main full scan과 새 번호의 명시 분류 승인 뒤에만 후속 상태를 확정한다. PR Rust CodeQL은 vendor 전체를 검사하지 않으므로 이 PR check 성공을 protected ref 전체 종료로 표시하지 않는다.

[전체 결과와 ref별 instance·flow](https://github.com/edwardkim/rhwp/blob/7213ed1f96266b95e22cd5645a07851c9c18298e/mydocs/tech/investigations/code-scanning-closure-20261110/devel-closure-verification.json)를 보존했다. 원문 SARIF는 임시 자료이며 정규화된 판단 근거만 커밋했다. query·path·language·severity와 sanitizer는 바꾸지 않았다.

## 조판 원칙 준수 검토

| 항목 | 결과·근거 |
| --- | --- |
| 선행 이슈·해결 PR·독립 검증 | 충족 — #8745·#8750·#8867과 실제 red/green·전체 분석을 분리 |
| 구현 근거와 공통 경계 | 충족 — 실제 로그 payload와 nullable 참조·C 예산 경계를 직접 보정 |
| 입력 커밋 확인 | 충족 — 공개 입력 4개와 source 합성 PDF |
| main 기준 기능 유지 | 충족 — source 8개 파일 제한, 전 backend tree 동일성·집중/전체 회귀·필수 lint |
| 측정·배치·분할·좌표 소비 | 비해당 — 해당 source/연산을 수정하지 않음 |
| 시각 회귀·확대 판독·handbook | 비해당 — 로그·포인터 계약이며 렌더링 fixture/golden을 추가·갱신하지 않음 |
| 최종 소비 계약 | 충족 — decoded/cached bytes, 로그 양성/비노출, null·예산 실패와 정상 LUT |

## code head CI

2026-11-10 09:09:50 KST에 exact code head의 모든 check 완료와 실패·취소·대기 없음, Build & Test success를 확인했다. Native의 실제 테스트 단계도 success다. 수동 재실행·CI 기준 변경·보호 설정 변경은 없었다.

| Workflow | Run | 결과 |
| --- | --- | --- |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42722522312) | `42722522312` | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42722522558) | `42722522558` | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42722522660) | `42722522660` | success |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/42722522512) | `42722522512` | success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42722522457) | `42722522457` | success |

[실제 check 원장](https://github.com/edwardkim/rhwp/blob/7213ed1f96266b95e22cd5645a07851c9c18298e/mydocs/tech/investigations/code-scanning-closure-20261110/main-code-ci.json)에 적용되지 않은 조건부 skip과 실행한 제품 검사를 구분했다. main은 CI Impact Policy의 게시 대상이 아니므로 존재하지 않는 legacy status를 성공으로 가정하지 않았다.

## 병합과 후속 조건

code head CI 성공을 확인했으며 이 review·오늘할일을 source branch의 문서-only trailing commit으로 추가한다. push 전 최신 main merge tree·공백·링크·기존 기록 보존과 원격 base/head를 확인한다. trailing head의 실제 CI·fast-pass·main의 필수 승인 리뷰 조건을 확인해 exact head로 일반 merge한다. 승인 리뷰 부족을 임의로 admin bypass하지 않는다.

보호 브랜치 종료를 위한 별도 승인된 main 전체 CodeQL 실행을 확인한 뒤 기존 8건의 fixed와 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623) 수치 flow·분류를 최종 확인한다. 일반 병합 후 중복 제품 CI는 실행하지 않는다. 기존 완료 이슈와 dismissed 상태를 보존하고 원격 결과·devel sync·전용 worktree/branch·루트 output·공유 debug를 정리한다.
