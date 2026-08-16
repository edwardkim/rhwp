# PR #4924 검토 - 실물 대형 문서 scale ladder 측정

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4924](https://github.com/edwardkim/rhwp/pull/4924) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합을 위한 archive review |
| base / head | `devel` / `docs/r55-large-document-limits` |
| source candidate | `a651d4656afba01ac695e91b65e8cb26a7689f2f` |
| 통합 commit | `aedb512edc1904fbcdf0e7251b0f023d5434d5f8` |
| 규모 | 5 files, +1,368 / -14 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- 합성 입력 대신 실물 HWPX 코퍼스와 반복 실행을 사용해 대형 문서의 parse·export scale limit을 기록하는
  표준 라이브러리 기반 harness를 추가한다.
- 최대 RSS는 Windows 관측값이라는 플랫폼 범위를 명확히 해, 다른 OS에서 측정되지 않은 값을 일반화하지 않도록
  #4931 메인터너 보정에서 표기를 정정했다.
- 측정은 product hard limit을 바로 변경하지 않고, 압축 해제 바이트와 time wall을 후속 guard 설계의 근거로
  남긴다.

## 검증

- source candidate의 Build & Test, Native Skia, CodeQL, 기본 feature 세 shard·slow shard 및 lint가 성공했다.
- frontend/WASM job은 변경 영향에 따라 skipped였다.
- #4931 누적 tree에서 `python3 -m py_compile tools/scale_ladder_real.py`와 전체 `release-test` integration
  회귀를 종료 코드 `0`으로 완료했다.

## 위험과 권고

이 PR의 수치는 특정 Windows 호스트·실행 환경에서의 관측이며 플랫폼 독립 성능 보장은 아니다. 하네스와
측정 기록의 범위가 이를 정직하게 구분하므로 #4931 통합 merge를 권고하며, 원 PR은 merge 뒤 supersede 처리한다.
