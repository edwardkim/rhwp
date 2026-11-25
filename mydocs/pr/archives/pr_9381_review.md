# PR #9381 검토 — 같은 앵커에 저장된 두 부동 표의 쪽 소속

## 최종 판정

**수용 가능** — 13pt 제목 뒤 첫 표와 둘째 표의 3쪽 배치, 10pt 대조 문서의 2쪽 배치를 각각 확인했습니다.

## 접수와 적용 이력

- 원 PR head는 915ea55a5ff3bb476c80fb61e5c07b1c78b614fd입니다. 2026-11-26 02:30:47 KST에 원 PR reviewer로 jangster77을 지정하고 API에서 확인했습니다. 대상은 devel이며 원 PR 코드 CI는 성공했습니다. [원 PR #9381](https://github.com/edwardkim/rhwp/pull/9381)
- 세 번째로 1301276a7d, f80024e76e를 적용했습니다. #2439는 역사적 관련 이슈입니다.

## 조판 판단

저장 HWP5에서 제목·호스트 글줄의 현재 모양과 두 표의 본체·여백을 대조해 공통 프레임을 만듭니다. 처음 표와 공백 호스트 줄의 점유 끝을 같은 쪽에 기록하고 이월된 둘째 표는 새 쪽의 본문 원점부터 배치합니다. 13pt 입력에서는 제목과 두 표가 1·2·3쪽, 10pt 입력에서는 제목과 첫 표가 1쪽에 이어지고 둘째 표가 2쪽에 놓였습니다.

두 공개 HWP와 한컴 2020 11.0.0.9136의 1-up Print PDF는 원 PR 영수증의 해시와 일치합니다. [원본과 영수증](../assets/stored-leading-float-page-break/README.md)

## 실행 증거

- 두 원본에서 기준 devel은 각각 1쪽을 냈습니다. 최종 후보는 Native와 fresh WASM 모두 한컴 Print의 3·2쪽이었고 각 원본의 원 제출 표시값 100.00%를 모든 쪽에서 유지했습니다. [13pt Native TSV](../assets/puckqook-9042-integration/negative-anchor/native/silhouette.tsv) · [13pt fresh WASM TSV](../assets/puckqook-9042-integration/negative-anchor/wasm/silhouette.tsv) · [10pt Native TSV](../assets/puckqook-9042-integration/negative-anchor-fits/native/silhouette.tsv) · [10pt fresh WASM TSV](../assets/puckqook-9042-integration/negative-anchor-fits/wasm/silhouette.tsv)
- 13pt 원본의 제목과 각 표, 10pt 원본의 제목–첫 표 간격과 다음 쪽 본문 원점·괘선을 확대해 판독했습니다. 표의 문자열이 각각 해당 쪽에 한 번만 나타나고 표를 포함한 쪽이 모두 Canvas에 출력됨을 Chrome Studio에서도 확인했습니다. [13pt 둘째 표 비교](../assets/puckqook-9042-integration/negative-anchor/wasm/review_003.png) · [10pt 첫째 표 비교](../assets/puckqook-9042-integration/negative-anchor-fits/wasm/review_001.png)

원 PR과 최종 후보의 96dpi 전쪽 표시값·TSV 해시는 [통합 대조 기록](../assets/puckqook-9042-integration/fixture-audit.json)에 보존했습니다.

## #9412 병합 후 재검증

새 기준을 반영한 제품 소스 `c8c869349`에서 13pt·10pt 원본의 양쪽 출력을 새로 확인했습니다. 각각 3·2쪽이며 원 PR 제출 표시값 대비 하락이 없었습니다. [최신 후보 검증 기록](../assets/puckqook-9042-integration/README.md#9412-병합-후-재검증)

## Merge 후 contributor PR comment 계획

제목 크기 두 조건의 3쪽·2쪽 배치와 표 사이 경계를 적고 두 경로의 전쪽 표시값과 최종 CI·회귀 결과를 기재합니다. 첫 comment에 13pt 원본의 1–3쪽과 10pt 원본의 1–2쪽 Native·fresh WASM 비교 및 overlay를 포함하고 병합 SHA로 고정합니다.

## 통합 반영

[통합 PR #9417](https://github.com/edwardkim/rhwp/pull/9417)의 코드 head `7671c17a0f43590ccce98a5ac9d4c9a665ff1507`에서 로컬 검증과 [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45701811247)가 통과했습니다. [통합 검토 기록](pr_9417_review.md)
