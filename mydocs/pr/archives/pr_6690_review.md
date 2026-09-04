# PR #6690 검토 - 개체 전용 마지막 줄의 꼬리 줄간격

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6690](https://github.com/edwardkim/rhwp/pull/6690) |
| 작성자 | `jeong-sik` |
| base / 원 head | `devel` / `c379257716458c30028dbd44f84ce8b463c0b96d` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `eb84bbbc7`, `9f0455b6f` (`-x`) |
| 관련 이슈 | [#6681](https://github.com/edwardkim/rhwp/issues/6681) |

## 검토 결과

- reviewer `jangster77`을 지정했고, 원 head의 required CI는 선정 시 성공 또는 정책상 skip이었다.
- 개체만 든 마지막 줄에서는 꼬리 줄간격을 더하지 않되, 다문단 글자처럼 표의 기존 예외는 보존한다.
- 동작 commit과 후행 clippy 정리 commit을 원래 순서대로 모두 적용했다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **머지 보류**
- 해제 조건: 통합 head Rust 검증 및 `exam_science.hwp` 4쪽 표 아래 baseline의 직접 시각 검증.
- 원격 조치: 수행하지 않았다.
