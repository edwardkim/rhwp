# PR #5663 검토 - feat(agent): 커서 자리 ParaShape를 조회하는 rhwp-q-para-shape CLI를 추가한다

- PR: https://github.com/edwardkim/rhwp/pull/5663
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `6314db096bc11fa1b3f6c67428de059bdfe932be`
- 원본 적용 SHA: `883137cf911418076b5c541e4f117a1724e7cb86`
- 누적 검토 branch: `review/kevin9327-q-cli-round2-20260819`

## 결론

누적 통합 PR에 **수용 권고**한다. `DocumentCore::para_shape_set_json`을 읽기 전용으로 감싸고, 존재하지 않는 문단 자리도 빈 객체로 안정적으로 표현한다.

## 검토 범위

- `--list`와 `--para` 입력, `paraShape` JSON 봉투, 빈 결과와 사용법 오류의 종료 코드 계약을 `tests/cases`에서 확인했다.
- 최신 `upstream/devel@14e4d01af` 위에 원 작성자 커밋을 `-x`로 적용해 계보를 보존했다.

## 검증

- suite 정책: `rust-test-suite-manifest --prepare` 뒤 `--check` 통과
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 tests / 298 modules)
- formatter 및 clippy: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과
- 실행 smoke: `samples/form-01.hwp --list 0 --para 0 --json`이 기대한 `tool`과 `command`를 반환
- 전체 회귀: release-test nextest **7,978/7,978 통과**, 38 skipped

## 리스크와 후속 조건

- 실제 병합은 누적 통합 PR의 최신 head와 원격 CI 성공을 대상으로 한다.
- 관련 이슈 #5658은 통합 PR 병합 뒤 수용 사실을 댓글로 남기고 close한다.

## 권고

원격 CI가 최신 head에서 통과하면 누적 통합 PR로 수용하고, 원 PR은 직접 병합하지 않는다.
