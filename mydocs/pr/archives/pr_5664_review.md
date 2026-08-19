# PR #5664 검토 - feat(agent): 쪽 안 개체 순환 순서를 조회하는 rhwp-q-object-cycle CLI를 추가한다

- PR: https://github.com/edwardkim/rhwp/pull/5664
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `020b784749a305ac91cc040982dabb24a115391c`
- 원본 적용 SHA: `cda599d6dba7e41fa2c622fdef491ba854d5a75a`
- 누적 검토 branch: `review/kevin9327-q-cli-round2-20260819`

## 결론

누적 통합 PR에 **수용 권고**한다. `object_cycle_json()` 결과를 조회 전용으로 노출하며, 빈 순환과 표 표본의 개체 순환을 같은 JSON 계약으로 다룬다.

## 검토 범위

- `cycle` 배열과 `cycleCount` 봉투, 알 수 없는 플래그의 종료 코드 2, 뮤테이터 비호출 계약을 integration test로 확인했다.
- 최신 `upstream/devel@14e4d01af` 위에 원 작성자 커밋을 `-x`로 적용해 계보를 보존했다.

## 검증

- suite 정책: `rust-test-suite-manifest --prepare` 뒤 `--check` 통과
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 tests / 298 modules)
- formatter 및 clippy: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과
- 실행 smoke: `samples/form-01.hwp --json`이 기대한 `tool`과 `command`를 반환
- 전체 회귀: release-test nextest **7,978/7,978 통과**, 38 skipped

## 리스크와 후속 조건

- 실제 병합은 누적 통합 PR의 최신 head와 원격 CI 성공을 대상으로 한다.
- 관련 이슈 #5661은 통합 PR 병합 뒤 수용 사실을 댓글로 남기고 close한다.

## 권고

원격 CI가 최신 head에서 통과하면 누적 통합 PR로 수용하고, 원 PR은 직접 병합하지 않는다.
