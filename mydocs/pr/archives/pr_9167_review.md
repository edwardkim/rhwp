# PR #9167 리뷰 — fix(typeset): 쪽보다 긴 각주를 쪽마다 들어가는 줄까지 나눠 이어 싣는다 (#9014)

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9167](https://github.com/edwardkim/rhwp/pull/9167) · `planet6897` |
| 원 head | `813e3e973469a8c58eea83f70d63e1a7882982e6` |
| 적용 커밋 | `e514153e54ed`, `4921c4e5682a` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

각주 30·45·60·100줄의 네 원본에서 본문과 각주가 속한 페이지, 각주 연속 순서 및 100줄 원본 4쪽을 확인했다. 한컴 2020 Print의 번호 표식과 Native/fresh WASM의 줄을 페이지별로 대조했다.

## 메인터너 정리 사유와 범위

원 PR이 긴 각주 분할과 본문 재개 위치를 고쳤고 통합에서도 각주 4종이 그 결과를 재현했다. 통합에서는 원 PR의 Windows CRLF TSV 10개를 저장소의 LF 줄바꿈으로 맞췄다. 쪽별 행과 수치는 원본과 동일함을 확인했다. [정리 커밋](https://github.com/edwardkim/rhwp/commit/f38ba519e0)

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

각주 연속 회귀 3건과 Native/fresh WASM 11쪽 모두 통과. 기준 devel의 100줄 입력은 3쪽이었으며, 독립 Print와 통합 후보는 4쪽이다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9167/regression.json)

Native 4쪽 [raw PNG](../assets/open9_integration_20261119/9167/footnote-l100/native/raw-004.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9167/footnote-l100/native/pdf-004.png) · [review](../assets/open9_integration_20261119/9167/footnote-l100/native/review-004.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/native/overlay-004.png) · [전쪽 TSV](../assets/open9_integration_20261119/9167/footnote-l100/native/silhouette.tsv)

fresh WASM 4쪽 [raw PNG](../assets/open9_integration_20261119/9167/footnote-l100/wasm/raw-004.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9167/footnote-l100/wasm/pdf-004.png) · [review](../assets/open9_integration_20261119/9167/footnote-l100/wasm/review-004.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/wasm/overlay-004.png) · [전쪽 TSV](../assets/open9_integration_20261119/9167/footnote-l100/wasm/silhouette.tsv)

[쪽별 각주·본문 보존](../assets/open9_integration_20261119/9167/page-content-audit.json)

## 독립 Print 입력

[footnote-l100 원본](../../../samples/issue9014/footnote-100-lines.hwpx) · [한컴 Print](../../../pdf/issue9014/footnote-100-lines-2020.pdf)

[footnote-l30 원본](../../../samples/issue9014/footnote-30-lines.hwpx) · [한컴 Print](../../../pdf/issue9014/footnote-30-lines-2020.pdf)

[footnote-l45 원본](../../../samples/issue9014/footnote-45-lines.hwpx) · [한컴 Print](../../../pdf/issue9014/footnote-45-lines-2020.pdf)

[footnote-l60 원본](../../../samples/issue9014/footnote-60-lines.hwpx) · [한컴 Print](../../../pdf/issue9014/footnote-60-lines-2020.pdf)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 긴 각주를 여러 쪽에 이어 싣고 본문이 다시 시작하는 위치를 보존하는 수정과 대조군을 준비해 주셔서 감사합니다. 30·45·60·100줄 입력은 데스크톱 한컴 2020 Hancom PDF Print와 Native·fresh WASM 전쪽이 모두 100.00%입니다. 60줄 2쪽과 100줄 2·3쪽의 이어받은 각주, 100줄 4쪽의 본문 재개와 마지막 각주 소속을 확인했습니다.

[Visual Sweep 병합 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 입력·쪽 | 출력 | raw·독립 Print | review·overlay |
| --- | --- | --- | --- |
| footnote-l30 1쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l30/native/raw-001.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l30/native/pdf-001.png) | [review](../assets/open9_integration_20261119/9167/footnote-l30/native/review-001.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l30/native/overlay-001.png) |
| footnote-l30 1쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l30/wasm/raw-001.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l30/wasm/pdf-001.png) | [review](../assets/open9_integration_20261119/9167/footnote-l30/wasm/review-001.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l30/wasm/overlay-001.png) |
| footnote-l30 2쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l30/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l30/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l30/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l30/native/overlay-002.png) |
| footnote-l30 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l30/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l30/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l30/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l30/wasm/overlay-002.png) |
| footnote-l45 1쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l45/native/raw-001.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l45/native/pdf-001.png) | [review](../assets/open9_integration_20261119/9167/footnote-l45/native/review-001.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l45/native/overlay-001.png) |
| footnote-l45 1쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l45/wasm/raw-001.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l45/wasm/pdf-001.png) | [review](../assets/open9_integration_20261119/9167/footnote-l45/wasm/review-001.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l45/wasm/overlay-001.png) |
| footnote-l45 2쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l45/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l45/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l45/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l45/native/overlay-002.png) |
| footnote-l45 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l45/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l45/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l45/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l45/wasm/overlay-002.png) |
| footnote-l60 2쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l60/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l60/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l60/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l60/native/overlay-002.png) |
| footnote-l60 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l60/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l60/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l60/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l60/wasm/overlay-002.png) |
| footnote-l60 3쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l60/native/raw-003.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l60/native/pdf-003.png) | [review](../assets/open9_integration_20261119/9167/footnote-l60/native/review-003.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l60/native/overlay-003.png) |
| footnote-l60 3쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l60/wasm/raw-003.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l60/wasm/pdf-003.png) | [review](../assets/open9_integration_20261119/9167/footnote-l60/wasm/review-003.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l60/wasm/overlay-003.png) |
| footnote-l100 2쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l100/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/native/overlay-002.png) |
| footnote-l100 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l100/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/wasm/overlay-002.png) |
| footnote-l100 3쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l100/native/raw-003.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/native/pdf-003.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/native/review-003.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/native/overlay-003.png) |
| footnote-l100 3쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l100/wasm/raw-003.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/wasm/pdf-003.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/wasm/review-003.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/wasm/overlay-003.png) |
| footnote-l100 4쪽 | native | [raw](../assets/open9_integration_20261119/9167/footnote-l100/native/raw-004.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/native/pdf-004.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/native/review-004.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/native/overlay-004.png) |
| footnote-l100 4쪽 | wasm | [raw](../assets/open9_integration_20261119/9167/footnote-l100/wasm/raw-004.png) · [pdf](../assets/open9_integration_20261119/9167/footnote-l100/wasm/pdf-004.png) | [review](../assets/open9_integration_20261119/9167/footnote-l100/wasm/review-004.png) · [overlay](../assets/open9_integration_20261119/9167/footnote-l100/wasm/overlay-004.png) |
