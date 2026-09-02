---
kind: pr-review
status: merged
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5746 검토 - nextest archive A/B/C 분할과 worker 집계 보정

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5746](https://github.com/edwardkim/rhwp/pull/5746) / `jangster77` |
| 제목 | `perf(ci): nextest archive를 lib와 integration으로 분리` |
| base / 최종 head | `devel` / `430bd9ddceea8ca60a67f17152e220f1535f67a9` |
| merge commit | `6ff317eabe9341694ff078e0c54e06c2522e7b21` |
| 변경 규모 | 12 files, +540 / -330, 9 commits |
| 관련 이슈 | [#5737](https://github.com/edwardkim/rhwp/issues/5737), merge로 자동 종료 |
| 라우팅 | `collaborator_self_merge` + `intake_and_review` + `local_validation` + `post_merge` |

PR 본문의 초기 A/B 설명은 구현 도중의 중간 설계다. 최종 head는 library archive A와 integration archive B/C의
세 builder, 그리고 A의 두 hash worker와 B/C의 단일 worker를 사용한다.

## 변경 범위와 판정

- A는 `--lib`만 archive로 만들고 두 worker가 `hash:1/2`, `hash:2/2`로 소비한다.
- metadata로 발견한 integration target 41개를 정렬한 뒤 B에 21개, C에 20개를 교차 배정한다. 같은 target을
  양쪽 archive가 빌드하거나 실행하지 않는다.
- aggregate는 A의 두 count와 B/C의 count를 archive별 expected count에 대조하고, 총합도 함께 확인한다.
- Native Skia의 사전 설치 toolchain 삭제는 30GB 미만일 때만 수행하도록 바꿔 충분한 디스크가 있는 hosted
  runner에서 불필요한 정리 시간을 피한다.

차단 결함은 발견하지 못했다. B/C는 각 runner에서 공통 workspace 의존성을 다시 컴파일할 수 있어 runner-minute은
증가하지만, 기존 하나의 integration archive link 병목을 동시에 실행 가능한 두 builder로 바꾸는 것이 이번 PR의
명시된 wall-clock 단축 목표와 일치한다.

## 검증

### 로컬 검증

- `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py scripts/tests/test_ci_impact_workflow.py`: 42 passed
- B/C aggregate 보정 뒤 `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`: 31 passed
- 관련 Node CI impact policy/classifier 계약 test: 통과
- `cargo fmt --all` 및 `cargo fmt --all -- --check`: 통과

### GitHub Actions

[CI run #32350142195](https://github.com/edwardkim/rhwp/actions/runs/32350142195)는 최종 head에서 성공했다.

| archive | 선택 범위 | runnable tests | archive size | archive build |
| --- | --- | ---: | ---: | ---: |
| A | `--lib` | 3,893 | 27,292,348 bytes | 3분 07초 |
| B | integration 41개 중 21개 | 2,000 | 260,932,755 bytes | 12분 14초 |
| C | integration 41개 중 20개 | 1,922 | 238,665,229 bytes | 12분 18초 |

`Build & Test` 집계는 A shard 1/2, B, C의 archive count 합계와 worker 결과를 모두 성공으로 확인했다.
Lint, Native Skia, Frontend package gate도 성공했다. 병합 시점에 별도
[CodeQL run #32350141811](https://github.com/edwardkim/rhwp/actions/runs/32350141811)는 Rust 분석을 계속 실행
중이었으며, 작업지시자의 명시적 admin merge 지시에 따라 이 non-blocking run 완료를 기다리지 않았다.

## 최종 권고와 후속 처리

**병합 완료.** #5746은 2026-08-20에 squash merge됐고 #5737은 closing keyword로 자동 종료됐다. 이 기록과
오늘할일은 code PR의 CI를 다시 돌리지 않기 위해 별도 review-only docs PR로 보존한다. 원 PR의 remote head
branch 정리와 docs-only PR의 최종 cleanup은 각 PR의 merge 뒤 `post_merge.md` 순서로 수행한다.
