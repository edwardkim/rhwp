# PR #5668 검토 - feat(agent): 조회 CLI 50개를 rhwp-q-kit에 모은다

- PR: https://github.com/edwardkim/rhwp/pull/5668
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `b3881178f823326c40eb017b7938f0a29bee7b45`
- 원본 적용 SHA: `2a894e667c3d983b754d17fc4d1d05bb09afe458`
- 누적 검토 branch: `review/kevin9327-q-cli-round2-20260819`

## 결론

메인터너 보정 포함으로 누적 통합 PR에 **수용 권고**한다. 50개 조회 명령은 읽기 전용 `DocumentCore` 질의로만 구성되고, 보정 뒤 각 하위 명령도 독립적으로 `--help`를 성공 종료한다.

## 검토 범위

- `rhwp-q-kit`의 전역 명령 목록, JSON 봉투, 빈 문서·하이퍼링크·알 수 없는 명령과 플래그의 오류 계약을 확인했다.
- 54개 파일, 5,492줄의 원 PR은 최신 `upstream/devel@14e4d01af` 위에 원 작성자 커밋을 `-x`로 적용해 계보를 보존했다.

## 메인터너 보정

- 원 구현은 전역 `--help`만 처리하고 하위 명령의 `--help`를 각 handler에 넘겨 종료 코드 2로 끝냈다.
- dispatcher가 등록된 명령의 `--help`와 `-h`를 공통 usage로 처리하도록 보정했다.
- 계약 테스트가 전역 help에 표시한 50개 명령 전부를 실제로 `<command> --help`로 실행해 종료 코드 0과 usage 표기를 확인한다.

## 검증

- suite 정책: `rust-test-suite-manifest --prepare` 뒤 `--check` 통과
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 tests / 298 modules)
- formatter 및 clippy: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과
- focused contract: `agent_q_kit_contract` **6/6 통과**
- 실행 smoke: 단건 q CLI 4개 JSON 조회와 q-kit 하위 명령 **50/50**의 `--help` 통과
- 전체 회귀: release-test nextest **7,978/7,978 통과**, 38 skipped

## 리스크와 후속 조건

- 실제 병합은 누적 통합 PR의 최신 head와 원격 CI 성공을 대상으로 한다.
- 관련 이슈 #5667은 통합 PR 병합 뒤 수용 사실을 댓글로 남기고 close한다.

## 권고

원격 CI가 최신 head에서 통과하면 누적 통합 PR로 수용하고, 원 PR은 직접 병합하지 않는다.
