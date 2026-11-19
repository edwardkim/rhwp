# PR #9179 리뷰 — feat(bookmark): 블록 책갈피를 넣고 고치며, 글 편집에도 범위를 지킨다

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9179](https://github.com/edwardkim/rhwp/pull/9179) · `semanticist21` |
| 원 head | `c33d5dbb185226507b80d353d1b0fdbc7960f55f` |
| 적용 커밋 | `9ffd6d78bc74` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

블록 책갈피의 앞뒤 입력, 문단 나누기·합치기·삭제와 책갈피 마커 뒤 누름틀 보존을 확인했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

블록 범위·마커 보존 Rust 회귀 20건 통과. fresh WASM의 교차 저장을 포함한 HWP/HWPX 네 저장 순서에서 책갈피와 본문이 두 번 재열기 뒤에도 보존됐다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9179/regression.json)

[각 HWP/HWPX 반복 저장의 확인값](../assets/open9_integration_20261119/9179/repeated-wasm-save.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 블록 책갈피와 경계 편집 사례를 준비해 주셔서 감사합니다. 본문·표 셀·머리말·바탕쪽에서 블록의 양 끝과 감싼 글을 확인했습니다. 문단 나누기·합치기 뒤 HWP/HWPX 교차 저장과 재열기에서도 책갈피와 인접 누름틀을 보존했습니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9179/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.
