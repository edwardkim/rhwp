# [PR #8341](https://github.com/edwardkim/rhwp/pull/8341) 리뷰 — fix(serializer/hwpx): 제목 차례 표시를 자기 글자 모양 run에 두고 경계를 밀지 않는다 ([이슈 #8305](https://github.com/edwardkim/rhwp/issues/8305))

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `74685790f5` → `bc4fa7f74a`, `2aecaab467` → `3fb51885a3`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8305_titlemark_boundary_runs` 5건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [PR #8341](https://github.com/edwardkim/rhwp/pull/8341), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `2aecaab46753b16ce51b426895b9b97dfeb5d5c7`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `2aecaab46753b16ce51b426895b9b97dfeb5d5c7`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8305](https://github.com/edwardkim/rhwp/issues/8305).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

제목 차례 표시를 원래 글자 모양 run에 두어 후속 경계가 밀리지 않도록 한다.
UTF-16 축의 표시 소유와 뒤 글자 모양을 저장 왕복에서 확인한다.

## 체리픽과 보정

- 원본 `74685790f53ab58f17a1a0cc0b00fe55cb1fc4a6` → 통합 `f331c82fe514b9c38c98882adedd4004c595c2e5`.
- 원본 `2aecaab46753b16ce51b426895b9b97dfeb5d5c7` → 통합 `c21bc6f69ffb8903b2fb9c2c50f1dab347d419e9`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8305_titlemark_boundary_runs.rs`](../../../tests/cases/issue_8305_titlemark_boundary_runs.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`1790387_policy_progress-hwpx-2020.pdf`](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf) — SHA-256 `4c0d9d0876410766d99f4589916971f97b751a3d71901068637c7e2cc1061f95`.

기여자가 제출한 기준 자료: [`pp_hwpx_base_silhouette.tsv`](../../../mydocs/pr/assets/issue_8305/native/pp_hwpx_base_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

원 [PR #8341](https://github.com/edwardkim/rhwp/pull/8341)의 기준 PDF와
수정 후 전체 TSV는 통합 [PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에
이미 포함됐다. 기존 [Native TSV](../assets/issue_8305/native/pp_hwpx_silhouette.tsv)와
[WASM TSV](../assets/issue_8305/wasm/pp_hwpx_silhouette.tsv)는 작성자가 제출한
당시 입력·head의 결과로 사용한다. 별도 재출력 결과를 이 기록과 혼동하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

### 새 base 재저장 동일성

[PR #8309](https://github.com/edwardkim/rhwp/pull/8309)가 반영된 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 Native 실행 파일로 다시 저장한
사양 대상 재저장 HWPX의 SHA-256은 `14ef498440ef15c3fe0a667a072c2141b1761eb5f0e0e970f6da44495be466c9`로 기존 비교 입력과 바이트 단위로 같다.
같은 입력의 이전 Print와 시각 결과를 재사용할 수 있다는 뜻이며, 별도 미달이나
기능 판정을 면제하지 않는다.

### 한컴 버전별 차트 출력과 전쪽 검증

동일 원본과 재저장본의 한컴 2020 Print는 둘 다 29쪽 WMF 차트를 확대·잘라
출력한다. 같은 재저장본의 한컴 2024 Print는 차트를 전체 표시한다. 두 기준의
출력 동작이 다르므로 한 버전을 다른 버전의 인쇄 결과로 취급하지 않는다.
[출력 근거와 파일 해시](../assets/issue_8305/followup8438/print-source.md).

2024 Print와 64쪽을 비교했을 때 35쪽 raw는 Native 89.11%, WASM 89.18%로
양쪽 모두 기준 미달이다. 35쪽 차트는 WMF이며 그 숫자의 원본 LOGFONT는
`Calibri`, 굵기 700이다. 검토 시스템에는 Calibri 일반체만 있어 Windows의
Calibri Bold를 검증용 `output/`에 공급했다. 다만 재측정은 Native 89.12%,
WASM 89.19%로 90%를 넘지 못했다. 글꼴 공급만으로 해결됐다고 판정하지 않고
실제 WMF 공급·그리기와 나머지 차이를 확인한 뒤 재검증한다. 글꼴 바이트는
저장소·배포물에 넣지 않는다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

[PR #8309](https://github.com/edwardkim/rhwp/pull/8309)가 포함된 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8305_titlemark_boundary_runs` 5건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 기존 한컴 2020 기준의 저장본 WASM 대조

원 PR이 사용한 [한컴 2020 Print](../../../pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf)와
재저장 HWPX를 96 DPI로 비교한 [64쪽 WASM TSV](../assets/issue_8305/followup8438/wasm-saved-original2020-all.tsv)는
최저 92.41%(42쪽), 90% 미만 0쪽이다. 중단됐던 계산은 기존 PNG와 입력 해시를 확인해
쪽별 체크포인트부터 이어서 완료했다. 이 결과는 이전 통합 후보의 WASM 패키지로 산출한 값이며
새 base에서 빌드한 WASM의 실행 결과가 아니다. 새 base는 이후 HWP 저장 경로가 바뀌었고,
그 외 렌더러는 이전 후보와 같음을 별도로 확인했다. 2024 Print 비교 결과와 혼동하지 않는다.

같은 저장본과 기준 PDF의 [Native 전체 TSV](../assets/issue_8305/followup8438/native-saved-original2020-all.tsv)는
64쪽 최저 92.61%(30쪽), 90% 미만 0쪽이다. 기존 Native 실행 파일 SHA-256은
`34dfddd062947e91c33cb3c2b2c16a10f9903d2d1184fcdb646683e6a0dc2f1f`이다.
중단 전에 완성된 PNG와 같은 실행 파일로 나머지 쪽을 다시 내보낸 PNG를 원본
해시와 함께 대조했다. 이 결과는 표 원점 후보 수정 전의 측정이다.

원 PR이 지목한 제목 차례 영역을 같은 입력의 기존 PNG로 다시 구성했다.
[20쪽 Native review](../assets/issue_8305/followup8438/native-original2020-review-020.png) ·
[20쪽 Native overlay](../assets/issue_8305/followup8438/native-original2020-overlay-020.png) ·
[59쪽 Native review](../assets/issue_8305/followup8438/native-original2020-review-059.png) ·
[59쪽 Native overlay](../assets/issue_8305/followup8438/native-original2020-overlay-059.png) ·
[20쪽 WASM review](../assets/issue_8305/followup8438/wasm-original2020-review-020.png) ·
[20쪽 WASM overlay](../assets/issue_8305/followup8438/wasm-original2020-overlay-020.png) ·
[59쪽 WASM review](../assets/issue_8305/followup8438/wasm-original2020-review-059.png) ·
[59쪽 WASM overlay](../assets/issue_8305/followup8438/wasm-original2020-overlay-059.png).
제목과 주변 본문의 순서·표시는 유지되며, 세부 글자 모양과 점선 차이도 함께 보인다.
