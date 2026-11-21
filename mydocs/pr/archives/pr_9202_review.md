# PR #9202 검토 — 나뉜 셀의 중첩 표와 후속 행

## 최종 판정

**보호 미달 두 쪽 보정 완료 — 대상 문서 Native·fresh WASM 160쪽이 페이지별 보호값을 충족했다.**

[메인터너 보정 원인](../assets/integration_open_prs_20261120/maintainer-cause.md) · [전쪽 결과](../assets/integration_open_prs_20261120/maintainer-repair/verification.json). 최종 통합 승인에는 새 코드의 전체 검증이 필요하다.
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

초기 비교에서 고정 devel의 Native·fresh WASM은 각각 11쪽이었다. 후보와 기존 첨부 Print는
각각 10쪽이었다. 기존 PDF는 파일명과 달리 `Creator: Hwp 2022 12.0.0.4547`이었다.
기준 11쪽의 공백 제외 본문 98자는 후보 10쪽 끝에 모두 남았고 마지막 세 행 순서가 독립
Print와 일치했다. 9쪽의 조직도 앞 빈 줄과 10쪽의 `Pension Policies`가 시작하는 행을 직접
대조했다. 후보 전쪽 최저는 Native 99.72049%, WASM 91.56133%다. WASM 최저 2쪽 수치는
같은 조건의 기준과 같다. 원본의 모아찍기 값은 4이며 시각 비교는 독립 1-up Print 조건이다.
[원본 HWP](../../../samples/issue9040/1371000-policy-research-application.hwp) ·
[기존 대조 자료](../../../pdf/issue9040/1371000-policy-research-application-2020.pdf) ·
[전체 TSV 위치와 파일 해시](../assets/pr-9202/evidence.json).

| 조판 경계 | Native | fresh WASM |
| --- | --- | --- |
| 9쪽 조직도 시작 | [review](../assets/pr-9202/issue9040-native-009-review.png) · [overlay](../assets/pr-9202/issue9040-native-009-overlay.png) | [review](../assets/pr-9202/issue9040-wasm-009-review.png) · [overlay](../assets/pr-9202/issue9040-wasm-009-overlay.png) |
| 10쪽 후속 행 | [review](../assets/pr-9202/issue9040-native-010-review.png) · [overlay](../assets/pr-9202/issue9040-native-010-overlay.png) | [review](../assets/pr-9202/issue9040-wasm-010-review.png) · [overlay](../assets/pr-9202/issue9040-wasm-010-overlay.png) |

정본 corpus의 별도 중첩 표 #2007에서 발견한 글꼴 적재 순서를 바로잡았다. #9040의 표
분할 코드와 별개의 검증 환경 보정이며 #2007 Native 17쪽 모두 정본을 복원했다.
[보정 원인과 확대 비교](../assets/integration_open_prs_20261120/README.md#중첩-표-native의-글꼴-공급-보정).

## 한컴 2024 인쇄본으로 최종 재검증

동일 HWP를 MCP 한컴 2024 `13.0.0.3901`의 Hancom PDF 인쇄 1-up으로 출력했다.
독립 PDF, Native, 최적화한 fresh WASM 모두 10쪽이다. 두 후보의 raw 최저는 4쪽에서
각각 **99.65872%**, **99.66454%**였다. 9쪽 조직도는 앞 빈 줄 뒤에 위치하고 같은 쪽의
후속 실적 표를 보존한다. 10쪽은 `Pension Policies`로 시작하며 이 행은 전체 출력에 한 번만
나온다. 마지막 세 행과 표 선·글자 가장자리의 미세한 차이를 직접 대조했다. 4쪽에서는 줄 순서와 제목 배치를
확인했고, 로마 숫자와 하이픈의 획 모양에 차이가 보였다.
[한컴 2024 Print](../../../pdf/issue9040/1371000-policy-research-application-2024.pdf) ·
[Native 전쪽 TSV](../assets/pr-9202/hancom-2024/issue9040-native.tsv) ·
[fresh WASM 전쪽 TSV](../assets/pr-9202/hancom-2024/issue9040-wasm.tsv) ·
[생산·대조 기록](../assets/pr-9202/evidence.json).

| 페이지 | Native | fresh WASM |
| --- | --- | --- |
| 4쪽 최저 | [review](../assets/pr-9202/hancom-2024/issue9040-native-p004-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-native-p004-overlay.png) | [review](../assets/pr-9202/hancom-2024/issue9040-wasm-p004-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-wasm-p004-overlay.png) |
| 9쪽 조직도와 후속 실적 표 | [review](../assets/pr-9202/hancom-2024/issue9040-native-p009-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-native-p009-overlay.png) | [review](../assets/pr-9202/hancom-2024/issue9040-wasm-p009-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-wasm-p009-overlay.png) |
| 10쪽 이어지는 표 | [review](../assets/pr-9202/hancom-2024/issue9040-native-p010-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-native-p010-overlay.png) | [review](../assets/pr-9202/hancom-2024/issue9040-wasm-p010-review.png) · [overlay](../assets/pr-9202/hancom-2024/issue9040-wasm-p010-overlay.png) |

신청서 9쪽은 실제 Studio의 Canvas2D·CanvasKit compat/default에서 화면·인쇄 프로필로 확인했다. 검증 세션에 한해 한양신명조 프로그램을 공급한 후 대상 쪽의 글꼴 폴백은 0건이었다.
[실제 Studio 캡처와 검증](../assets/integration_open_prs_20261120/actual-studio-renderers/README.md).

## Merge 후 contributor PR comment 계획

저장 줄을 이용해 표와 뒤 행을 같은 흐름으로 복원한 기여에 감사한다. 한컴과 같은 10쪽
결과와 기준 11쪽의 마지막 세 행이 보존된 근거를 설명한다. 통합 CI·병합 SHA·devel 반영을
확인한 뒤 한컴 2024 대조의 2·4·8·9·10쪽 Native·fresh WASM review와 overlay를 병합 SHA의 raw 이미지로
게시하고 응답 해시와 게시 본문, #9040 상태를 확인한다.
[Visual Sweep 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
