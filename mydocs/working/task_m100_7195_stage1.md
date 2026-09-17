# #7195 1단계 — 통과한 ignored 조판 계약 2건 복귀

- Issue: [#7195](https://github.com/edwardkim/rhwp/issues/7195)
- 계획: [승인된 수행계획서](../plans/task_m100_7195.md)
- 날짜: 2026-09-16
- 기준 SHA: `8d45f242baa1a565357aaa38e9f459595b1e756c`
- 작업 branch: `task_m100_7195`
- 검증 worktree: `/home/edward/mygithub/rhwp-review-7195`
- 상태: 1단계 구현·집중 검증 완료. 제품 조판 코드·기대값·샘플·baseline은 변경하지 않았다.

## 1. 변경과 검증 계약

| 대상 | 변경 | 유지한 assertion |
| --- | --- | --- |
| `test_552_passage_box_top_gap_p2_4_6` | ignore 제거, optional 로더 대신 read/parse/render 오류를 명시적으로 실패시킴 | 물리 p2 오른쪽 단의 `[4~6]` header와 박스 상단 탐색, gap ≥ 6px |
| `test_task77_image_cell_no_intra_row_split` | ignore 제거, 샘플 부재 return 제거, 과거 ignore 사유 주석 정리 | 표6의 두 조각 rows 0..2 / 2..4, end/start cut, 두 페이지 그림 소유 |

두 테스트만 엄격한 입력 계약으로 바꿨다. 공용 optional 로더와 다른 테스트의 skip 계약은 변경하지 않았다.
#552의 `render_page_svg_native` 오류도 빈 문자열로 숨기지 않고 원인과 함께 실패시킨다.
기존 테스트의 좌표 탐색·임계값을 유지한 것이며, 현재 한컴과 정확히 일치한다는 새 시각 판정은 아니다.

## 2. 음성 대조

기존 devel 검증에서 사용한 lib binary에 `--exact <이름> --ignored --nocapture`를 전달하고,
별도 임시 작업 폴더에서 각 테스트의 상대 경로 입력을 누락하거나 비-HWP bytes로 제공했다.
수정본은 같은 방법으로 `--ignored` 없이 실행한다. 원본 샘플과 다른 작업의 파일은 건드리지 않는다.

| 테스트 | 입력 | 변경 전 | 변경 후 |
| --- | --- | --- | --- |
| #552 | 누락 | exit 0: 거짓 통과 | exit 101: 명시적 읽기 실패 |
| #552 | 비-HWP bytes | exit 0: 거짓 통과 | exit 101: 명시적 파싱 실패 |
| #77 | 누락 | exit 0: 거짓 통과 | exit 101: 명시적 읽기 실패 |
| #77 | 비-HWP bytes | exit 101: 파싱 실패 | exit 101: 명시적 파싱 실패 |

이 음성 대조는 **입력 오류 검출**의 전후 증거다. 이전 조판 결함 자체를 RED/GREEN으로 재현했다는
의미는 아니다. 조판 assertion은 앞서 두 devel SHA에서 이미 통과했다.

## 3. 검증 명령과 결과

검증 작업은 공유 고정 target `/home/edward/mygithub/rhwp/target/pr-review`에서 순차 수행한다.
현재 host는 16 logical CPU / 31 GiB RAM이며 빌드 jobs 4, 집중 테스트 threads 1을 사용했다.

```sh
# /home/edward/mygithub/rhwp-review-7195
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check --base-ref 8d45f242baa1a565357aaa38e9f459595b1e756c
node scripts/rust-unit-test-tiers.mjs --check --base-ref 8d45f242baa1a565357aaa38e9f459595b1e756c

CARGO_BUILD_JOBS=4 cargo nextest run --locked \
  --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  -p rhwp --lib --test-threads 1 --no-fail-fast --success-output immediate \
  -E 'test(=renderer::layout::integration_tests::tests::test_552_passage_box_top_gap_p2_4_6) | test(=wasm_api::tests::test_task77_image_cell_no_intra_row_split)'
```

- 전용 worktree의 fmt: PASS.
- suite 정책: PASS, 1,340 sources / 28 suites + 20 exceptions. 생성 파일은 커밋하지 않는다.
- base 대비 unit-tier: PASS, 4,205 tests / 298 modules.
- 집중 nextest: **2 PASS / 0 FAIL**, 0.082초, 빌드 3분 31초, exit 0.
  Run ID: `0ac64840-80c2-4ac7-97e6-5f50576c2ec5`.
- 기본 nextest 목록: 두 테스트 모두 `ignored: false`, `filter-match.status: matches`.
  lib에는 3,886개 테스트와 ignored 11개가 있다. 집중 실행의 `3884 skipped`는 이름 필터에 따른 제외이며
  전체 회귀 ignored 수가 아니다. workspace 전체 제외 수는 이번에 재집계하지 않았다.
- 입력·source 바이트 동일성 및 손상/누락 입력 음성 대조: PASS. 네 음성 대조 모두 해당 읽기/파싱
  오류 메시지와 exit 101을 확인했으며, 의도한 실패 검출을 검사한 실행 자체는 성공이다.

환경 경고: 설치된 nextest는 0.9.137로 권장 0.9.140보다 낮으며
`profile.ci-duration-observation.junit.report-skipped` 키를 무시한다는 경고가 있었다.
이번 기본 profile의 정확한 이름 선택·실행 결과는 직접 확인했다. CI-duration 관측 profile이나
최신 권장 버전에서의 전체 검증까지 수행했다는 의미는 아니다. 도구 버전은 이 단계에서 바꾸지 않았다.

### 기본 작업 폴더의 포맷 검사 문제

최초 기본 폴더에서의 `cargo fmt --all -- --check`는 기존 #7090 파생 suite가 현재 branch에 없는
`tests/cases/issue_7090_sibling_table_occupancy.rs`를 참조하여 실패했다.
이는 이번 변경의 Rust formatting 실패가 아니라 남아 있던 파생 파일의 참조 문제다.
기존 #7090 증적·파생 파일을 삭제하지 않고 #7195 전용 검증 worktree에서 현재 base의 suite를
준비하여 fmt를 통과했다. 기본 폴더의 전체 포맷 검사까지 성공했다고 기록하지 않는다.

## 4. 입력·소스 증거와 미검증 범위

수정 소스 SHA-256:

- `src/renderer/layout/integration_tests.rs`: `0f10468de946d2d0ee2806dbf4d76a4d983ea2071270bfac3892fba43792e16c`
- `src/wasm_api/tests.rs`: `4b6f38c1f967a4d86ab1521b399379d4fda07d6b37921bce70fbf8718e75a6ab`

위 두 소스는 기본 폴더와 검증 worktree에서 동일함을 확인했다. 검증은 위 base에 이 테스트 diff를
더한 후보를 대상으로 하며, 이후 문서만 추가하여 단계 commit으로 보존한다.

실행 스크립트와 로그는 로컬 `output/7195/stage1/`에 둔다.
`verify.mjs`, `before-guards.json`, `before-*.log`는 음성 대조의 실행 인자·작업 폴더·binary hash를 남긴다.
`focused.log`, `focused-result.json`, `test-list.json`, `after-results.json`, `after-*.log`에
정상 실행, 기본 선택 상태, 입력/source/binary hash, 음성 대조를 저장했다.

커밋 입력과 기본/검증 worktree의 샘플 bytes가 동일함을 확인했다:

- `samples/21_언어_기출_편집가능본.hwp`: 435,200 bytes,
  SHA-256 `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`
- `samples/20250130-hongbo.hwp`: 643,072 bytes,
  SHA-256 `4062580dbe01654a903c88a33ac2443ba1682b9d00aeb324b749f9a902f47257`

이번 단계에서는 전체 회귀, Clippy 3종, WASM 빌드, 새 시각 검증을 수행하지 않았다.
PR 준비 완료나 이슈 해결 완료가 아니다. 실패 3건의 ignore와 기대값은 유지했고 재실행하지 않았다.
다음 단계는 #2279 / #2308 / #3798의 정밀 조사이며, 제품 수정은 조사 후 별도 구현 승인을 받는다.
