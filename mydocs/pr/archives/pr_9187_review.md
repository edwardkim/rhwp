# PR #9187 리뷰 — fix(field): 각주 지우기·쪽 감추기가 뒤 누름틀의 컨트롤 번호를 함께 옮긴다

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9187](https://github.com/edwardkim/rhwp/pull/9187) · `semanticist21` |
| 원 head | `42d14f0f930a64996c1cc472d8c7b1dfff9e42d4` |
| 적용 커밋 | `7ddcbe2ec6b9` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

앞 각주 제거와 쪽 감추기 삽입·해제 뒤 누름틀의 컨트롤 참조, 이름과 안내문을 확인했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

Rust 누름틀 관계 회귀 2건과 fresh WASM HWP/HWPX 네 교차 저장 순서를 두 편집 경계에서 각각 확인했다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9187/regression.json)

[각 HWP/HWPX 반복 저장의 확인값](../assets/open9_integration_20261119/9187/repeated-wasm-save.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 누름틀의 컨트롤 번호가 앞 각주와 쪽 감추기 편집을 따라가도록 수정해 주셔서 감사합니다. 앞 각주 삭제와 쪽 감추기 전환 후 HWP/HWPX를 교차 저장하고 다시 열어 누름틀 이름·안내문·후속 본문을 확인했습니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9187/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.
