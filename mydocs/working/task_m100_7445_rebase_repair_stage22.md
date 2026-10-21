# #7445 보정22 — 공유 괘선의 원문 소유와 최종 Print 좌표 검사

Issue: #7445. parent `16abb05616`, base `2e3b4f880bd11d35398e6ea0100846611f13a72d`.
제품 소스 변경은 없으며, Stage20 전체 회귀에서 실패한 기존 검사1건의 좌표계 혼동을 보정했다.

## 원 계약과 실제 실패

기존ef3e49f648/1d09623ae9는 공유 수평선의 아래 셀 색상 소유와 검은 하단→흰 상단
덧그리기 순서·compound coverage를 보호했다. 독립 Print의 짧은 회색 잉크를 실제 Native PNG로
확인하는 두 번째 검사도 포함한다. 이 계약과 정상 검은 경계의 단일 획은 유지한다.

Stage20은 표 괘선을 논리 RenderTree에서 최종 Print 물리 종이 좌표로 전달한다.
기존 `matching_paint_strokes`가 논리 선 좌표와 paint 좌표의 완전 동일성을 신원으로 사용하여
실제로 발행한 두 획을 모두 찾지 못했다. 색상 순서가[]로 관측됐으나 PNG 잉크 검사는 통과했다.
기존 검사 source/현재787 library 실행은1PASS/1FAIL이다(802 before).

원문 Line 노드의 `id`→LayerNode의 `source_node_id`로 같은 획을 찾게 했다.
두 물리 획의 좌표 동일성·수평 방향·인접 정상 경계와의 같은 높이를 검사한다.
검정→흰색 순서·compound_component·인접 경계1회 발행·원본 셀 소유는 그대로다.
완료28쪽의 독립 PDF 좌표±2px도 유지하되, 논리 좌표 대신 실제 발행된 Print 선에 적용했다.
Native PNG의 연속 회색 잉크 검사는 변경하지 않았다. 테스트 삭제·허용치 확대·renderer 되돌리기는 없다.

## 먼저 완료한 같은 원본 전66쪽 검증

현재 Native787/fresh WASM791은16abb의 같은 제품 소스다. canonical PDFium smooth_rgb_v1,
192DPI/compat2022/raw2px90%, 원본 개인정보의 명시 글꼴 환경과 기존 세 공급 경로를 유지했다.
원본에 선언되지 않은 HFT 기호 검토용 overlay를 제품 글꼴로 주입하지 않았다.
8쪽씩/캡처8/공유 lock으로 새 전쪽 TSV를 산출했다.

| 경로 | 쪽수 | 최저/쪽 |90% 미만·누락|
|---|---:|---:|---:|
|Native|66/66|94.49334% /32쪽|0|
|fresh WASM|66/66|94.53407% /32쪽|0|

9·28쪽은 양쪽 raw100%다. 변경 검사와 같은28쪽의 양쪽 review/overlay를 직접 열어
짧은 회색 경계·옆 검은 경계·판단근거 셀·본문/쪽번호 소유를 대조했다. 일부 글자/선 가장자리의
색 차이가 남으며 raw100%를 RGB 완전 일치라고 쓰지 않는다. 이 결과는 선언 환경66쪽이며
default 글꼴 환경·다른 문서·actual36 또는 PR 전체 완료를 대신하지 않는다.

[전쪽 TSV·원본/PDF/글꼴/생산자·검사 결과](../pr/assets/issue7445/completed-20261009/stage22-20261021/verification.md)

[Native28쪽 review](../pr/assets/issue7445/completed-20261009/stage22-20261021/native-p028-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage22-20261021/native-p028-overlay.png)

[fresh WASM28쪽 review](../pr/assets/issue7445/completed-20261009/stage22-20261021/wasm-p028-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage22-20261021/wasm-p028-overlay.png)

## 실제 실행과 잔여

같은787 library에서 원 검사1PASS/1FAIL→보정한 기존2검사2PASS(802)다.
prepare 후 자동 nextest804도2PASS/0FAIL/260미선택, 실행0.768초/전체12.796초/exit0이다.
manifest 및 fmt check를 확인했다. 제품 변경이 없으므로 테스트 보정 후 새 WASM 빌드를 중복하지 않았다.
로그는 ignored output의repair800/802/804에만 보존했다.

Stage20의11실패 중 다단4건과 이번 괘선1건의 집중 보정만 확인한 상태다.
새 전체 nextest의 실패 수나 전체 통과로 환산하지 않는다. 캡션·CSAP·머리말 그림·PrEP 격자와
과학 기준선 잔여, 최신 전체 corpus/base 비교·게이트·실제 소비 경로는 계속 처리한다.

디스크는 종료된 이전 캡처의 생성 SVG8개를 SHA manifest 후 정리해0.660GiB를 확보했다(798).
같은 이전 캡처의 글꼴1872개를 실제 SHA로 대조해39개 내용으로 하드링크하고 모든 경로·내용을
재검증하여12.639GiB의 중복을 줄였다(799). 원본·PDF·PNG·TSV·로그·공유 target은 변경하지 않았다.
글꼴 경로는 모두 남아 있으며, 제거된 SVG8개만 재사용 가능한 증거로 주장하지 않는다.
