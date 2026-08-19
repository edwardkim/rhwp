# PR #5662 검토 - feat(agent): 구역 용지 설정을 조회하는 rhwp-q-page-def CLI를 추가한다

- PR: https://github.com/edwardkim/rhwp/pull/5662
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `4e8c179106cba1e5dda79a75e064a64392eb2a4a`
- 원본 적용 SHA: `2afd526a6dbbb687c95ad2a9f01b4002b7143189`
- 누적 검토 branch: `review/kevin9327-q-cli-round2-20260819`

## 결론

누적 통합 PR에 **수용 권고**한다. 기존 읽기 전용 `DocumentCore::get_page_def_native`로 폭·높이·여백을 JSON으로 노출하며 저장 경로를 추가하지 않는다.

## 검토 범위

- `--section` 입력, HWPUNIT 단위의 용지 정보, 범위 오류와 알 수 없는 플래그의 종료 코드 계약을 integration test로 확인했다.
- 체리픽 뒤 `agent_q_page_def_contract.rs`의 형식만 rustfmt 정본으로 보정했으며 동작·기대값은 바꾸지 않았다.

## 검증

- suite 정책: `rust-test-suite-manifest --prepare` 뒤 `--check` 통과
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 tests / 298 modules)
- formatter 및 clippy: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과
- 실행 smoke: `samples/form-01.hwp --section 0 --json`이 기대한 `tool`과 `command`를 반환
- 전체 회귀: release-test nextest **7,978/7,978 통과**, 38 skipped

## 리스크와 후속 조건

- 실제 병합은 누적 통합 PR의 최신 head와 원격 CI 성공을 대상으로 한다.
- 관련 이슈 #5657은 통합 PR 병합 뒤 수용 사실을 댓글로 남기고 close한다.

## 권고

원격 CI가 최신 head에서 통과하면 누적 통합 PR로 수용하고, 원 PR은 직접 병합하지 않는다.
