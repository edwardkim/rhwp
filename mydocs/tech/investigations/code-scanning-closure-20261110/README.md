---
kind: investigation
status: active
canonical: mydocs/manual/github_operations.md
last_verified: 2026-11-10
---

# CodeQL 보정의 보호 브랜치 종료 확인

2026-11-10 승인한 순서에 따라 devel 전체 분석, main 보안 백포트, 양쪽 실제 경고 상태와 [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597) 분류를 확인한다. 완료된 #8745·#8750과 [PR #8867](https://github.com/edwardkim/rhwp/pull/8867)의 source·검증 근거를 구분하며 기존 분류를 되돌리지 않는다.

## main의 최소 백포트

main 기준 `c9095c5f67632e6721e800ccc7fb43ea49d26682`에 source `913713dbfd0a911130e7d590629a656ffc1940b3`의 여덟 파일만 적용했다. 로컬 source head는 `7bbe651832c6fd0dab9c2fe381f5a5d0752ecef7`이다. 다른 기능·의존성·workflow 변경은 포함하지 않는다. main 반영 지시에 따라 main 기반 전용 branch를 사용하며 일반 devel 기능 제출과 구분한다.

[로컬 실행 원장](main-local-validation.json)에서 필수 Rust lint·workspace build, PDF 로그 2건·기존 인증 경계 6건과 전체 회귀 **12,999 PASS·0 FAIL·기존 45 skip**을 확인한다. 최소 글꼴 격리에서 공개 기본 글꼴 5개만 노출했다. 실제 사용한 nextest 버전과 기존 권고/관찰 보고 설정 경고도 구분했다.

[동일성·입력 원장](main-source-identity.json)은 실제 보정 Rust 소스, 선택 C 테스트 manifest, codec-resource crate와 toolchain이 수용된 devel source와 같음을 확인한다. 공개 PDF·HWPX 4개도 실제 bytes와 commit bytes가 같다. qcms 선택 기능 30 PASS의 원 실행 증거는 [수용된 devel의 runtime-validation](https://github.com/edwardkim/rhwp/blob/b5064fae1eefff92d60464c51844a38debec09f9/mydocs/tech/investigations/code-scanning-20261109/runtime-validation.json)이며, 독립 전후 CodeQL·ICC 입력·실패 경계는 [원 검증 원장](https://github.com/edwardkim/rhwp/blob/b5064fae1eefff92d60464c51844a38debec09f9/mydocs/tech/investigations/code-scanning-20261109/README.md)을 따른다. 이를 새 main의 전체 CodeQL 분석 완료로 표시하지 않는다.

## 보호 브랜치 분석과 원격 완료 조건

devel의 exact `b5064fae1e`에서 [전체 분석](https://github.com/edwardkim/rhwp/actions/runs/42714977872)을 실행했다. 분석은 성공했고 실제 Rust full 초기화, 동일한 CodeQL 2.27.4·query pack 및 26개 규칙을 확인했다. [전체 SARIF와 ref별 상태 대조](devel-closure-verification.json)에서 결과는 144→137건, qcms 포인터는 7→0건이며 [보안 경고 #616](https://github.com/edwardkim/rhwp/security/code-scanning/616)–[보안 경고 #622](https://github.com/edwardkim/rhwp/security/code-scanning/622)의 devel instance는 모두 fixed다. PR Rust 분석의 축소 경로를 protected ref 전체 분석으로 대신하지 않는다.

기존 [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)도 devel에서 fixed로 확인됐다. GitHub는 같은 함수의 수치 진단을 새 **[보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)**으로 등록했다. devel OPEN은 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623) 한 건이며 4개 flow 모두 `page_index: usize`로 이어진다. [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)은 이전의 번호별 분류 승인에 포함되지 않았으므로 추가 명시 승인 없이 분류하지 않는다. main에도 본문 출력 제거를 적용하고 전체 분석에서 같은 수치-only 범위를 확인한 뒤 이 새 번호의 분류를 확정한다. main의 본문 출력이 남아 있는 동안에는 분류하지 않는다. main PR의 필수 리뷰 1건과 모든 적용 CI 성공을 확인한 뒤에만 병합하며, 사용자 승인 없는 보호 설정 변경·리뷰 요건 우회는 하지 않는다.
