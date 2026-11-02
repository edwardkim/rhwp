# [PR #8396](https://github.com/edwardkim/rhwp/pull/8396) 리뷰 — fix(typeset): HWP5 출신 HWPX 분할 측정기에 저장 조판 프로필 적용 ([이슈 #8382](https://github.com/edwardkim/rhwp/issues/8382) 부분)

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `2008497a17` → `234761ecad`, `d6eb7f9345` → `57cfea700d`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8382_hwp5_origin_hwpx_split_budget` 1건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [PR #8396](https://github.com/edwardkim/rhwp/pull/8396), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `d6eb7f93457b6e63a1060f7460b3663e2c989c08`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `d6eb7f93457b6e63a1060f7460b3663e2c989c08`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8382](https://github.com/edwardkim/rhwp/issues/8382).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP5 출처 HWPX 표 분할 측정이 실제 배치와 같은 저장 조판 프로필을 소비하게 한다.
첫 조각 예산과 행 소유를 확인한다. [이슈 #8382](https://github.com/edwardkim/rhwp/issues/8382) 전체 해결로 확대하지 않는다.

## 체리픽과 보정

- 원본 `2008497a17f901fb4f1b9df381eb948fc15a3e27` → 통합 `dd801e4e51e9e1c705a725a1e819f56faa41cc4c`.
- 원본 `d6eb7f93457b6e63a1060f7460b3663e2c989c08` → 통합 `46c4a7068ab005fe4e7ede7979d418d8f57293f3`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8382_hwp5_origin_hwpx_split_budget.rs`](../../../tests/cases/issue_8382_hwp5_origin_hwpx_split_budget.rs): 이전 실행 1건 통과, 0건 실패, 0건 건너뜀.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## 2026-11-02 일괄 대조 추가 증거

원 PR의 네 원본 HWP를 검토 엔진으로 HWPX로 저장하고 같은 원본의 한컴 Print와
96dpi로 전쪽 대조했다. 네 저장본은 `export-hwpx --verify-pages`에서 원본과
쪽수가 같았다. 2020 Print는 호환 세대 2022, 2024 Print는 2024로 측정했다.
이전 기여자 측정과 글꼴 환경이 같다고 단정하지 않는다.

| 입력 / 독립 기준 | 쪽 | Native 최저 | WASM 최저 | TSV |
| --- | ---: | ---: | ---: | --- |
| [headerrow HWPX](../assets/issue_8382/followup8438/headerrow.hwpx) / [2020 PDF](../../../pdf/issue8149/headerrow-2020.pdf) | 31 | 100.00% | 100.00% | [Native](../assets/issue_8382/followup8438/headerrow-native.tsv) · [WASM](../assets/issue_8382/followup8438/headerrow-wasm.tsv) |
| [reservefit HWPX](../assets/issue_8382/followup8438/reservefit.hwpx) / [2020 PDF](../../../pdf/issue8149/reservefit-2020.pdf) | 65 | 100.00% | 100.00% | [Native](../assets/issue_8382/followup8438/reservefit-native.tsv) · [WASM](../assets/issue_8382/followup8438/reservefit-wasm.tsv) |
| [rowfit HWPX](../assets/issue_8382/followup8438/rowfit.hwpx) / [2020 PDF](../../../pdf/issue8149/rowfit-2020.pdf) | 33 | 100.00% | 100.00% | [Native](../assets/issue_8382/followup8438/rowfit-native.tsv) · [WASM](../assets/issue_8382/followup8438/rowfit-wasm.tsv) |
| [71352 HWPX](../assets/issue_8382/followup8438/71352.hwpx) / [2024 PDF](../../../pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf) | 48 | 96.84% | 96.86% | [Native](../assets/issue_8382/followup8438/71352-native.tsv) · [WASM](../assets/issue_8382/followup8438/71352-wasm.tsv) |

원 PR이 지목한 24쪽은 H12 행 4~10이 표에서 빠짐없이 이어지고, 71352의
37쪽 마지막 표 조각도 한컴 PDF의 쪽과 같은 내용을 보인다.
[24쪽 Native review](../assets/issue_8382/followup8438/headerrow-native-review-024.png) · [24쪽 overlay](../assets/issue_8382/followup8438/headerrow-native-overlay-024.png) ·
[37쪽 Native review](../assets/issue_8382/followup8438/71352-native-review-037.png) · [37쪽 overlay](../assets/issue_8382/followup8438/71352-native-overlay-037.png).

### 새 base 재저장 동일성

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 Native 실행 파일로 네 HWP를
다시 HWPX로 저장했다. `--verify-pages`는 네 문서 모두 통과했고, 각각의 저장본은
위 TSV에 사용한 HWPX와 SHA-256이 같다. 위 한컴 Print와 시각 결과를 재사용한다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8382_hwp5_origin_hwpx_split_budget` 1건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.
