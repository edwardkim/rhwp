# CodeQL 잔여 경고의 보호 브랜치 종료 확인

## 승인과 경로

2026-11-10 작업지시자는 devel 전체 분석 → 포인터 7건 확인 → [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597) 수치 flow 확인·분류 → main 반영·전체 분석·최종 상태 기록의 절차에 “순서대로 진행합니다.”라고 승인했다. [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)의 분류는 main에도 본문 출력 제거가 실제 반영된 뒤로 늦춘다. main의 원문 출력이 남은 상태에서 전체 경고를 오탐으로 가리지 않는다.

- devel source: PR #8867 merge `b5064fae1eefff92d60464c51844a38debec09f9`.
- devel 전체 분석: 수동 CodeQL run `42714977872`, exact SHA와 vendor 포함 실제 full 초기화·SARIF를 확인한다.
- main base: `c9095c5f67632e6721e800ccc7fb43ea49d26682`.
- main의 보안 수정 준비는 `task/8745-code-scanning-main` 전용 worktree에서 진행한다. 이번 지시의 main 반영을 위한 제한 백포트이므로 일반 변경의 devel 기반 원칙과 구분한다.
- main에 반영할 source는 이미 검증·수용한 `913713dbfd0a911130e7d590629a656ffc1940b3`의 여덟 파일뿐이다. 다른 기능·의존성·workflow 변경을 포함하지 않는다.
- PR 경로: `collaborator_self_merge`. 보조 문서는 intake_and_review, local_validation, review_only_fast_pass, post_merge다. 모 문서·선택표와 해당 자식 절차를 확인했다. 본인 OPEN PR은 0건이다.

## 선행 근거와 검증

완료된 #8745·#8750은 유지하고 PR #8867과 최초/현재 SARIF·로컬 전후 분석·실제 로그/포인터 경계 증거를 추적한다. 같은 source를 백포트하고 원 PR의 독립 보정 근거를 재사용하되 main 후보의 필수 Rust lint·집중 회귀·전체 release-test와 새 PR CI를 실제 실행한다. 입력 PDF와 HWPX의 commit 동일성, 선택 qcms source·자원 dependency·toolchain 동일성을 확인한다. 조판·색 변환·기본 feature 계산은 수정하지 않으므로 시각 baseline 변경은 없다.

main PR의 required 승인 리뷰 1건은 일반 merge 조건으로 유지한다. 해당 리뷰 부족을 self-review나 admin 권한으로 임의 우회하지 않는다. main 반영·전체 분석 뒤 [보안 경고 #616](https://github.com/edwardkim/rhwp/security/code-scanning/616)–[보안 경고 #622](https://github.com/edwardkim/rhwp/security/code-scanning/622)의 fixed 상태, [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)의 실제 수치-only flow와 승인된 false positive 분류, 기존 dismissed 상태 보존을 확인해 종료한다.

## devel 전체 분석 완료와 실제 경고 상관관계

수동 run `42714977872`의 네 언어 분석이 모두 성공했다. Rust analysis `2131136443`은 exact `b5064fae1e`의 vendor 포함 전체 범위이며 CodeQL 2.27.4와 26개 규칙·query pack 신원이 기준 분석과 같다. SARIF 144→137건, qcms 포인터 7→0건을 확인했고 [보안 경고 #616](https://github.com/edwardkim/rhwp/security/code-scanning/616)–[보안 경고 #622](https://github.com/edwardkim/rhwp/security/code-scanning/622)와 기존 [보안 경고 #597](https://github.com/edwardkim/rhwp/security/code-scanning/597)의 devel instance는 모두 fixed다.

잔여 수치 진단은 새 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)으로 등록됐다. 4개 flow의 실제 끝은 `page_index: usize`이며 본문 출력이 아니다. 이 새 번호는 기존 번호별 분류 승인과 구분해 OPEN을 유지한다. main의 보정·전체 분석을 확인한 뒤 [보안 경고 #623](https://github.com/edwardkim/rhwp/security/code-scanning/623)의 명시 분류 결정을 받는다.

main의 source `7bbe651832`는 전체 회귀 12,999 PASS·기존 45 skip, 집중 8 PASS와 필수 Rust lint·workspace build·suite 정책을 통과했다. 실제 qcms·PDF backend 전체 tree와 codec-resource·toolchain이 수용된 devel과 같고 공개 검증 입력 4개도 Git bytes와 같다. 새 원격 head를 제출한 뒤 해당 PR CI·self-review·main의 필수 승인 리뷰를 확인한다.
