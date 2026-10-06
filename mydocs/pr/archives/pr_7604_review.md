# PR #7604 검토 — 릴리스 운영 계약의 실행 경로 보정

## 현재 판정

로컬 보정과 검증을 완료했고 사용자 승인 후 [Open PR #7604](https://github.com/edwardkim/rhwp/pull/7604)을 등록했다. 최종 제출 head의 CI 완료와 별도 병합 판단은 남아 있다. devel의 manual caller 실행 성공이나 릴리스 완료로 표현하지 않는다.

## 접수와 경로

- 작성자 edwardkim의 self-review다. collaborator_self_merge 기본 경로와 intake_and_review, local_validation을 적용하며 본인 PR에 reviewer를 assign하지 않는다.
- base devel `d84dce2ac9f6a7fcaf522036b8801aba76234dac`, 구현·최초 게시 head `4aa478027e9b894925d04b8364c6566b22d07556`.
- 최초 API 조회: Open·draft false·6파일·102줄 추가/17줄 삭제·mergeable true, mergeable_state blocked. API 값은 조회 시점 정보이며 최종 head와 병합 직전에 다시 확인한다.
- [선행 PR #7603](https://github.com/edwardkim/rhwp/pull/7603)이 성공한 CI head를 merge commit으로 통합한 뒤 이 브랜치를 만들었다. 최신 main은 base의 ancestor다. 게시한 한국어 본문과 임시 UTF-8 파일의 정확한 일치, 선두 BOM·`??` 치환 부재를 확인했다.

## 원인과 변경 경로

새 dispatch-only `release-operations-contracts.yml`은 GitHub workflow 목록에 없었고 GET API가 404였다. 등록된 CI는 active였다. main 승격 전 필요한 증적을 만들기 위해 main 직접 수정·검증 면제를 쓰지 않고, `.github/workflows/ci.yml`의 manual dispatch가 같은 commit의 local reusable adapter를 호출하게 했다.

실제 호출은 `ci.yml:release-operations-contracts` → `./.github/workflows/release-operations-contracts.yml:contracts` → 기존 Python·Node script/API 실패 계약이다. caller는 preflight 성공과 workflow_dispatch에서만 실행된다. PR/push에서는 skipped다. caller/callee 모두 contents read이며 secrets 전달과 metadata 쓰기 이벤트가 없다. checkout은 caller의 github.sha를 사용한다.

정책의 metadata 세 source와 adapter 자체는 evidencePath를 CI로 연결한다. 각각 CI preflight·Build & Test·`release-operations-contracts / Release operations contracts`의 success를 요구한다. collector의 source별 hash·mode 보존과 verifier의 exact SHA·actor·pagination 검사는 유지한다. same-run 기록 중 각 source의 contracts job만 skipped/failure로 바꾸면 gate가 그 source를 거부하며 나머지 source는 수용하는 것을 실행으로 확인했다. CI required check 이름과 기존 worker lane 집계는 변경하지 않았다. adapter 실패는 run failure와 promotion의 필수 job 검사로 거부된다.

## 검증과 범위

[준비 기록의 보정 절](../../plans/release_0_8_7_20261006.md#pr-7603-병합과-preflight-등록-경로-보정)에 근거·명령·source·미검증을 연결했다. ignored `output/release/v0.8.7-20261006/registration-*.log/json`에 원시 결과를 보존한다.

- 보정 전 호출/정책 검사 3 FAIL, 보정 후 관련 Python 계약 66 PASS. 실제 gate의 네 source 각각 skipped/failure 거부도 PASS다.
- CI impact/policy 배선 Python 51 PASS, Node CI evidence 11 PASS, CI/reusable adapter actionlint PASS, diff check PASS.
- 구현 head의 promotion inventory: executable 16개·독립 dispatch 10개·정책 위반 0. 변경된 모든 executable 파일의 필수 검증 범위는 유지했다.
- 기존 Rust·렌더링·manifest·패키지 바이트는 변경하지 않았다. 앞선 ZIP/WASM 증거는 해당 패키지 source의 증거이며 이 PR에서 전체 Rust 검사를 새로 수행했다고 주장하지 않는다.
- 신규 Rust source/test 변경에 대한 Clippy 묶음은 비해당이다. 렌더링·LineSeg·좌표/높이·pagination/continuation·baseline 변경이 없어 조판 원칙/Visual Sweep도 비해당이다. 이 PR의 검사에서 HWP/HWPX/PDF 파일을 새로 사용하지 않았으므로 입력 커밋 증적은 비해당이다.

## 제출과 남은 절차

이 검토·오늘할일·준비 상태만 문서 trailing commit으로 같은 PR에 포함한다. code/workflow/test가 바뀌면 기존 로컬 결과를 그대로 적용하지 않는다. 문서-only head는 구현 tree 동일성과 최신 base의 merge-tree를 확인한 뒤 push한다.

최종 exact-head CI 완료·병합 판단 후 devel을 동기화하고 candidate SHA를 다시 고정한다. 그 SHA에서 승인된 수동 preflight 10개와 실제 CLI 5-platform 검증을 마쳐야 devel→main 릴리스 PR을 진행한다. 등록된 caller의 실제 remote reusable job 성공과 exact source hash 증적은 이 로컬 검사로 대체하지 않는다. 태그·publish·스토어 제출·advisory 공개는 별도 단계다.
