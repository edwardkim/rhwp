# PR #8517 검토 — PR #8438의 미종결 원 PR 보정

## 최종 판정

**메인터너 보정 후 수용 가능 — PR #8438에 반영된 열린 원 PR 11건의 로컬 검증 완료.**
각 원 PR의 주장 입력과 독립 한컴 Print PDF를 Native·fresh WASM으로 전쪽 대조했고,
변경 영역의 review·overlay를 직접 판독했다. code head
`ba0557118ad896f1c410e80da5100fedf10139d8`의 모든 필수 CI가 통과했으며,
[PR #8517](https://github.com/edwardkim/rhwp/pull/8517)은
`c97709b59742372a817b3886f442144728fca646`로 `devel`에 병합됐다.
사용자 지정 옵션 2에 따라 이 기록과 오늘할일은 별도 문서 PR에 반영한다.
원 PR 댓글과 종료는 그 문서 PR의 병합 뒤 진행한다.

## 접수와 적용 범위

| 항목 | 확인 결과 |
| --- | --- |
| 대상 | [PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 체리픽된 원 PR 중 검토 당시 열린 11건 |
| code head | `ba0557118ad896f1c410e80da5100fedf10139d8` |
| 시각·전체 회귀 측정 head | `db9e84189448c72337a1568fd275a7820ae88bf2` |
| 시각 비교 base | `4e2cc6bb36576b95d58d4c76cd833281ea652554` |
| 최종 병합 | `c97709b59742372a817b3886f442144728fca646` |

코드 후보는 원 기여자의 변경을 보존하면서, 독립 Print와 어긋난 기호 슬롯·문단·표·차트
출력 및 저장 경계를 공통 처리에서 보정했다. 마지막 제품 조판 변경은 `5aa6ee08dd`이고,
실물 회귀 보강은 `c8e4f0ffac`이다. 이후 `57ce496ef7`은 민감 문서의 조판 수치
진단 로그 네 곳을 제거했고, `0d56d7f69e`는 종료된 원 PR의 중복 문서·증적을 제외했다.
`fd5795bce6`은 의도된 테스트 모듈 재사용을 Clippy에 명시했다.
`ba0557118a`는 Rust CodeQL의 45분 timeout을 90분으로 늘렸으며 분석 범위와 보안
쿼리는 유지했다. 이 후속 커밋들은 제품 조판 출력을 변경하지 않는다.

## 개별 PR 검토와 증거

아래 각 문서는 원 PR의 실제 입력·기준 PDF, 전체 페이지 Native/fresh WASM TSV,
주장 페이지 review·overlay 및 직접 판독 결과를 분리해 기록한다.
[#8386](pr_8386_review.md)는 저장 bit 계약이므로 시각 변경 대상이 아니다.

| 원 PR | 검토 영역 | 개별 판정 |
| --- | --- | --- |
| [#8316](https://github.com/edwardkim/rhwp/pull/8316) | 기호·영문 슬롯 분류 | [검토](pr_8316_review.md) |
| [#8321](https://github.com/edwardkim/rhwp/pull/8321) | 기호 폭·리터럴 소프트 하이픈 | [검토](pr_8321_review.md) |
| [#8336](https://github.com/edwardkim/rhwp/pull/8336) | 셀 세로쓰기·저장 왕복 | [검토](pr_8336_review.md) |
| [#8341](https://github.com/edwardkim/rhwp/pull/8341) | 제목 차례 run·WMFC 차트 | [검토](pr_8341_review.md) |
| [#8364](https://github.com/edwardkim/rhwp/pull/8364) | HWP3 표 캡션 저장 | [검토](pr_8364_review.md) |
| [#8365](https://github.com/edwardkim/rhwp/pull/8365) | 편집 입력 고정 줄간격 | [검토](pr_8365_review.md) |
| [#8375](https://github.com/edwardkim/rhwp/pull/8375) | 편집 입력 저장 줄 배치 | [검토](pr_8375_review.md) |
| [#8378](https://github.com/edwardkim/rhwp/pull/8378) | 1행 표 쪽 경계 | [검토](pr_8378_review.md) |
| [#8380](https://github.com/edwardkim/rhwp/pull/8380) | 그림과 본문 흐름 | [검토](pr_8380_review.md) |
| [#8386](https://github.com/edwardkim/rhwp/pull/8386) | HWPX 개체 attr bit 28 | [검토](pr_8386_review.md) |
| [#8396](https://github.com/edwardkim/rhwp/pull/8396) | 저장 조판 프로필·표 분할 | [검토](pr_8396_review.md) |

[원본·PDF·SHA와 전체 TSV 목록](../assets/review_8438_followup/final-original-inputs/README.md)을
기준으로 실제 입력 23개, [#8321](https://github.com/edwardkim/rhwp/pull/8321) 대조군
2개, [#8341](https://github.com/edwardkim/rhwp/pull/8341) 차트 1개를 측정했다.
Native 1,123쪽·fresh WASM 1,123쪽, 총 2,246행에서 90% 미만·누락·고정 devel 대비
2%p 초과 하락은 모두 0이다. [대조 분석](../../working/review_8438_wmfc_chart_diagnostic.md)에
측정 조건과 결과를 기록했다. 별도
[#8389·#8397 Native TSV](../assets/review_8438_followup/pr8389_8397/native-final-comparison.tsv)와
[fresh WASM TSV](../assets/review_8438_followup/pr8389_8397/wasm-final-comparison.tsv)의
2,153행도 소수 둘째 자리 표시값 하락이 0이다.

#8341 차트 재저장본은 독립 한컴 2024 Print와 전 64쪽을 비교했다. Native 최저는
42쪽 94.90%, fresh WASM 최저는 같은 쪽 94.92%이며, 차트 35쪽은 각각 95.52%와
95.56%다. [35쪽 Native review](../assets/issue_8305/followup8438/native-wmfc-rebased-review-035.png)·
[overlay](../assets/issue_8305/followup8438/native-wmfc-rebased-overlay-035.png),
[WASM review](../assets/issue_8305/followup8438/wasm-wmfc-rebased-review-035.png)·
[overlay](../assets/issue_8305/followup8438/wasm-wmfc-rebased-overlay-035.png)에서
차트 격자·막대·범례와 앞뒤 본문을 직접 대조했다. 다른 주장 페이지의 확대 판독은
위 개별 검토 문서에 남겼다.

## 로컬·원격 검증

- 최소 글꼴 환경 전체 Rust nextest: 12,387건 통과, 45건 건너뜀. 전체 빌드,
  Native Skia 집중 검사 3종, Native/WASM Clippy, fmt, fresh WASM 패키지와
  Markdown 링크 검사를 완료했다.
- code head의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41592450512),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41592450498),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41592450511),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41592450513),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41592450641),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41595836744)가
  성공했다. 첫 CodeQL Rust run은 45분 timeout으로 중단됐고, 제한 변경 뒤
  전체 Rust 분석이 26분 18초에 정상 통과했다.
- 병합 후 [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/41596147061)가
  성공했다. 이 후속 작업은 병합 전 CI의 대체 증거로 세지 않는다.
- 새 devel `8e9ce0247757ae603027892d45115c1d7753b975`와의 병합 시뮬레이션은
  충돌 없이 완료됐다. 코드 PR의 최종 GitHub 병합 결과도 확인했다.

## Merge 후 contributor PR 처리

문서 전용 후속 PR이 병합된 뒤 개별 검토 문서의 댓글 계획과 실제 asset을 다시
대조한다. 원 PR 11건 각각에 원 기여와 메인터너 보정을 구분한 한국어 댓글을
게시하고, [Visual Sweep comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에
따라 같은 입력의 대표 review·overlay를 표시한다. 그 뒤 이미 통합된 원 PR을
중복 병합 없이 종료한다. [이슈 #8382](https://github.com/edwardkim/rhwp/issues/8382)는
[#8396](https://github.com/edwardkim/rhwp/pull/8396)의 부분 해결 범위를 넘으므로
원 PR 종료와 별도로 상태를 판정한다.
