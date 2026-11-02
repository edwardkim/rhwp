# [PR #8378](https://github.com/edwardkim/rhwp/pull/8378) 리뷰 — fix(typeset): HWP5 출처 HWPX의 1행 표도 원본 HWP처럼 쪽 경계에서 나눈다 ([이슈 #8373](https://github.com/edwardkim/rhwp/issues/8373))

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `fcd58cb347` → `242e2c7e83`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `hwp5_origin_hwpx_single_row_table_split` 1건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8378](https://github.com/edwardkim/rhwp/pull/8378), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `fcd58cb347583f3b4ba7e3ce24b20418e299c539`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `fcd58cb347583f3b4ba7e3ce24b20418e299c539`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8373](https://github.com/edwardkim/rhwp/issues/8373).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP5 출처 HWPX의 1행 표가 HWP 원본처럼 쪽 경계에서 분할되도록 한다.
분할 조각·뒤 본문·재저장 원본의 내용 누락과 중복을 확인한다.

## 체리픽과 보정

- 원본 `fcd58cb347583f3b4ba7e3ce24b20418e299c539` → 통합 `75fad7d97257995ba0307a4682c6ccdb62f2c7e7`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`hwp5_origin_hwpx_single_row_table_split.rs`](../../../tests/cases/hwp5_origin_hwpx_single_row_table_split.rs): 이전 실행 1건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`hwpx_sample2-rhwp-hwpx-2024.pdf`](../../../pdf/hwpx_sample2-rhwp-hwpx-2024.pdf) — SHA-256 `711219ae8b3090bedb7721c7b885546dba07a114f870b12b2894af20b79aa796`.
- [`hwpx_sample2.hwp`](../../../samples/hwpx_sample2.hwp) — SHA-256 `ca66d521d070ac8442e09fd3f283e17611eb9efb45eb4bf6c3a8f65e604f6b5f`.
- [`hwpx_sample2-2024.pdf`](../../../pdf/hwpx_sample2-2024.pdf) — SHA-256 `406d3d9da7967f50dd04e115ca5f53953e61c88d4641005165f251a65d8028af`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## 2026-11-02 일괄 대조 추가 증거

[sample2 원본의 1세대 HWPX 저장본](../assets/issue_8373/followup8438/sample2-saved.hwpx)과
[그 파일의 한컴 2024 Print](../../../pdf/issue8373/batch-8378-sample2-2024.pdf)를
96dpi에서 29쪽 전체 비교했다. 파일 해시와 MCP 작업은 [기준 기록](../assets/issue_8373/followup8438/print-sources.md)에 있다.
Native 최저 95.86%, fresh WASM 최저 95.78%로 90% 미만은 없었다.
[Native 전체 TSV](../assets/issue_8373/followup8438/native-all.tsv) ·
[WASM 전체 TSV](../assets/issue_8373/followup8438/wasm-all.tsv).
19쪽은 셀의 「받아서 제출」로 끝나고 20쪽은 「⑧ 선착순」으로 이어지는 것을
양쪽 출력에서 직접 확인했다. 두 경로의 19쪽 raw는 99.94%, 20쪽은 100.00%다.
[19쪽 Native review](../assets/issue_8373/followup8438/native-review-019.png) · [19쪽 Native overlay](../assets/issue_8373/followup8438/native-overlay-019.png) ·
[20쪽 Native review](../assets/issue_8373/followup8438/native-review-020.png) · [20쪽 Native overlay](../assets/issue_8373/followup8438/native-overlay-020.png) ·
[19쪽 WASM review](../assets/issue_8373/followup8438/wasm-review-019.png) ·
[19쪽 WASM overlay](../assets/issue_8373/followup8438/wasm-overlay-019.png) ·
[20쪽 WASM review](../assets/issue_8373/followup8438/wasm-review-020.png) ·
[20쪽 WASM overlay](../assets/issue_8373/followup8438/wasm-overlay-020.png).

### 새 base 재저장 동일성

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 Native 실행 파일로 다시 저장한
1세대 HWPX의 SHA-256은 `c5149ebea0e34d78103e8b81def04e4a3cdc72aa9f9bb97dab222cad84dba9ca`로 기존 비교 입력과 바이트 단위로 같다.
같은 입력의 이전 Print와 시각 결과를 재사용할 수 있다는 뜻이며, 별도 미달이나
기능 판정을 면제하지 않는다.

이후 제품 코드 `dd284b5cda`의 Native 실행 파일로도 원본을 다시 저장했다. `--verify --verify-pages`는 IR 차이 없음과 전후 29쪽을 확인했고 저장 바이트의 SHA-256은 위 `c5149ebea0e34d78103e8b81def04e4a3cdc72aa9f9bb97dab222cad84dba9ca`와 같았다. 이는 입력 보존 근거이며 현재 진행 중인 WMFC 변경을 포함한 출력의 신규 시각 검증은 아니다.

같은 제품 코드의 Native로 저장 HWPX 전 29쪽을 위 한컴 2024 Print와 다시 비교했다. 최저 95.86%, 90% 미만 0쪽이며 산출 TSV SHA-256 `4b252487b9f9a6a2591edfc1ec87c1159d7e5ddb47aa1d501faf500e00a87ca0`은 위 기존 Native TSV와 바이트 단위로 같아 재사용한다. 같은 제품 코드의 fresh WASM 및 이후 시험 중인 WMFC 변경은 별도로 확인한다.

동일 제품 코드의 fresh WASM도 같은 저장본 29쪽을 한컴 Print와 비교했다. 최저 95.78%, 90% 미만 0쪽이며 산출 TSV SHA-256 `30c55c60bb8864ef9f80a869e36ea7a0630985996b7103816377768fd082f94a`가 위 기존 WASM TSV와 같아 재사용한다. 이는 WMFC 인쇄 장치 변경 전 패키지의 결과다.

원 [PR #8378](https://github.com/edwardkim/rhwp/pull/8378)이 통합
[PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 제출한
[기존 한컴 Print](../../../pdf/hwpx_sample2-rhwp-hwpx-2024.pdf)의 입력은
원 PR 본문에서 SHA-256 접두어 `23d94c2e`로 기록했다. 위 재저장본
`c5149eb...`과 바이트가 다르므로 두 Print의 입력을 같은 바이트로 단정하지 않는다.
기존 Print와 원 PR의 Native·WASM TSV 및 쪽 19·20의 이미지는 원 PR이 주장한
동작을 확인하는 기존 증거로 재사용한다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `hwp5_origin_hwpx_single_row_table_split` 1건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.
