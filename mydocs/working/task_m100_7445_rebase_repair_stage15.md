# 단계15: 저장 그림 호스트의 예약과 배치 끝점 공유

## 선행 분석과 원인

기존 helper는 로컬 보정3b346b6bcd에서 원본 HWP3의 캡션 호스트 줄 점유를 복구했다.
굵은 글자 검사 도입e84f39a095의 원문·프로그램 출처 계약을 유지하면서 경로를 나눠 재현하니
원본/HWPX는 통과하지만 HWP5 저장·재열람에서 그림 뒤 본문이21.4133CSSpx 위로 당겨졌다.
같은 저장 바이트의 독립 engine2020/Hancom11.0.0.9136 Print method0/no preprocess도
기존 본문 위치가 맞음을 확인한다. 원본은 기존 유효한 Hwp2022 Print PDF를 유지했다.

저장 문단의 독립 successor가 host LH+LS와 같은 원본 vpos 전진을 증명할 때 host 줄을
이어받도록 했다. successor는 빈 줄뿐 아니라 실제 본문 한 줄일 수 있다. 변환 계보나
글자 유무로 줄 점유를 대신하지 않으며 원본 HWP3 경로는 유지한다.

줄 점유만 더한 중간 후보에서는8쪽 표가6.77333px 내려갔다. 그림은 원본 문단 앵커에서
배치하지만 예약은 이미 host 줄만큼 전진한 커서 뒤에 그림 전체 띠를 다시 더했다.
뒤 표의 InlinePlacement가 이 과잉 예약을 그대로 소비한 것이 원인이다.

## 공통 결과와 실제 소비

`hwp3_caption_host::stored_flow_end_px`는 원본 문단 앵커에 signed 그림 offset,
위 바깥여백, 공통 그림/캡션 높이, 아래 바깥여백, 소유 host 줄을 더해 물리 흐름 끝을 만든다.
정수항은 각각 px로 환산해 중간 합산 overflow를 피한다.

- 측정 `typeset/section/controls.rs`는 같은 쪽·단 앵커와 가용 예산을 확인하고 이미 예약한
  높이에서 이 끝까지 남은 띠만 더한다.
- 실제 배치 `layout/shape_item.inc.rs`는 같은 저장 줄 질의를 실제 그림 원점에서 소비해
  다음 흐름 끝을 정한다. 그림 잉크 끝과 host 줄 점유는 구분한다.
- 뒤 표는 공통 예약 결과를 InlinePlacement로 받는다. 소유 증거가 없는 일반 그림은
  기존 경로를 유지하며 문서명·쪽 번호·측정한21.4/6.77px 보정 상수는 제품에 넣지 않았다.

## 전쪽 검증

최종 Native540/fresh WASM541,192DPI/compat2022,canonical PDFium smooth_rgb_v1/raw2px90%다.
원본과 두 저장본 모두 독립 PDF와16/16쪽이며 backend별48쪽에90% 미달이 없다.

| 입력 | Native 최저 | fresh WASM 최저 |
|---|---:|---:|
| 원본 issue265 HWP3 |95.34478%|95.33483%|
| 같은 코어의 HWP5 저장본 |99.02719%|98.92066%|
| 같은 코어의 HWPX 저장본 |99.03102%|98.92493%|

같은 HWP5 저장본의 보정 전후3쪽은78.94771/78.97684→99.94409/99.94480%,
8쪽은38.04003/38.13085→99.38620/99.91881%다. 전16쪽 대조에서0.1%p 이상 하락은0쪽이다.
원본 전쪽 TSV는 직전 후보와 양쪽 모두 같다. 이번 보정 전후 비교가 전체 fixed devel 비교를 대신하지 않는다.

Native 원본3/6쪽·저장본1/3/8/12쪽, WASM 원본4쪽·저장본1/3/8/12쪽의 대표 review/overlay를
직접 판독했다(형식별 상세 범위는validation.json). 그림·표·앞뒤 본문 흐름이 복구됐고 원본의
미세한 글리프·간격과 꼬리말 로고 외곽 차이는 남는다. **저장본8쪽 캡션은 PDF1/RHWP3으로
다르며 미해결이다.** 높은 점수나 위치 복구로 이 내용 차이를 완료 처리하지 않는다.

## 회귀와 실행 결과

- 새 `caption_host_followed_by_text_keeps_the_table_and_its_following_body`는 원본/HWPX/HWP5의
  그림·표·앞뒤 문단이8쪽을 한 번씩 소유하고 원문 순서·누락·중복·그림→설명→표→본문 관계를
  지키는지 검사한다. 저장본의 완료8쪽 독립 본문 기준선도±2CSSpx로 보호한다.
  frozen507의 실제1FAIL(481.14667 vs502.01139)→frozen540의1PASS다.
- 기존 굵은 제목의 프로그램 출처·잘못된 보조 메타데이터 대조군·원문/굵기/자간0·한글/영문
  인접 관계·실제 글리프 비확대 검사는 유지했다. 원본3/12쪽은98% 미만이므로 절대 좌표를
  관계 검사로 정리하고 동일 동작은 양쪽99% 이상인 정확한 저장본3/12쪽의 독립±2px 검사로
  보호한다. 저장본3쪽 기대값은 같은 저장 입력의 Print 원점으로 교정했다.
- 정식 현재 manifest의30검사 중28PASS/2FAIL,1702선택 제외,실행48.180초/전체297.149초,
  exit100이다. 이번 캡션/글꼴·원본 수식·쪽번호·1937/4367 대조군은 모두 PASS다.
  두 FAIL은 `issue_1105`의2022변환본 줄 경계와 off_canvas partition15의
  `hwpx/issue1948_cross_para_fieldend.hwpx` 신규1건이며 계속 보정한다. 전체 PASS가 아니다.
- 첫 standalone 진단은 frozen library와 다른 serde_json dependency를 골라 컴파일 실패했다.
  해당 로그를 보존하고 일치하는 dependency로 실행한 실제 FAIL→PASS만 최종 근거로 쓴다.
- 최종 Native build159.746초/exit0, fresh WASM330.877초/exit0이며 source3파일 hash가 일치한다.
  Mac WASM은 locked wrapper의`--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 통과가 아니다.
- root pkg·Studio public hash와 Studio7736 실제GET200/캐시 미사용 초기화 응답을 확인했다.
  첫 observer의 중복 JS 요청 body 오류를 보존하고 깨끗한 브라우저 재시도에서 errors0/
  초기화true를 확인했다. 문서 paint/CanvasKit/actual36 수용을 대신하지 않는다.

[전쪽 TSV·대표 review/overlay·입력/PDF/빌드 출처와 검증 정본](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/caption-stored-physical-flow/verification.md)

[정식 회귀 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair546-caption-physical-flow-gates/nextest.log)

전체 PR 검증은 미완료다. 위 두 회귀, 저장 캡션 번호, Science3쪽 fixed base 대비0.32810%p 하락,
최종 전체 회귀·범위 게이트·전체 시각/actual 검증이 남아 있으므로 PR을 보류한다.
