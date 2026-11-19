# PR #9184 리뷰 — fix(paste): 한/글 복사본 붙여넣기 — 누름틀·필드 끝 두 표기·글자 안 탭 보존, 모르는 개체는 HTML 경로로

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9184](https://github.com/edwardkim/rhwp/pull/9184) · `emptinessform` |
| 원 head | `b4c5efd3794580263e83a211d629f7133c4a899c` |
| 적용 커밋 | `2697f6b622fc` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |
| 기여 이력 | [PR #8300 병합](https://github.com/edwardkim/rhwp/pull/8300) |

## 변경과 검토 범위

한/글 클립보드의 두 필드 끝 표기, 글자 안 탭, 미지원 개체의 HTML 붙여넣기를 확인했다. 새 브라우저 탭의 실제 Studio 붙여넣기에서도 검증했다.

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

Rust 변환 6건·Studio 선택 계약 6건 통과. 실제 Studio에서 누름틀 하나와 안내문·뒤 본문, HWP/HWPX 각각 두 번의 재저장, 미지원 모델의 HTML 결과를 확인했다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9184/regression.json)

[실제 Studio 변환·저장 확인](../assets/open9_integration_20261119/9184/studio-roundtrip.json)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 한/글 클립보드의 누름틀과 필드 끝 두 표기, 글자 안 탭을 처리해 주셔서 감사합니다. 실제 새 Studio 탭에서 두 표기의 붙여넣기, HWP/HWPX 반복 저장과 재열기, 지원하지 않는 개체의 HTML 대체를 확인했습니다. 말씀해 주신 경계를 다시 검사할 수 있도록 결과를 개별 리뷰에 남겼습니다.

[해당 기능의 회귀 실행](../assets/open9_integration_20261119/9184/regression.json)과 위 편집·저장 검증을 코멘트에 연결한다.
