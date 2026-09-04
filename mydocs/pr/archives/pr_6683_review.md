# PR #6683 검토 - 개체 전용 셀 높이

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6683](https://github.com/edwardkim/rhwp/pull/6683) |
| 작성자 | `jeong-sik` |
| base / 원 head | `devel` / `e5dde4373ed0c8d26543482c8031b0e2aa556baa` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `dd8ca73a2`, `4c333ab94` (`-x`) |
| 관련 이슈 | [#6660](https://github.com/edwardkim/rhwp/issues/6660) |

## 검토 결과

- local 통합 전에 reviewer `jangster77`을 지정했다.
- 선정 시 원 head는 `MERGEABLE/CLEAN`이었고 CI, CodeQL, Render Diff, Adapter inter-diff,
  Proptest는 성공 또는 정책상 skip이었다.
- 개체가 선언 셀 높이를 넘는 단일 문단 셀에서만 빈 문단 줄 높이를 제외하도록 좁혀 `#6312`의
  반대 사례를 보존한다.
- `exam_science.hwp`의 영향 행 높이를 검증하는 회귀 테스트가 포함된다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **머지 보류**
- 해제 조건: 통합 head Rust 검증과 `exam_science.hwp` 4쪽의 한컴 기준 직접 시각 검증.
- 원격 조치: comment, close, push, PR 생성, approve, merge를 수행하지 않았다.
