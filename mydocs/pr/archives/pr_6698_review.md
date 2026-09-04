# PR #6698 검토 - NBSP 전진폭

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6698](https://github.com/edwardkim/rhwp/pull/6698) |
| 작성자 | `jeong-sik` |
| base / 원 head | `devel` / `7e47ef6914edfed1852c7fff99cd04cdc71713a4` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `c0145ec66` (`-x`) |
| 관련 이슈 | [#6646](https://github.com/edwardkim/rhwp/issues/6646) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- `U+00A0`을 이미 검증된 일반 공백 측정 경로로 보내며 글꼴별 신규 상수를 추가하지 않는다.
- 회귀 테스트는 `exam_eng.hwp`의 문항 번호 뒤 실제 전진폭을 잠근다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **머지 보류**
- 해제 조건: 통합 head Rust 검증 및 `exam_eng.hwp`의 한컴 기준 간격 직접 검증.
- 원격 조치: 수행하지 않았다.
