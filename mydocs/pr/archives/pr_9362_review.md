# PR #9362 검토 — 인라인 중첩 표 뒤 저장 간격과 쪽 배치

## 최종 판정

**메인터너 보정 후 수용 가능** — 제목 표와 본문 표의 같은 쪽 배치를 수용하고, 마지막 셀의 뒤 글줄 원점을 보정했습니다.

## 접수와 적용 이력

- 원 PR head는 0ea2dbbefc0ce5e78839872c9b58e0dd2337e2e2입니다. 2026-11-26 02:30:37 KST에 원 PR reviewer로 jangster77을 지정하고 API에서 확인했습니다. 원 PR 코드 CI는 성공했고 대상은 devel입니다. [원 PR #9362](https://github.com/edwardkim/rhwp/pull/9362)
- 기준 devel 32014de6e0ade1a3baea8852c02270e40b27b345 위에 첫 번째로 적용했습니다. 체리픽은 4fa325da04, 5761c484ff, 251cca8bf1, 905bccec28, 29497a2341, 324490a7a7입니다.
- #9042의 공동 앵커 배치는 이미 #9066 병합 5f78bb8d0297b499043dc4c45d76996549c0b2ef에 반영됐습니다. 기존 2쪽 입력을 최신 devel 및 통합 후보에서 대조했습니다. [#9066 검토](pr_9066_review.md)

## 조판 판단

원 PR은 마지막 행의 저장 내용 하한을 계산할 때 중첩 표의 아래쪽 바깥 여백 안에 들어오는 음수 간격을 수용합니다. 괘선 외곽과 행의 최소 점유를 유지하며 제목 표와 본문 표는 1쪽, 뒤 표는 2쪽에 놓습니다. 원본과 기준 PDF는 한컴 2020 11.0.0.9136 인쇄 영수증의 SHA-256과 일치합니다. [원본과 영수증](../assets/coanchored-inline-tail-negative-spacing/README.md)

원 PR의 합성 입력은 문단 모양 160%를 유지한 채 호스트 저장 간격을 −56 HWPUNIT, 뒤 글줄 원점을 1592 HWPUNIT로 기록했습니다. 한컴 2020 11.0.0.9172에서 재저장하니 호스트 높이 1648 HWPUNIT와 중첩 표 상자가 같았고, 간격은 388과 뒤 원점은 2036 HWPUNIT였습니다. 90% 대조군에서는 한컴이 −64와 1584 HWPUNIT를 저장했습니다.

메인터너 커밋 feb24532e2와 e766629fc7은 인라인 표가 줄간격 계산에 참여하지 않고 같은 문단 모양의 다음 글줄이 현재 글자 크기와 간격을 입증하는 경우, 셀 끝의 간격과 뒤 원점을 렌더 사본에 함께 반영합니다. 4 HWPUNIT 정수화의 정확한 중간값은 검증된 후속 줄의 저장 간격을 썼습니다. 이 렌더 사본을 높이 측정, 분할 및 배치가 공유합니다. 원본 IR과 유효한 압축 간격은 보존합니다.

## 실행 증거

- 한컴 기준 PDF는 2쪽입니다. 최신 devel의 Native·WASM은 3쪽이었고, 통합 후보는 두 경로 모두 2쪽이었습니다. 두 경로 모두 제목 표·본문 표·꼬리 글줄을 1쪽에 한 번씩, 뒤 표를 2쪽에 배치했습니다.
- 1쪽 원 제출 TSV의 Native 98.98%, fresh WASM 98.97%가 각각 100.00%로 올랐습니다. 2쪽은 각 100.00%를 유지했습니다. [Native TSV](../assets/puckqook-9042-integration/negative-spacing/native/silhouette.tsv) · [fresh WASM TSV](../assets/puckqook-9042-integration/negative-spacing/wasm/silhouette.tsv)
- 1쪽의 중첩 표 아래 테두리부터 꼬리 글줄까지 3.0px에서 8.9px로 바뀌었고, 동일 원본을 한컴에서 재저장한 대조본도 8.9px였습니다. 재저장본의 저장 바깥 여백 283 HWPUNIT과 간격 388 HWPUNIT으로 이 관계를 대조했습니다. 90% 대조본은 저장 간격 −64 HWPUNIT를 유지하며 양쪽 출력에서 전쪽 100.00%였습니다. 두 대조본 모두 보정 전후 각 출력 경로의 PNG 바이트가 일치했습니다. [재저장 및 Print 영수증](../assets/puckqook-9042-integration/9362-hancom-controls/receipts.json)
- 추가 회귀 커밋 `d4df7bf7a6`은 160% 원본과 한컴 재저장 160%·90% 대조군에서 표와 글줄의 관계, 1·2쪽 소속, 원본 저장 간격 보존을 검사합니다. 1쪽의 중첩 표 하단, 뒤 글줄, 앞뒤 괘선을 확대해 확인했습니다. [1쪽 Native 비교](../assets/puckqook-9042-integration/negative-spacing/native/review_001.png) · [1쪽 fresh WASM 비교](../assets/puckqook-9042-integration/negative-spacing/wasm/review_001.png)

원 PR과 최종 후보의 96dpi 전쪽 표시값·TSV 해시는 [통합 대조 기록](../assets/puckqook-9042-integration/fixture-audit.json)에 보존했습니다.

## #9412 병합 후 재검증

새 기준을 반영한 제품 소스 `c8c869349`에서도 1·2쪽이 양쪽 출력에서 100.00%였습니다. 1쪽 뒤 글줄과 표 하단 간격은 8.9px였고, 실제 한컴 90% 대조군의 음수 간격을 함께 확인했습니다. [최신 후보 검증 기록](../assets/puckqook-9042-integration/README.md#9412-병합-후-재검증)

## Merge 후 contributor PR comment 계획

1쪽의 제목 표·본문 표 배치 성과를 적고, 160% 합성 입력과 독립 한컴 재저장값의 차이에 대해 메인터너가 공통 렌더 사본을 보정한 근거를 설명합니다. 두 경로의 1·2쪽 표시값과 최종 CI·회귀 결과를 기재합니다. 첫 comment에 1쪽과 2쪽의 Native·fresh WASM 비교 및 overlay를 포함하고 병합 SHA로 고정합니다.

## 통합 반영

[통합 PR #9417](https://github.com/edwardkim/rhwp/pull/9417)의 코드 head `7671c17a0f43590ccce98a5ac9d4c9a665ff1507`에서 로컬 검증과 [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45701811247)가 통과했습니다. [통합 검토 기록](pr_9417_review.md)
