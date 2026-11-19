# PR #9186 리뷰 — fix(cursor): 본문 칸 캐럿·선택·칸 상자 조회가 같은 주소의 머리말 표를 잡지 않는다

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9186](https://github.com/edwardkim/rhwp/pull/9186) · `semanticist21` |
| 원 head | `289f63add5f40a2d95c5e7de0d0dfec0b656ae41` |
| 적용 커밋 | `595643294323` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

같은 주소를 갖는 머리말 표와 본문 표에서 본문 캐럿·선택·칸 상자가 본문으로 유지되는지, 머리말 캐럿이 머리말에 남는지 확인했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

캐럿 및 선택 위치 조회 회귀 2건 통과. 변경은 조회 경로에 적용된다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9186/regression.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 본문 표와 머리말 표의 같은 주소가 충돌하는 사례를 보정해 주셔서 감사합니다. 본문 표의 caret·선택·칸 상자와 머리말 표의 자체 경로를 확인했고, 통합 렌더링 검증에서 관련 보호 문서의 페이지 하락은 0건입니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9186/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.
