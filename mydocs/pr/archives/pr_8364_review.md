# PR #8364 리뷰 — fix(hwpx): HWP3 출처 HWPX의 줄 시작을 한컴과 같은 문단 축으로 저장한다 (#8303)

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `15a1018ee1` → `745901788f`, `b3286debf7` → `d55301451c`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8303_hwp3_hwpx_textpos_idempotent` 3건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [#8364](https://github.com/edwardkim/rhwp/pull/8364), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `b3286debf7b1eb6e0a7886f6f7ee954672643dcb`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `b3286debf7b1eb6e0a7886f6f7ee954672643dcb`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8303](https://github.com/edwardkim/rhwp/issues/8303).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP3 계보 HWPX의 줄 시작을 한컴의 문단 축에 맞게 저장한다.
원본 HWP3·저장 HWPX의 줄 시작과 반복 저장을 확인한다.

## 체리픽과 보정

- 원본 `15a1018ee1c1bda85a728ac6849f06be2a11c204` → 통합 `198cdd0f5906cf2173a960f994039d8816f867e7`.
- 원본 `b3286debf7b1eb6e0a7886f6f7ee954672643dcb` → 통합 `043374e265943ce0bf12c9e9e22c65d2900f11fa`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

반복 저장 코퍼스에서 사용자가 이번 검토 범위에서 제외한 sample16과 미해결 #7807의 `SO-SUEOP.hwp`를 제거했다. 다른 입력과 한컴 기준 `hwp3-table-caption`의 줄 시작 검사는 유지한다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8303_hwp3_hwpx_textpos_idempotent.rs`](../../../tests/cases/issue_8303_hwp3_hwpx_textpos_idempotent.rs): 이전 실행 3건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`hwp3-table-caption-2020.pdf`](../../../pdf/hwp3-table-caption-2020.pdf) — SHA-256 `7f996403564b65d70c89f4e2b464b297f736b5234a709aa9f8f09e11bfba2bcf`.
- [`hwp3-table-caption-hwpx-2020.pdf`](../../../pdf/hwp3-table-caption-hwpx-2020.pdf) — SHA-256 `227fa3c5da909c6055f2e34072a8e1ec6750d2560a52955a55df949a4f9dd280`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

제외 입력 정리 후 최소 글꼴 환경에서 #8303의 남은 3개 검사가 모두 통과했다. #8304의 6개 검사와 한 번에 실행한 [집중 검사 로그](../../../output/pr-review/planet6897-ordered-20261031/excluded-fixture-focused-minfont.log)에서 두 모듈 합계 9건 통과를 확인했다. 전체 검사의 대체 결과는 아니다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.
