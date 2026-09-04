# PR #6709 검토 - 용지 기준 Square 그림 이동 뒤 본문 재투영

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6709](https://github.com/edwardkim/rhwp/pull/6709) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `36b5500891e750be7680c2559e2c278d4cbbe175` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `e6b9a3ed5`, `ffd47191e` (`-x`) |
| 메인터너 보정 | `872f3d4c5` 및 정식 fixture 등록 후속 commit |
| 관련 이슈 | [#6202](https://github.com/edwardkim/rhwp/issues/6202) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- `PaperOrigin`으로 용지 기준 좌표를 host band의 로컬 좌표로 옮겨 그림 이동 뒤 본문을
  새 배제 밴드로 재투영한다.
- 기존 private-path 탐색과 fixture 부재 성공 처리는 제거한다. 원본 HWP는
  `samples/issue6202/`에 SHA-256 manifest와 함께 등록한다.
- 그림 이동 API 오류를 성공 처리하던 경로는 `872f3d4c5`에서 fail-closed로 보정했다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **메인터너 보정 후 수용 가능**
- 원 head: private fixture와 편집 실패를 엄격히 검증하지 못한다.
- 보정 뒤 통합 head: 정식 fixture와 fail-closed 테스트를 포함한다.
- 수용 전 조건: 통합 Rust 검증과 그림 이동 전후의 직접 시각 증적.
- 원격 조치: 수행하지 않았다.
