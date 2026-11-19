# PR #9180 리뷰 — fix(layout): 표 셀 안 도형·묶음의 글상자 글 경로를 그 셀부터 잡는다

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9180](https://github.com/edwardkim/rhwp/pull/9180) · `semanticist21` |
| 원 head | `b4010c23b085982b6bba66bd6ebb25e6d82a2012` |
| 적용 커밋 | `8133ec54f73f` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

표 셀 안에 묶인 글상자의 전체 셀 경로로 위치를 조회하고 해당 글상자의 캐럿이 선택되는지 확인했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

원 PR의 글상자 경로 회귀 1건 통과. 변경은 글상자 조회 주소와 캐럿에 적용된다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9180/regression.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 표 셀 안 묶음 글상자의 주소 문제를 보정해 주셔서 감사합니다. 두 묶음 글상자의 hit와 caret 주소는 선택한 셀을 포함한 전체 경로를 가리켰습니다. 입력과 함께 저장한 회귀 증거를 개별 리뷰에 연결했습니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9180/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.
