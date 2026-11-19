# PR #9182 리뷰 — fix(typeset): 다음 쪽으로 넘어간 표의 문단 거리·표 끝 간격·빈 호스트 글줄·쪽 위 앞 간격을 저장 위치대로 맞춘다 (#9060)

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9182](https://github.com/edwardkim/rhwp/pull/9182) · `planet6897` |
| 원 head | `8253c9bfc93998439b3efe12c213a26e3d1ebc5b` |
| 적용 커밋 | `1eb39c75087e`, `0b7b98eb81b8`, `f5469a46cc76`, `c2685e17bf82`, `fe1ccb7af514`, `842dd13fdd6f`, `4938ed64831f` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

쪽을 넘긴 표의 문단 거리, 빈 호스트 줄과 이어지는 본문 소속을 검증했다. ODA 79·148·152쪽의 표 경계와 후속 본문을 독립 한컴 Print와 대조했다.

## 메인터너 보정 사유와 범위

원 PR이 이월 표의 문단 거리, 빈 호스트 글줄, 쪽 위 간격을 고쳐 ODA 79·148·152쪽의 배치와 전체 159쪽을 복원했다. 통합 검토에서 70쪽 아래 표는 여전히 Print보다 왼쪽에 3.84px 밀려 있었다. 두 출력의 90% 문턱은 넘었지만 같은 원본의 독립 Print와 직접 대조해 표 외곽선과 셀 안 내용이 함께 옮겨진 결함을 확인했고 사용자가 지적한 차이를 보정했다.

가운데 정렬의 여유 폭이 음수일 때 추가 간격을 0으로 처리해 왼쪽 여백을 보존했다. 여유 폭이 있는 같은 쪽의 위 표는 기존 가운데 정렬을 유지한다. 두 표의 관계를 검사해 과폭 표의 시작점과 일반 가운데 정렬을 함께 보호한다. [수정 커밋](https://github.com/edwardkim/rhwp/commit/c714511737) · [회귀 커밋](https://github.com/edwardkim/rhwp/commit/67f14c1936) · [Print 대조](../assets/open9_integration_20261119/9182/oda/attestation.json)

통합에서 Windows CRLF TSV 8개의 줄바꿈도 LF로 정리했다. 쪽별 값은 모두 원본과 같다. [정리 커밋](https://github.com/edwardkim/rhwp/commit/f38ba519e0)

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

ODA는 Native/fresh WASM 모두 Print와 같은 159쪽이다. 고정 devel은 160쪽이다. 공통 159쪽 중 2%p 초과 하락은 0쪽이며 주장 79·148·152쪽은 양쪽 99.95·99.90·99.75%다. 70쪽의 <표 3-21>은 양쪽 바깥 여백까지 포함한 점유 폭이 저장 줄보다 576 HU 더 넓었다. 기존 가운데 정렬이 음수 여유 폭의 절반을 적용해 표 전체를 왼쪽으로 3.84px 밀었다. 메인터너 보정 `c714511737`에서 남는 공간이 있을 때만 가운데 정렬 간격을 더하게 했다. 70쪽을 재출력한 원시 일치율은 Native 90.60% → 99.93%, fresh WASM 90.77% → 99.93%다. 159쪽 최저는 124쪽 Native 96.89%, fresh WASM 96.93%다. 145쪽도 Native 95.77% → 99.54%, fresh WASM 95.79% → 99.52%로 올랐다. 138쪽 Native는 99.98% → 99.97%로 바뀌었으며 표 왼쪽 선은 PDF에 가까워졌다. M2·A1·T34는 양쪽 전쪽 100.00%다.

Studio Canvas2D Print의 70쪽 별도 캡처는 98.46%다. ODA 전쪽 CanvasKit 사전 검사의 128쪽 한도로 고정 devel과 후보 모두 Canvas2D를 사용했다. 짧은 M2 이동 표는 실제 CanvasKit에서 검증했다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9182/regression.json)

Native 70쪽 [raw PNG](../assets/open9_integration_20261119/9182/oda/native/raw-070.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9182/oda/native/pdf-070.png) · [review](../assets/open9_integration_20261119/9182/oda/native/review-070.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-070.png) · [전쪽 TSV](../assets/open9_integration_20261119/9182/oda/native/silhouette.tsv)

fresh WASM 70쪽 [raw PNG](../assets/open9_integration_20261119/9182/oda/wasm/raw-070.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9182/oda/wasm/pdf-070.png) · [review](../assets/open9_integration_20261119/9182/oda/wasm/review-070.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-070.png) · [전쪽 TSV](../assets/open9_integration_20261119/9182/oda/wasm/silhouette.tsv)

[70쪽 위치와 전후 값](../assets/open9_integration_20261119/9182/oda/attestation.json)

Native 148쪽 [review](../assets/open9_integration_20261119/9182/oda/native/review-148.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-148.png)

fresh WASM 148쪽 [review](../assets/open9_integration_20261119/9182/oda/wasm/review-148.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-148.png)

## 독립 Print 입력

[a1 원본](../../../samples/issue9060/a1-small-top-para-offset.hwpx) · [한컴 Print](../../../pdf/issue9060/a1-small-top-para-offset-2024.pdf)

[m2 원본](../../../samples/issue9060/m2-full-t2tall-para-offset-next-page.hwp) · [한컴 Print](../../../pdf/issue9060/m2-full-t2tall-para-offset-next-page-2020.pdf)

[oda 원본](../../../samples/issue9060/oda-1092000-201600017.hwp) · [한컴 Print](../../../pdf/issue9060/oda-1092000-201600017-2020.pdf)

[t34 원본](../../../samples/issue9060/t34-rows4000-para-offset-split.hwpx) · [한컴 Print](../../../pdf/issue9060/t34-rows4000-para-offset-split-2024.pdf)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> 쪽을 넘기는 표의 문단 거리, 빈 호스트 줄과 후속 본문 위치를 보정해 주셔서 감사합니다. ODA는 데스크톱 한컴 2020 Hancom PDF Print와 같은 159쪽이고, 79·148·152쪽은 Native·fresh WASM에서 각각 99.95·99.90·99.75%입니다. 통합 검토에서 사용자가 지적한 70쪽을 독립 Print와 확대 대조했습니다. 원 PR에서 고친 79·148·152쪽과 별도로, 70쪽의 가운데 정렬 표가 저장 글줄보다 넓어 외곽선과 셀 안 내용이 함께 3.84px 왼쪽으로 밀렸습니다. 여유 폭이 양수일 때만 가운데 정렬 간격을 더하는 메인터너 보정과 소유 관계 검사를 추가해 두 경로 모두 99.93%가 됐습니다. Native 138쪽은 99.98%에서 99.97%로 바뀌었으며 표의 왼쪽 선은 Print에 가까워졌습니다. 합성 A1·M2·T34도 대응 한컴 2020·2024 Print 전쪽에서 두 경로가 100.00%입니다.

[Visual Sweep 병합 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 입력·쪽 | 출력 | raw·독립 Print | review·overlay |
| --- | --- | --- | --- |
| a1 1쪽 | native | [raw](../assets/open9_integration_20261119/9182/a1/native/raw-001.png) · [pdf](../assets/open9_integration_20261119/9182/a1/native/pdf-001.png) | [review](../assets/open9_integration_20261119/9182/a1/native/review-001.png) · [overlay](../assets/open9_integration_20261119/9182/a1/native/overlay-001.png) |
| a1 1쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/a1/wasm/raw-001.png) · [pdf](../assets/open9_integration_20261119/9182/a1/wasm/pdf-001.png) | [review](../assets/open9_integration_20261119/9182/a1/wasm/review-001.png) · [overlay](../assets/open9_integration_20261119/9182/a1/wasm/overlay-001.png) |
| a1 2쪽 | native | [raw](../assets/open9_integration_20261119/9182/a1/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/a1/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/a1/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/a1/native/overlay-002.png) |
| a1 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/a1/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/a1/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/a1/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/a1/wasm/overlay-002.png) |
| m2 2쪽 | native | [raw](../assets/open9_integration_20261119/9182/m2/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/m2/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/m2/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/m2/native/overlay-002.png) |
| m2 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/m2/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/m2/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/m2/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/m2/wasm/overlay-002.png) |
| m2 3쪽 | native | [raw](../assets/open9_integration_20261119/9182/m2/native/raw-003.png) · [pdf](../assets/open9_integration_20261119/9182/m2/native/pdf-003.png) | [review](../assets/open9_integration_20261119/9182/m2/native/review-003.png) · [overlay](../assets/open9_integration_20261119/9182/m2/native/overlay-003.png) |
| m2 3쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/m2/wasm/raw-003.png) · [pdf](../assets/open9_integration_20261119/9182/m2/wasm/pdf-003.png) | [review](../assets/open9_integration_20261119/9182/m2/wasm/review-003.png) · [overlay](../assets/open9_integration_20261119/9182/m2/wasm/overlay-003.png) |
| oda 70쪽 | native | [raw](../assets/open9_integration_20261119/9182/oda/native/raw-070.png) · [pdf](../assets/open9_integration_20261119/9182/oda/native/pdf-070.png) | [review](../assets/open9_integration_20261119/9182/oda/native/review-070.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-070.png) |
| oda 70쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/oda/wasm/raw-070.png) · [pdf](../assets/open9_integration_20261119/9182/oda/wasm/pdf-070.png) | [review](../assets/open9_integration_20261119/9182/oda/wasm/review-070.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-070.png) |
| oda 79쪽 | native | [raw](../assets/open9_integration_20261119/9182/oda/native/raw-079.png) · [pdf](../assets/open9_integration_20261119/9182/oda/native/pdf-079.png) | [review](../assets/open9_integration_20261119/9182/oda/native/review-079.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-079.png) |
| oda 79쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/oda/wasm/raw-079.png) · [pdf](../assets/open9_integration_20261119/9182/oda/wasm/pdf-079.png) | [review](../assets/open9_integration_20261119/9182/oda/wasm/review-079.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-079.png) |
| oda 148쪽 | native | [raw](../assets/open9_integration_20261119/9182/oda/native/raw-148.png) · [pdf](../assets/open9_integration_20261119/9182/oda/native/pdf-148.png) | [review](../assets/open9_integration_20261119/9182/oda/native/review-148.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-148.png) |
| oda 148쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/oda/wasm/raw-148.png) · [pdf](../assets/open9_integration_20261119/9182/oda/wasm/pdf-148.png) | [review](../assets/open9_integration_20261119/9182/oda/wasm/review-148.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-148.png) |
| oda 152쪽 | native | [raw](../assets/open9_integration_20261119/9182/oda/native/raw-152.png) · [pdf](../assets/open9_integration_20261119/9182/oda/native/pdf-152.png) | [review](../assets/open9_integration_20261119/9182/oda/native/review-152.png) · [overlay](../assets/open9_integration_20261119/9182/oda/native/overlay-152.png) |
| oda 152쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/oda/wasm/raw-152.png) · [pdf](../assets/open9_integration_20261119/9182/oda/wasm/pdf-152.png) | [review](../assets/open9_integration_20261119/9182/oda/wasm/review-152.png) · [overlay](../assets/open9_integration_20261119/9182/oda/wasm/overlay-152.png) |
| t34 1쪽 | native | [raw](../assets/open9_integration_20261119/9182/t34/native/raw-001.png) · [pdf](../assets/open9_integration_20261119/9182/t34/native/pdf-001.png) | [review](../assets/open9_integration_20261119/9182/t34/native/review-001.png) · [overlay](../assets/open9_integration_20261119/9182/t34/native/overlay-001.png) |
| t34 1쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/t34/wasm/raw-001.png) · [pdf](../assets/open9_integration_20261119/9182/t34/wasm/pdf-001.png) | [review](../assets/open9_integration_20261119/9182/t34/wasm/review-001.png) · [overlay](../assets/open9_integration_20261119/9182/t34/wasm/overlay-001.png) |
| t34 2쪽 | native | [raw](../assets/open9_integration_20261119/9182/t34/native/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/t34/native/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/t34/native/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/t34/native/overlay-002.png) |
| t34 2쪽 | wasm | [raw](../assets/open9_integration_20261119/9182/t34/wasm/raw-002.png) · [pdf](../assets/open9_integration_20261119/9182/t34/wasm/pdf-002.png) | [review](../assets/open9_integration_20261119/9182/t34/wasm/review-002.png) · [overlay](../assets/open9_integration_20261119/9182/t34/wasm/overlay-002.png) |
