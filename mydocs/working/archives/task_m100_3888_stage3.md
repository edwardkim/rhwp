# Task M100 #3888 3단계 — shard별 nextest archive 전송 분할

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 기준 head: `c1e442b83`

## 문제 확인

PR #3892의 첫 실행에서 `test-archive-30813876861`은 1,350,230,657 bytes였다. 8개 worker가
모두 같은 artifact를 내려받았고, slow worker의 download step만 165초가 걸렸다. 실행 대상만
`hash:m/7`로 나누어도 artifact 자체는 줄지 않으므로, worker마다 약 1.35GB를 전송하는 구조가
남는다.

nextest 0.9.140의 `archive -E`는 선택한 **test binary**만 archive에 넣을 수 있다. 다만
archive filterset은 `test()` predicate를 수용하지 않는다. 따라서 기존 test-name hash partition을
그대로 archive 분할에 사용할 수는 없다. 같은 binary의 서로 다른 테스트가 여러 hash worker에
걸릴 수 있기 때문이다.

## 구현

`.github/scripts/allocate_nextest_archive_shards.mjs`는 `cargo nextest list --message-format json`
결과에서 비-ignored 테스트 수를 읽는다.

1. `overflow_cell_lines_do_not_grow`는 slow archive에 단독 실행 대상으로 남긴다.
2. 나머지는 binary 단위로 내림차순 정렬하고, 현재 runnable 수가 가장 적은 일반 shard에 하나씩
   배정한다.
3. slow test가 든 binary는 slow archive와 일반 archive에 한 번씩 포함한다. 일반 worker의
   filterset은 해당 exact test를 제외하므로 실행은 중복되지 않는다.
4. build job은 `slow`, `1`~`7`의 `.tar.zst` 8개를 별도 GitHub artifact로 upload한다.
   각 test worker는 matrix의 `archive_label`에 해당하는 artifact 하나만 download한다.
5. `Build & Test`는 8개 worker의 Summary `run` 합계와 planner가 계산한 전체 runnable 수를
   대조한다. 따라서 binary 배정 또는 archive 전달 오류가 테스트 누락·중복 green으로 보이지 않는다.

이 방식은 build job에서 archive를 8번 생성하므로 archive 준비 시간이 일부 늘 수 있다. 반대로
worker download는 전량 1.35GB가 아니라 자신의 binary 집합으로 제한된다. 최종 CI에서 archive별
바이트 수, download 시간, build job 증가분과 전체 critical path를 함께 비교한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `node --check .github/scripts/allocate_nextest_archive_shards.mjs` | 통과 |
| synthetic nextest JSON: slow 1건 분리 | 통과 |
| synthetic nextest JSON: 일반 binary 중복 배정 없음 | 통과 |
| synthetic nextest JSON: 일반 7건 + slow 1건 = runnable 8건 | 통과 |
| 생성 filter에 `test()` predicate 없음 | 통과 |
| `actionlint .github/workflows/ci.yml` | 통과 (v1.7.12) |
| `git diff --check` | 통과 |

제품 Rust·TypeScript·fixture는 수정하지 않았다. workflow와 planner 변경이므로 Cargo 제품 테스트는
로컬에서 재실행하지 않으며, 실제 archive 생성·download 크기·8 worker 실행 합계는 최신 GitHub
Actions에서 검증한다.
