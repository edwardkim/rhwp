# PR #6703 검토 - HWP5 near-top 리셋 범위 축소

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6703](https://github.com/edwardkim/rhwp/pull/6703) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `219868e86f94b47f0b033bf2b50d64ca655ef8d0` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `ceadaf94a`, `2dd41febf` (`-x`) |
| 관련 이슈 | [#5941](https://github.com/edwardkim/rhwp/issues/5941) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head의 required CI는 선정 시 성공 또는 정책상 skip이었다.
- HWP5 저장 조판에서는 이미 찬 쪽에만 저장 near-top 리셋을 유지하고, HWPX 및 거의 빈 쪽의
  기존 완화 계약을 음성 대조로 남긴다.
- 이제 정식 fixture가 아닌 외부 대형 corpus 입력은 통합 검증 시 별도 근거로만 사용한다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **머지 보류**
- 해제 조건: 통합 head Rust 검증 및 채운 쪽/빈 쪽 두 축의 직접 시각 증적.
- 원격 조치: 수행하지 않았다.
