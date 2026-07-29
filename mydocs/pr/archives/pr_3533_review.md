# PR #3533 검토 기록 — HWP3 문단 char_count 규약 정합

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3533](https://github.com/edwardkim/rhwp/pull/3533) — HWP3 `char_count` 문단 끝 마커 누락 (#3510) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `8ae1a3cc84850a008e402c2b2bdf33d94ca888da` / `9cc72451f827564515e6e09e142b4892dcd41fb5` |

HWP5/HWPX와 달리 HWP3 두 문단 생성 경로가 끝 마커 `0x000D`를 `char_count`에 넣지 않아, 내용은
같아도 HWP3→HWPX `--verify`가 off-by-one IR diff를 대량 보고하던 결함을 고친다. 본문과 제목차례
장식 inject 경로 모두 같은 규약으로 `+1` 처리한다.

## 검증과 판정

- `hwp3_charcount_convention`: 3 passed — HWP3의 정확히 1 차이 패턴 제거·diff 감소와 HWP5 무회귀를
  검증했다.
- 통합 clippy가 test helper의 `or_else(|| None)` no-op closure를 `-D warnings`로 거부했다. 의미 변경
  없이 제거한 메인터너 commit `e0efa3ea3` 후 clippy gate를 재실행해 통과했다.
- 남는 구역 시작 인접 차이는 char_count 결함과 다른 원인으로 구분돼 있으며, 이 변경은 해당 범위를 넓히지 않는다.
- 통합 후보의 release library·전체 test gate에서 HWP3 parser 회귀를 추가 확인한다.

문단 문자·control 정렬의 기반 규약을 다른 포맷과 맞추고 범위를 좁게 유지하므로 **기술적 수용 가능**이다.
