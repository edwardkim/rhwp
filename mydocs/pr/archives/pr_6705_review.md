# PR #6705 검토 - 이어진 문단의 떠 있는 그림 앵커

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6705](https://github.com/edwardkim/rhwp/pull/6705) |
| 작성자 | `jeong-sik` |
| base / 원 head | `devel` / `05325df7c4350b101276580803a208c62709c05a` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `7b15d9582`, `dda7902e5` (`-x`) |
| 관련 이슈 | [#6704](https://github.com/edwardkim/rhwp/issues/6704) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- 앞쪽에서 이어진 `vert=Para` 그림은 기존 `PartialParagraph { start_line > 0 }` 의미에 맞춰
  현재 쪽 본문 상단을 기준으로 놓는다.
- 회귀 테스트는 고쳐진 큰 그림과 움직이면 안 되는 같은 쪽 바닥글 그림을 함께 확인한다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **머지 보류**
- 해제 조건: 통합 head Rust 검증 및 `hwp3-sample.hwp` 7쪽 한컴 기준 직접 시각 검증.
- 원격 조치: 수행하지 않았다.
