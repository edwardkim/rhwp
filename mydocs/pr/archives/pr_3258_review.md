# PR #3258 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3258](https://github.com/edwardkim/rhwp/pull/3258) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `94a55ccfeb0e5f32ea38a4763205561e5b59bd93` (2026-07-25 조회 참고값) |
| 규모 | +997/-8, 7 files, 3 commits |
| 관련 이슈 | #3237, #3238 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 메인터너 보정 | `1b9cf58fd` — `info` 다중 입력을 사용법 오류로 처리 |
| 판단 | 보정 적용 완료, 최신 full CI 대기 |

## 범위와 검토

- `info`·`export-text`의 JSON 출력과 `batch` 병렬 스트리밍을 추가한다.
- `upstream/devel` `fa953ffa6` 위 가시성 검토 branch
  `review/kevin9327-cli-json-20260725-v2`에서 원 feature `d99e76da2`를 누적 적용했다.
- 원 구현은 `info <file> --json`에 두 파일을 넘겼을 때 성공 종료하고 첫 파일만 사용했다. 자동화 CLI에서
  입력 누락은 데이터 손실이므로 사용법 오류여야 한다.

## 보정과 검증

- `1b9cf58fd`는 `info`가 정확히 한 파일만 받도록 하고, 다중 입력은 stdout 없이 `EXIT_USAGE`로 끝내는
  회귀를 추가했다. contributor 원 변경을 rewrite하지 않은 별도 메인터너 commit이다.
- 보정 source branch에서 `CARGO_TARGET_DIR=target/kevin9327-20260725-review CARGO_INCREMENTAL=0`
  `cargo test --profile release-test --test cli_json_contract --quiet`를 성공했다. 누적 검토 branch의
  `cargo test --profile release-test --tests --quiet` 전체도 성공했다.
- renderer·layout·sample·PDF/golden은 바뀌지 않아 visual sweep과 IR field-sweep baseline은 대상이 아니다.

## 권고와 다음 조건

- **권고: 수용.** 이 code/test commit 뒤의 최신 head full CI, mergeability, 사용자 merge 승인을 재확인한 뒤에만
  #3262보다 먼저 merge한다.
- 문서·오늘할일만 담은 후속 commit은 code 보정 commit과 분리했다. code/test가 있으므로 review-only fast-pass가
  아니라 full CI가 필요하다.
