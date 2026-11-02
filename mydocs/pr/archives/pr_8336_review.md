# PR #8336 리뷰 — fix: HWPX 읽기·쓰기가 쪽 번호 줄표와 셀 세로쓰기(영문 세움)를 보존한다 (#7633, #8334)

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `2d17f00451` → `eba13ba3a4`, `eb6ae84654` → `b2b0100674`, `7b2495fb8b` → `b426c12d8a`, `ccd0256d6b` → `f7ab73dc54`, `1c8edc9b07` → `a090f19131`, `e6bf361188` → `054e0c49a5`, `4e004d650a` → `a2541933cb`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_7633_8334_hwpx_attr_roundtrip` 5건, `issue_7633_hwpx_page_number_side_char` 2건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [#8336](https://github.com/edwardkim/rhwp/pull/8336), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#7633](https://github.com/edwardkim/rhwp/issues/7633), [#8334](https://github.com/edwardkim/rhwp/issues/8334).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

쪽 번호 줄표 없음과 영문 세움 셀 세로쓰기 속성을 HWPX 읽기·쓰기에서 보존한다.
쪽 번호 문자와 셀 방향의 서로 다른 저장·재열기 경로를 확인한다.

## 체리픽과 보정

- 원본 `2d17f00451d9989f6ef33deb39d2b8fc34a90efa` → 통합 `a740f282e2986611fe83b9cbcf07b146c72f3345`.
- 원본 `eb6ae8465403d554e20c509f5ad84edcaffad563` → 통합 `38aaa6e39f3d05b67530efa9f28e3e48ed0694a4`.
- 원본 `7b2495fb8b2705b32ce290c44b25ef8eff4b5777` → 통합 `0737f4adb7b31a7df646fadfda63893abdb78091`.
- 원본 `ccd0256d6b79a3dfe777f960ba067c64a756bb87` → 통합 `da05a76bb4d98d4859f372753b91588f3d4e1588`.
- 원본 `1c8edc9b079e0357671b87fc9c1cd99f8dbc4027` → 통합 `eaf5d44d01a26f435e545f653042504536c7f3c4`.
- 원본 `e6bf3611882010740717dd965677b775054bc4e2` → 통합 `c8436fcf5dedebf06a98cd29f6d5764989c4415d`.
- 원본 `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e` → 통합 `abbdd6e3c770ce6522b09e3e77a3259ecc444406`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_7633_8334_hwpx_attr_roundtrip.rs`](../../../tests/cases/issue_7633_8334_hwpx_attr_roundtrip.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.
- [`issue_7633_hwpx_page_number_side_char.rs`](../../../tests/cases/issue_7633_hwpx_page_number_side_char.rs): 이전 실행 2건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`issue6029-3200477_icao_procedure-2024.pdf`](../../../pdf/issue6029-3200477_icao_procedure-2024.pdf) — SHA-256 `75ce9ab871a588ecbc8bd07d3ca05a61a8c4f70c2487cd66983b70f950d6d1d0`.
- [`148751598-briefing-rhwp-hwpx-2024.pdf`](../../../pdf/page-number-side-char/148751598-briefing-rhwp-hwpx-2024.pdf) — SHA-256 `9384c32e21d1cdef626af78147c61be96ee3bebc46811a5d557668afdc1b6ae1`.
- [`hwp-3.0-HWPML-rhwp-hwpx-2024.pdf`](../../../pdf/page-number-side-char/hwp-3.0-HWPML-rhwp-hwpx-2024.pdf) — SHA-256 `f145f4e5655febfc210bc36e96e74e78a39e42f4e546e1c5bc099090cc23eb20`.

기여자가 제출한 기준 자료: [`hwpml_hwpx_devel_silhouette.tsv`](../../../mydocs/pr/assets/issue_7633_8334/native/hwpml_hwpx_devel_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.
