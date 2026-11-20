# PR #9202 검토 — 나뉜 셀의 중첩 표와 후속 행

## 최종 판정

**머지 보류 — 통합 후보의 정본 WASM 보호 두 쪽이 하한에 도달해야 한다.**
원 PR의 분할 셀 흐름 개선과 뒤 행 보존은 고정 기준 및 독립 Print와 확인했다.
[통합 검증](../assets/integration_open_prs_20261120/README.md).

## 접수와 적용

| 항목 | 확인 |
| --- | --- |
| 원 PR·작성자·리뷰어 | [#9202](https://github.com/edwardkim/rhwp/pull/9202) · `planet6897` · `jangster77` |
| 원 head·관련 이슈 | `ca1102c3977195fd7c919f63095facfe813e7163`, 열린 PR, MERGEABLE, 22파일 · [#9040](https://github.com/edwardkim/rhwp/issues/9040) |
| 적용 순서 | 두 번째. `6e56b48f66a3f2a9ca5071ebbc5d7f37c3e56322`부터 `f45fa4cc1295f4c6156f798e44ab00714abece41`까지 4개 |
| 고정 기준·제품 후보 | `0a437a7eefb19bfe81fc12866887e4548ea9554d` · `25f9587840d09b08af2b37be178ab93379a7b9dc` |

## 조판 검토와 실제 출력

저장 줄 사이의 거리로 앞 빈 줄과 중첩 표 외곽을 확인할 수 있을 때, 같은 간격을 표 조각의
높이와 배치에 중복 소비하지 않는다. 원본의 p41 끝 `3000 + 141 + 26502 + 141 = 29784`는
p42의 호스트 줄에 닿고, 그 뒤 호스트 줄 높이 `1500`을 적용하면 p43의 `31284`에 맞는다.
해당 경로의 측정과 paint, 시작·끝 컷과 후속 행의 쪽 소속을 확인했다.
[원본·Print와 저장 줄 설명](../../../samples/issue9040/README.md) ·
[분할 경계 회귀](../../../tests/cases/issue_9040_empty_spacer_nested_table_band.rs).

기준은 Native·fresh WASM 각각 11쪽이었다. 후보와 한컴 2020 Print 1-up은 각각 10쪽이다.
기준 11쪽의 공백 제외 본문 98자는 후보 10쪽 끝에 모두 남았고 마지막 세 행 순서가 독립
Print와 일치했다. 9쪽의 조직도 앞 빈 줄과 10쪽의 `Pension Policies`가 시작하는 행을 직접
대조했다. 후보 전쪽 최저는 Native 99.72049%, WASM 91.56133%다. WASM 최저 2쪽 수치는
같은 조건의 기준과 같다. 원본의 모아찍기 값은 4이며 시각 비교는 독립 1-up Print 조건이다.
[원본 HWP](../../../samples/issue9040/1371000-policy-research-application.hwp) ·
[독립 Print](../../../pdf/issue9040/1371000-policy-research-application-2020.pdf) ·
[전체 TSV 위치와 파일 해시](../assets/pr-9202/evidence.json).

| 조판 경계 | Native | fresh WASM |
| --- | --- | --- |
| 9쪽 조직도 시작 | [review](../assets/pr-9202/issue9040-native-009-review.png) · [overlay](../assets/pr-9202/issue9040-native-009-overlay.png) | [review](../assets/pr-9202/issue9040-wasm-009-review.png) · [overlay](../assets/pr-9202/issue9040-wasm-009-overlay.png) |
| 10쪽 후속 행 | [review](../assets/pr-9202/issue9040-native-010-review.png) · [overlay](../assets/pr-9202/issue9040-native-010-overlay.png) | [review](../assets/pr-9202/issue9040-wasm-010-review.png) · [overlay](../assets/pr-9202/issue9040-wasm-010-overlay.png) |

정본 corpus의 별도 중첩 표 #2007에서 발견한 글꼴 적재 순서를 바로잡았다. #9040의 표
분할 코드와 별개의 검증 환경 보정이며 #2007 Native 17쪽 모두 정본을 복원했다.
[보정 원인과 확대 비교](../assets/integration_open_prs_20261120/README.md#중첩-표-native의-글꼴-공급-보정).

## Merge 후 contributor PR comment 계획

저장 줄을 이용해 표와 뒤 행을 같은 흐름으로 복원한 기여에 감사한다. 한컴과 같은 10쪽
결과와 기준 11쪽의 마지막 세 행이 보존된 근거를 설명한다. 통합 CI·병합 SHA·devel 반영을
확인한 뒤 2·8·9·10쪽의 실제 Native·fresh WASM review와 overlay를 같은 SHA의 raw 이미지로
게시하고 응답 해시와 게시 본문, #9040 상태를 확인한다.
[Visual Sweep 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
