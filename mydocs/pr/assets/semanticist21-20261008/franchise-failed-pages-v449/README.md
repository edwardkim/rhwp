# 가맹사업 최종 Native 미달 쪽 — 보정 진행 중

생산 Rust `7749d712ca338d1883cc092d8020133b312082f4`, 고정 base `552f5228d2dbfd6046c838f14241704f1bcb8a05`의 검증이다.
원본과 독립 한컴2020 Print, 새 전142쪽 TSV의 생산 기록·해시는 [provenance](provenance.json)에 연결한다.
Native/Print는142/142쪽이고 미달은55·57·79쪽이다. 이 문서는 **직접 보정 대상**이며 이관하거나 승인하지 않는다.

| 쪽 | 2px 실루엣 | review | overlay |
| --- | ---: | --- | --- |
|55|84.64501%|[review](native-p055-review.png)|[overlay](native-p055-overlay.png)|
|57|88.24553%|[review](native-p057-review.png)|[overlay](native-p057-overlay.png)|
|79|85.28897%|[review](native-p079-review.png)|[overlay](native-p079-overlay.png)|

## 직접 판독과 원인 추적

같은 현재 PNG에서 canonical helper로 review·overlay를 생성하고 세 쪽을 실제로 열었다.
표 외곽과 내용·쪽 소속은 대체로 대응하지만 완전한 가운데 정렬 셀의 글줄은 위로 올라가 있다.
55쪽 오른쪽의34개 대응 줄 중 본문은 약4px, 왼쪽은 약2.7px 차이이며,57·79쪽도 같은 저장 앞 띠 차이를 보인다.
원본 첫 LineSeg의300/200HU와 ParaShape 앞 간격600/400(2배 저장)이 각각4/2.6667px에 대응한다.
[원본 메트릭](source-metrics.txt)과 [동일 문장 SVG/Print 기준선 대조](native-print-baseline-differences.json)를 보존한다.

`cell_paragraph_spacing::paragraph_before.stored_lead`는 측정·CellUnit에 들어가지만,
부분 표에서 온전히 남은 Center 셀의 `layout_composed_paragraph` 호출은 첫 저장 앞 띠를 억제한다.
whole-table의 기존 `allow_first_para_lead` 소비 경로와 대조 중이다.
79쪽 마지막 분할 행은 조각의 물리 높이·가운데 정렬·다음 쪽 공간도 추가 추적한다.
진단 숫자를 절대좌표 기대값이나 문서별 예외로 넣지 않는다. 아직 보정 후 통과·완료 증거가 아니다.

가맹사업 fresh 전쪽은 이 미달을 확인한 뒤 시작하지 않았다. 다른 문서의 완료 결과와 분리한다.
원본·Print·PNG·TSV·JSON을 보존하고 종료된 비대표 SVG만 ledger 후 정리했다.
