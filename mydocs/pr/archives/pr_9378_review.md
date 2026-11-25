# PR #9378 검토 — 글줄 앞 부동 표의 저장 상자와 이어받기

## 최종 판정

**메인터너 보정 후 수용 가능** — 부동 표 뒤에서 행 D만 2쪽으로 이어지는 배치를 수용하고 최신 devel과의 충돌을 해소했습니다.

## 접수와 적용 이력

- 원 PR head는 cc9e5f625deb5d9de4dd87ed9f8b2fc03f2b3999입니다. 2026-11-26 02:30:45 KST에 원 PR reviewer로 jangster77을 지정하고 API에서 확인했습니다. 원 PR 코드 CI는 성공했고 최신 devel과 충돌 상태였습니다. [원 PR #9378](https://github.com/edwardkim/rhwp/pull/9378)
- 두 번째로 a8db39c4d9, bc155ff945를 적용했습니다. table_layout.rs에서 37ba25bf05가 추가한 일반 글줄 앞 간격 배치를 유지하고, 완전한 부동 표 상자를 소유하는 글자 포함 호스트의 분기를 연결했습니다.

## 조판 판단

앞뒤 저장 글줄과 표 본체·바깥 여백이 닫히면 같은 프레임으로 표 원점과 글줄 전진량을 계산합니다. 분할 단위 높이는 호스트 줄에서 이 값을 사용해 표 높이를 한 번 예약합니다. 전체 셀과 잘린 셀 배치는 같은 프레임으로 표 상단을 정합니다. 첫 쪽의 앞 글줄, 작은 표, 호스트 글줄과 A–C행 순서, 둘째 쪽의 D행과 뒤 문단을 대조했습니다.

한 단 선언이 있는 공개 HWP와 한컴 2020 11.0.0.9136 1-up Print의 해시를 대조했습니다. [원본과 영수증](../assets/stored-float-before-visible-line/README.md)

원 PR이 공개한 회귀와 Print 증거의 범위는 한 단 선언이 있는 입력입니다. 원 작성자가 기록한 단 선언 없는 별도 대조 입력의 D행 쪽 차이는 문서에 남아 있습니다. [원 PR의 대조 입력 설명](../assets/stored-float-before-visible-line/README.md)

## 실행 증거

- 기준 devel과 최종 후보가 모두 2쪽입니다. 기준 devel 1·2쪽은 Native 59.37%·48.72%, fresh WASM 59.36%·48.73%였습니다. 최종 후보는 두 경로의 두 쪽에서 원 제출 표시값 100.00%를 유지했습니다. [Native TSV](../assets/puckqook-9042-integration/visible-host/native/silhouette.tsv) · [fresh WASM TSV](../assets/puckqook-9042-integration/visible-host/wasm/silhouette.tsv)
- 1쪽에서 작은 표 하단과 열림 문단, 이어지는 표 A·B·C행의 괘선을 확대해 대조했습니다. 2쪽의 D행, 종료 괘선과 뒤 본문을 읽고 이전 행의 중복과 내용 누락이 없음을 확인했습니다. [1쪽 Native 비교](../assets/puckqook-9042-integration/visible-host/native/review_001.png) · [2쪽 fresh WASM 비교](../assets/puckqook-9042-integration/visible-host/wasm/review_002.png)
- 이전 PR #9411의 표·캡션 경계 13쪽과 #9415의 글상자·크기 보호 경계 4쪽에서 기준 devel과 최종 후보의 양쪽 TSV 및 PNG 바이트가 일치했습니다. [#9411 검토](pr_9411_review.md) · [#9415 검토](pr_9415_review.md)

원 PR과 최종 후보의 96dpi 전쪽 표시값·TSV 해시는 [통합 대조 기록](../assets/puckqook-9042-integration/fixture-audit.json)에 보존했습니다.

## #9412 병합 후 재검증

검토 도중 #9412가 최신 devel에 병합됐습니다. 분할 칸에서 높이를 예약한 저장 호스트의 표와 흐름 밖 겹침 표는 후속 문단 원점을 다시 전진시키지 않도록 `c8c869349`에서 두 조건을 보존했습니다. 공개 입력의 1·2쪽은 양쪽 출력에서 100.00%였고 #9412의 71개 입력·인쇄 조합은 각 출력 경로의 전쪽 PNG가 기준과 같았습니다. [최신 후보 검증 기록](../assets/puckqook-9042-integration/README.md#9412-병합-후-재검증)

## Merge 후 contributor PR comment 계획

1쪽 A–C행과 2쪽 D행·뒤 본문 재현 결과를 알리고, 메인터너 충돌 해소가 기존 일반 문단의 앞 간격과 원 PR의 저장 부동 표 위치를 각각 유지했다는 근거를 적습니다. 두 경로의 1·2쪽 표시값과 최종 CI·회귀 결과를 기재합니다. 첫 comment에 1쪽과 2쪽의 Native·fresh WASM 비교 및 overlay를 포함하고 병합 SHA로 고정합니다.

## 통합 반영

[통합 PR #9417](https://github.com/edwardkim/rhwp/pull/9417)의 코드 head `7671c17a0f43590ccce98a5ac9d4c9a665ff1507`에서 로컬 검증과 [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45701811247)가 통과했습니다. [통합 검토 기록](pr_9417_review.md)
