# PR #7605 검토 — Oracle sparse checkout 제품 빌드 입력 보정

## 최종 판정

**머지 보류 — 최종 head CI 확인 중.** 로컬 보정과 적용 범위 검증을 완료했다. 필수 CI 완료와 사용자 병합 판단을 남기며 릴리스 promotion 통과로 표현하지 않는다.

## 접수와 경로

- [PR #7605](https://github.com/edwardkim/rhwp/pull/7605), 작성자 edwardkim, base devel의 self-review다. collaborator_self_merge와 intake_and_review·local_validation 경로를 적용하며 본인 PR에 reviewer를 지정하지 않는다.
- base `d6c3286a74a771ed4b1d4aec9febf50d7b89f83d`, 구현·최초 게시 head `c687bfd1fa9b5a061891b9ebab5d3cc6c97c4ccd`.
- 사용자 승인 후 원본 upstream 작업 브랜치로 push·Open PR 등록했다. 최초 API 조회는 draft false·4파일·100줄 추가/4줄 삭제·MERGEABLE/BLOCKED였다. 한국어 본문 exact 일치와 선두 BOM·`??` 치환 부재를 확인했다. 상태는 조회 시점 정보다.
- main `680111ec7bea2fe11110de18c3676ba5a1cf7847`의 devel ancestry, 최신 base와 후보 merge-tree exit 0을 확인했다. 자동 CI가 실제로 시작됐으며 최종 문서 head 결과는 따로 확인한다.

## 원인과 변경 경로

[Oracle run 37416229026](https://github.com/edwardkim/rhwp/actions/runs/37416229026)은 sparse checkout에서 `vendor`를 받지 않았다. root `Cargo.toml:[patch.crates-io]`의 `svg2pdf.path`가 제품 dependency resolver에 소비되면서 누락된 `vendor/svg2pdf/Cargo.toml` 때문에 exit 101로 즉시 종료했다. timeout이 아니다.

workflow sparse 목록에 `vendor`를 포함한다. 제품 source·manifest·lockfile·렌더링 규칙은 바꾸지 않는다. 회귀 검사는 실제 Cargo manifest의 모든 path patch가 checkout 목록의 동일 경로나 상위 경로에 포함되는지 확인한다. Cargo 입력이 바뀌어도 특정 dependency 이름만 확인하는 검사로 남지 않는다.

기존 advisory `continue-on-error`·권한·트리거·timeout 및 promotion의 accepted verdict 정책을 유지했다. Actions 성공만으로 build 실패를 수용하지 않는다. 이전 exact SHA에서 10개 Actions run은 success였지만 실제 collector/verifier는 executable 16개 중 15개만 수용하고 Oracle `build-failed`를 거부했다.

## 검증과 한계

[릴리스 준비 기록](../../plans/release_0_8_7_20261006.md#pr-7604-병합-후-실제-preflight와-oracle-입력-누락)에 실제 run·명령·결과를 연결한다. 원시 증거는 ignored `output/release/v0.8.7-20261006/oracle-*.log/json`와 `preflight-verdict-final.json`이다.

- 새 회귀 검사를 workflow 수정 전에 실행해 누락된 path patch 원인으로 1 FAIL을 확인했다. 수정 후 Oracle·release operations·promotion preflight/evidence/gate·release channel의 Python 계약 56개가 PASS였다.
- 실제 workflow sparse 목록을 독립 clone에 적용해 CLI build exit 101/missing vendor manifest를 재현했다. 수정된 목록을 적용한 뒤 `cargo build --locked --profile release-test --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review` exit 0, `rhwp --version`의 `rhwp v0.8.7`을 확인했다.
- clone의 제품 source SHA는 base `d6c3286a74a771ed4b1d4aec9febf50d7b89f83d`이고 tracked clean이다. 이 PR은 제품 source를 바꾸지 않으므로 실제 변경은 checkout 입력 공급이다. local release-test 결과를 remote release LTO·실제 Oracle 비교 성공으로 대신하지 않는다.
- 변경 workflow actionlint PASS, diff check PASS, main 대비 promotion inventory 정책 위반 0. source head의 workflow SHA-256은 `2ff371bb44cdac644dcecfec5512f84232707180f174bddfd919666ee5ee7206`이다.
- 조판 원칙·Visual Sweep·baseline 변경은 비해당이다. Rust source/test 변경이 없어 신규 Rust lint 묶음도 비해당이다. 컴파일에 소비한 커밋된 `saved/blank2010.hwp`는 base SHA의 제품 include 입력이며 한컴 조판 검증에 사용하지 않았다. 새 HWP/HWPX/PDF fixture는 없다.

## 남은 절차

검토·오늘할일·게시 상태만 문서 trailing commit으로 같은 PR에 포함한다. 실행 파일과 test가 동일함을 확인한 뒤 승인된 branch로 push한다. 최신 head CI·base/mergeability·사용자 병합 판단을 확인한 뒤 devel에 통합한다. 새 candidate SHA를 고정하고 필수 preflight를 다시 실행해 실제 Oracle verdict를 확인해야 main 승격을 진행한다. 태그·publish·스토어 제출·advisory 공개는 별도 단계다. contributor 시각 comment는 이 self PR에 비해당이다.
