# PR #5659 검토 - feat(agent): 한글 구역 시작 문단을 조회하는 rhwp-q-section-starts CLI를 추가한다

- PR: https://github.com/edwardkim/rhwp/pull/5659
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `8002d658969cdbc65847f954ac36a2de77b42098`
- 원본 적용 SHA: `b1ce8f6973dd8687802fa9255a6445fec231d5d3`
- 누적 검토 branch: `review/kevin9327-q-cli-round2-20260819`

## 결론

누적 통합 PR에 **수용 권고**한다. 읽기 전용 `section_starts_json()`만 호출하며 문서 변경 경로와 기존 CLI 표면을 건드리지 않는다.

## 검토 범위

- `rhwp-q-section-starts`의 JSON 봉투, 구역 시작 문단 배열, 잘못된 플래그의 사용법 종료 코드 계약을 `tests/cases` integration contract로 확인했다.
- 최신 `upstream/devel@c2a36398d` 위에 원 작성자 커밋을 `-x`로 적용해 계보를 보존했다.

## 검증

- suite 정책: `rust-test-suite-manifest --prepare` 뒤 `--check` 통과
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 tests / 298 modules)
- formatter 및 clippy: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과
- 실행 smoke: `samples/form-01.hwp` JSON 조회가 기대한 `tool`과 `command`를 반환
- 전체 회귀: release-test nextest **7,978/7,978 통과**, 38 skipped

## 리스크와 후속 조건

- 실제 병합은 누적 통합 PR의 최신 head와 원격 CI 성공을 대상으로 한다.
- 관련 이슈 #5656은 통합 PR 병합 뒤 수용 사실을 댓글로 남기고 close한다.

## 권고

원격 CI가 최신 head에서 통과하면 누적 통합 PR로 수용하고, 원 PR은 직접 병합하지 않는다.
