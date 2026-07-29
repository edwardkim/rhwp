# PR #3548 검토 — HWPX LineType 좌표의 core 네임스페이스 복원

- 검토일: 2026-07-29
- 작성자: [@JamesPsh](https://github.com/JamesPsh) — rhwp 첫 제출 PR
- PR: https://github.com/edwardkim/rhwp/pull/3548
- 관련 이슈: #3542
- base / 원본 head: `devel` / `c928a73689b60880f6dac9ad80d35bf96e03761c` (문서 작성 시점 참고값)
- 규모: +65 / -7, 2 files, 3 commits (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 검토 경로: collaborator 매개 외부 PR. `JamesPsh/rhwp:fix/3542-line-startpt-namespace`의
  `maintainerCanModify=true`를 확인했다.

## 변경 범위와 판정

`write_line`의 비-connector `hp:line` 분기만 `hc:startPt`/`hc:endPt`를 방출하도록 바꾼다.
`hp:connectLine`은 `ConnectPointType`의 `hp:startPt`/`hp:endPt`와 subject 참조 속성을 그대로
유지한다. 코드에서 두 분기가 명시적으로 나뉘어 있고, 기존 connector 태그·제어점 회귀도 보존된다.

이는 layout·paint가 아니라 저장 HWPX의 요소 네임스페이스 계약을 바로잡는 serializer 변경이다.
파서가 local-name으로 읽어 자체 왕복만으로는 발견하지 못하는 결함이라는 #3542의 설명과 일치한다.

## 로컬 검증

검토 worktree `review/jamespsh-20260729`에서 최신 `upstream/devel` `f2755aeb`에 원 head를
`--no-commit --no-ff`로 병합했다. 충돌 없이 병합됐고 `git diff --cached --check`도 통과했다.
모든 Cargo 실행은 `CARGO_TARGET_DIR=target/review-jamespsh-20260729`,
`CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| `issue_3542_line_startpt_namespace` | 1 passed, 0 failed |
| `cargo test --profile release-test --tests` | exit 0; library 3019 passed, 0 failed, 7 ignored 및 전체 integration 통과 |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --all-targets -- -D warnings` | passed |
| 실제 `export-hwpx --verify --verify-pages` | IR 차이 없음, 2쪽 |

실제 대상 `samples/hwpx/opengov/36392900_결재문서본문_일일굴착복구공사현황보고.hwpx`를
재저장해 `Contents/section0.xml`을 확인했다. 출력에는
`xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core"` 선언과 `hc:startPt`/`hc:endPt`만 있었고,
connectLine이 없는 이 fixture에는 잘못된 `hp:startPt`/`hp:endPt`가 없었다.

## 시각·fixture 판단

기존 HWPX fixture를 재저장하는 serializer 변경이지만, 개체 geometry·renderer·layout·페이지 수를
바꾸지 않는다. 따라서 PDF/SVG visual sweep은 merge 판단의 필수 근거로 사용하지 않았다. 대신 실제
재저장 파일의 2쪽 `--verify-pages`와 XML 네임스페이스를 확인했다. 한컴오피스의 직접 열기 거부·수용
실측은 #3542 작성자의 외부 관찰이며, 이번 검토 환경에서는 HWP 2020 재열기를 별도로 실행하지 않았다.

## 원 code candidate CI

문서 commit을 추가하기 전 원 code candidate `c928a73689b60880f6dac9ad80d35bf96e03761c`의 GitHub
Actions를 다시 확인했다. CI preflight·Lint·test archive·Native Skia·기본 기능 test 8개 shard·`Build & Test`
aggregate와 CodeQL 분석이 모두 성공했고, PR의 병합 상태는 `MERGEABLE`·`CLEAN`이었다.

## 권고와 merge 전 조건

**권고: 수용.** 위 code candidate와 동일한 head에 review-only 기록 commit을 추가한다. 그 뒤 latest
head의 preflight와 aggregate가 fast-pass 조건을 충족하는지 재확인하고, 작업지시자 승인 범위에서
merge한다. merge 뒤 #3542 auto-close 상태와 contributor 후속 comment를 확인한다.
