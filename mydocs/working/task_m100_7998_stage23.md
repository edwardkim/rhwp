# #7998 축별 굵기가 다른 실선의 행·열 상속

- 시작 SHA `8ce598d33`, 2026-10-19 다음 절편 진행 승인.
- 위반: 완전한 같은 색 실선이라도 가로·세로 굵기가 다르면 표 전체가 같은 pen이라는
  stage22 가정 때문에 구조 편집을 거절한다. 축별 원본 pen을 보존한 채 같은 축에 새
  경계를 상속해야 한다. 최근접 표준 굵기나 두 축의 큰 값 하나로 바꾸지 않는다.
- 소비는 인증 source/crop의 물리 잉크 → `Proof::complete` → 축별 pen 결과 → 실제
  Table 행·열 편집 → `border_paint_bands::apply`의 셀 안/바깥 반쪽 → 기존 Rectangle
  paint·HWP/HWPX 저장이다. 요구/본문 높이·줄 소속·pagination은 바꾸지 않는다.
- 같은 축의 pen은 일정하며, 두 축의 색은 같은 불투명 연속 실선인 완전한 직사각
  단일 span 격자가 이번 경계다. source의 한 단위 양자화 안에서 같은 pen이면 이전
  공통 pen을 그대로 사용한다. 반쪽은 각 축의 정수 pen에서 각각 나눠 누적 손실을 피한다.
- 축 안에서도 바뀌는 pen·다른 색의 접점 우선순위·dash/alpha·실제 틈·가림·병합/지역
  축의 일반 상속은 증명 밖이다. R2와 P5도 유지한다. 미지원은 원자적으로 거절하며,
  이 절편을 전체 R1·P4 완료로 표시하지 않는다. `parser_allocation_bounds_proven=false`다.
- 입력은 기존 공개 mixed-pen(가로 0.50/세로 0.70pt)과 독립 재생성한 홀수 0.51/0.73pt·
  Arial 파란 선이다. 원본 단위·색·내용/자식 보존·양 저장/재편집·undo/redo를 먼저
  진단한 뒤 독립 한컴 Print와 Native/fresh WASM 전쪽 및 확대 판독으로 확인한다.
- ignored `output/pr-review/issue7998/p4-axis-border-width/`에 작은 수정 전후 실행·입력·
  로그·TSV/PNG를 보존한다. 새 정식 회귀/fixture는 90%와 직접 판독 뒤 등록한다.
  기존 통과 증거를 재사용하며 변경 영향 캡처만 새 producer로 수행한다. 전체 검증과
  devel 통합·원격 제출은 P5다.

## 작은 경계 진단

- `before.log`: stage22 코드로 같은 mixed-pen의 행 삽입이 미지원 오류로 실패했다.
  입력·실제 API·실행 파일을 보존했고 컴파일 실패를 결함 재현으로 세지 않는다.
- 두 굵기를 분리한 첫 코드도 거절했다(`before-axis-z.log`). `inspect.log`에서 원본 HWP IR의
  가로/세로 그림은 각각 z=1/2임을 확인했다. PDF의 서로 다른 stroke 연산을 같은 z라고
  가정하면 굵기만 나눠도 실제 경로가 열리지 않는다.
- 입력의 가운데 가로선은 별도 Rectangle(z=3), 위아래 가로선은 PNG(z=1), 세로선은
  PNG(z=2)다. source IR을 전부 PNG나 하나의 z로 가정하지 않는다. 같은 색의 불투명
  경계끼리의 순서는 교환할 수 있지만 다른 paint와의 순서는 보존해야 한다. 인증된
  경계만 제거한 candidate와 나머지 본문/중첩 개체의 rank를 검사해, 원래 경계 rank
  구간 사이 또는 같은 rank에 순서 교환을 입증하지 못한 paint가 있으면 거절한다. 이 증명 후 축별 기존 z로
  재생성한다. bbox 근접이나 가려졌을 것이라는 추측으로 다른 개체를 무시하지 않는다.
- 입력에는 같은 원본 PNG의 **변 소유가 없는 조각**도 남아 있다. 이미 인증해 읽은 같은
  색 stroke witness와 기존 `picture_digest`의 중립 그림 계약을 모두 만족할 때만 잉크
  순서의 교환 가능성을 인정한다. 필터·테두리·캡션·그림자 등이 바뀐 그림, 일반 그림과
  미확인 도형을 같은 것으로 추정하지 않는다. 소유 없는 조각은 지우거나 변으로 재분류하지
  않는다. 같은 source를 참조한다는 사실만으로 삭제 권한을 주지 않는다.
- 실제 error 중단점을 잡은 `failure-backtrace.log`·`failure-values.log`는 원본 굵기
  집계 단계의 또 다른 가정을 검출했다. 같은 0.70pt 선의 두 crop은 **69.77848 /
  70.67308HU**로 1HU 안이지만, 표준 0.25mm와의 동등성은 한쪽만 수용돼 None/Some이
  달랐다. 모든 조각이 같은 표준 굵기를 입증할 때만 승격하며, 자격이 다르면 실제
  source pen을 보존한다. 원본 단위 오차 한도를 늘리거나 bucket에 맞춰 clamp하지 않는다.
- 순서 감사의 스택·실행 개수는 200,000으로 제한하고 예약 전에 크기를 확인한다.
  이 국소 제약을 parser 전체 할당 경계 증명으로 승격하지 않는다. 최종 보완 중인
  중간 Native 빌드는 종료하고 최종 source를 다시 준비했다. 미완성 빌드를 최종으로
  표시하거나, 아직 수정 중인 source로 WASM 전체 최적화를 시작하지 않았다.
- 잘못된 진단 probe가 crate-private source witness API를 직접 import해 컴파일에 실패한
  실행은 제품 결함으로 세지 않는다. 공개 IR과 PNG witness의 관측 데이터를 대조했다.

## 진행 중 검증과 직접 인쇄 인계

- 최종 Native producer `c17c69a9ed927547a4907e79c16a391167ce1033def51d75dce0b280cfd947a9`의
  원본 2쪽은 raw 최저 97.15040%, 비수용 대조군 3쪽은 99.97209%다. 기존 집중 17건과
  축별 pen·양 저장 재편집·중립 anchor·미확인 rank의 원자 거절 진단이 통과했다.
  같은 stroke 자원의 **소유 없는** 중립 PNG를 보존하는 양성 진단과 brightness·border를
  바꾼 PNG의 원자 거절 2건도 통과했다. 제품 source는 바꾸지 않았으며 fresh WASM 최적화가
  진행 중이다. 아직 새 편집본의 양 backend Print 비교나 사용자 시각 완료로 표시하지 않는다.
- 첫 자동 Print 요청은 client `fetch failed`만 남겼고 HTTP 상태·job ID는 없었다.
  후속 read-only HEAD는 400, MCP initialize·tools/list는 200으로 응답했다. 연결을 다시
  확인한 뒤 **한 건만** 제출하여 job ID와 queued 상태를 받았다. 원래 실패를 변환 엔진의
  손상 판정으로 해석하거나 통신 실패의 구체적인 원인을 임의 확정하지 않았다.
- 사용자가 서비스 전원공사로 서버가 내려갔다고 알려 직접 인쇄로 전환했다. 이 작업의
  로컬 polling을 중단했고 다른 13건은 제출하지 않았다. 기존 job 기록은
  `remote-print/reconnected/`에 보존하며 새 요청으로 중복 제출하지 않는다.
- `E:\\tmp\\issue7998-axis-border-review\\edited\\`의 7종 × HWP/HWPX 14개가 모두
  최종 Native export와 바이트 동일함을 재확인했다. `row-above`, `row-below`, `col-left`,
  `col-right`, `delete-row`, `delete-col`, `blue-row` 각각을 정상 열림 확인 후 원본 용지·
  100%·한 쪽씩 Print PDF로 요청했다. 같은 폴더에 `<이름>-hwp.pdf`, `<이름>-hwpx.pdf`로
  원본 형식을 구분한다. 이전 stage22 인쇄본과 다른 새 축별 굵기 입력이다.

## 서버 재가동 뒤 상태 확인

- 사용자의 서버 재가동·재시도 지시에 따라 기존 `row-below-hwpx` job ID를 재사용해
  상태를 조회했다. MCP 요청은 정상 응답했지만 `queued`·대기열 7번째이며 시작 시각과
  오류는 모두 null이다. 새 start 요청이나 나머지 13건을 제출하지 않았다. 기존 작업의
  실행을 아직 확인하지 못했으므로 연결 복구와 변환 엔진 실행을 구분한다.
  `remote-print/reconnected/restart-status.json`과 같은 폴더의 status 기록을 보존한다.
  재시도 지시 후에도 동일 job이 1582초째 queued·6번째로 응답했으며, 시작 시각·오류는
  여전히 null이다. `latest-retry-summary.json`에 관측을 남겼고 원격 start는 추가하지 않았다.
- 기다리는 동안 **이미 진행 중이던** fresh Docker WASM 빌드가 성공했다. source 4개가
  Native와 같은 해시인지 확인했고 `pkg`와 Studio public의 WASM·glue도 바이트가 같다.
  WASM 19,621,736바이트의 SHA-256은
  `45038fc39003b892aaed0645bc0cbd01509df15eab45adcc2b0dcc10bc42a414`, glue는 기존
  `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`와 같다.
- 최종 WASM의 실제 Studio 7개 SnapshotCommand·snapshot undo/redo·양 저장·재열기와
  1쪽 표시가 통과했다. 이 14개 저장본은 최종 Native와 바이트 동일하다
  (`studio-result.json`, `actual-identical-inputs.json`). R2 직접 역삭제 원점의 해결을
  뜻하지 않는다. 원본 2개·거절 대조군 3개는 Native/fresh WASM 각 5쪽, raw 최저
  **97.15040% / 97.18802%**, 누락·쪽수 차이 0이다(`source-controls-summary.json`).
  편집 저장본 14개의 독립 Print는 아직 없으므로 이 범위의 시각 완료·정식 회귀 등록은
  보류한다. 같은 제품의 빌드나 통과한 캡처를 다시 실행하지 않았다.

## 직접 인쇄 수령과 중첩 셀의 빈 그림 결함

- 메인테이너가 **14개 HWP/HWPX 모두 한컴 정상 열림**, HWPX 7개를 직접 Print한 PDF를
  제공했다. 각 입력/기준 PDF SHA·Creator/Producer·페이지 크기는 `manual-print-sources.json`이다.
  독립 Print 7개는 HWPX 출처이며, HWP 재열기도 같은 편집의 HWPX Print와 비교했다.
  이를 HWP 직접 인쇄 7건으로 보고하지 않는다. 이 producer의 Native/fresh WASM 각
  19입력·19쪽은 raw 최저 **97.15040% / 96.19767%**, 누락·쪽수 차이 0이었다.
- 하지만 메인테이너가 네 E2E 탭 모두에서 **중첩 표 셀 위에 여러 개체가 겹치는 문제**를
  지적했다. 점수 통과를 편집 구조 통과로 처리하지 않는다. 사용자 blue 탭은 읽기만 했고
  현재 편집본은 `user-blue-current.hwpx`에 별도 보존했다. 기존 탭을 다시 로드하지 않았다.
- `child-source.json` / `child-saved.json`에서 A/B/C/D 각 셀에 부모의 가로·세로 source PNG
  crop 2개, 총 8개가 남아 있음을 확인했다. 셀 자체의 실제 BorderFill과 별개이며 parent
  stroke의 큰 bitmap frame이 descendant cell을 덮는다는 이유로 빈 영역까지 Picture가 됐다.
  `Compound::cell`의 non-target 분기가 무조건 `(piece, None)`를 내보내는 원인이다.
  source와 저장본에서 같은 문제이고, 이번 행 삽입이 자식 개체를 새로 복제한 것은 아니다.
- `empty-paint-fix/before.log`는 기존 최종 Native에서 **8개 → 기대 0개**의 의도한 FAIL이다.
  원본 PDF에는 이 셀 안에 글자와 자체 테두리만 있다. 원본 HWPX PNG를 독립 Pillow로
  해석해 crop 주변 2px까지 alpha=0인 것도 8개 모두 확인했다
  (`independent-empty-alpha.json`). 구현의 삭제 판단에 pixel threshold를 쓰지는 않는다.
- 인증된 원본 stroke의 폭을 포함한 outward HU 잉크 영역이 non-target 조각과 교차하지
  않을 때만 Picture 발행을 생략한다. 실제 잉크가 교차하거나 인증되지 않은 경로는 기존
  소유·출력을 보존한다. 단순 bbox·같은 resource ID나 hit-test 필터로 개체를 숨기지 않는다.
  Native 작은 경계 후 fresh WASM·편집/저장·전쪽 출력을 다시 검증한다. 기존 producer와
  캡처는 보존하고 새 증거는 `empty-paint-fix/`로 분리했다. 아직 시각·편집 구조 완료가 아니다.

## 빈 그림 제거 보정본 — 직접 판정 수용

- 최종 Native `e9f9691bc2da59d1484d822353756f4e78b9d0a97cfde3140447648615cafeee`,
  fresh Docker WASM 19,608,425바이트
  `f23f34eb39a717547f5eb55347143048a2ebdc59a5b1887ce208f22857b8b524`다.
  glue는 기존과 같다. `empty-paint-fix/source-runtime.json`의 runtime 5파일 SHA를 확인했고,
  그 뒤 제품 코드는 바꾸지 않았다. 테스트·fixture·문서만 추가했으므로 같은 빌드는 반복하지 않는다.
- 전쪽 Native/fresh WASM은 각각 **23입력·23쪽**, raw 최저 **97.15040% / 96.19767%**,
  누락·쪽수 차이 0이다. 원본 2개·편집 양 형식 14개·거절 대조군 3개·실제 그림/잉크
  대조군 4개를 포함한다. 수정 전후 **46개 PNG가 모두 바이트 동일**하며, 원래 19입력의
  RenderTree도 중첩 셀 Image를 제외한 모든 노드·좌표가 같다(`all-before-after.json`,
  `render-node-preservation.json`). bbox/paint를 숨긴 결과가 아니라 빈 IR Picture 제거다.
- Native의 원본 2개와 실제 Studio의 7개 구조 편집·양 저장 재열기에서 빈 Picture는
  **8→0개**, A/B/C/D와 실제 셀 테두리는 유지됐다. 실제 Studio 저장본 14개는 최종 Native와
  바이트 같다. 실제 2×2 내용 그림은 양 형식에서 정확한 PNG를 유지한다(`content-guard.log`).
  source pen·정렬/anchor·변경된 비소유 그림의 원자 거절도 다시 통과했다.
- 실제 UI 열기 수명주기와 마우스 클릭은 이전 WASM에서 **빈 그림 선택**, 새 WASM에서
  **2단계 cellPath의 글자 캐럿 진입**으로 바뀐다. 실제 내용 그림은 양쪽에서 선택된다.
  `click-before.json`, `click-after.json`, 클릭 screenshot에 기록했다. 첫 진단은 내용 그림
  대신 작은 테두리 crop을 클릭해 실패했으며 그 로그를 보존했다. 이를 제품 회귀로 세지 않는다.
  수정한 진단은 원본 PDF의 셀 안쪽 내용 그림을 지정한다. hit-test의 제품 조건은 바꾸지 않았다.
- 독립 기준은 제공받은 **기존 HWPX Print 7개**다. 제거된 영역의 alpha=0과 양 backend
  최신 재출력의 완전 동일성, 다른 노드/좌표 보존을 근거로 재사용한다. 새 저장본 SHA는
  기존 인쇄 입력 SHA와 구별해 `manual-print-sources.json`에 기록했다. 2026-10-19 메인테이너가
  `empty-paint-fixed/final-saved/`의 새 HWP/HWPX에 대해 **“새 14개 모두 정상 열림”**으로
  별도 확인했다. 이 확인을 새 인쇄 PDF 14개가 생성됐다는 뜻으로 확대하지 않는다.
- 새 회귀 5개를 기존 `tests/cases/issue_7998_pdf_retained_border_structure.rs`에 연결했다.
  빈 source padding의 비물질화·축별 새 셀/자식 보존·원본 pen·양 저장 반복 편집·비소유
  PNG 보존/변경 거절·실제 내용 그림을 보호한다. 기존 mixed-pen 거절은 지원 범위 확장에
  맞춰 **축 안에서 실제 pen이 다른** `nonuniform-axis.pdf`의 거절로 바꾸고 invalid-index
  검사는 유지했다. 새 입력은 독립 PDF 연산자로 재생성하고 이미 검증한 입력과 바이트 같다.
  native focused harness가 기존 자동 수집 원본 3모듈의 **32 passed / 0 failed / 0 filtered**를
  확인했다(`guards-final.log`). CI 개별 호출은 추가하지 않았다. 전체 표준 runner·P5 검증은 남는다.
- 이동 가능한 비교는 `E:\tmp\issue7998-axis-border-review\empty-paint-fixed\index.html`,
  새 HWP/HWPX 14개는 그 폴더의 `final-saved\`다. 예전 파일과 사용자 편집본은 보존했다.
  Studio 7793의 `?review=issue7998-axis-width-empty-fixed-{source,row,column,blue}&lang=ko`
  네 새 탭에서 같은 새 WASM SHA와 1쪽을 확인했다. 대표 compare/standalone overlay/review,
  확대·전쪽 TSV·실제 클릭 증거가 상대 경로로 연결되고 깨진 링크가 없다.
- 2026-10-19 메인테이너가 새 네 탭과 비교 화면을 **“시각·편집 구조 통과”**로 판정했고,
  새 저장본 14개 정상 열림도 확인했다. `empty-paint-fix/maintainer-verdict.json`에 두 판정과
  새 저장본 14개·검증 runtime의 SHA를 연결했다. 같은 source/runtime의 기존 검사·캡처를 재사용해
  stage23을 로컬 커밋으로 보존한다. 축별 일정 pen의 실선 상속과 빈 crop 비물질화 범위의
  완료이며, 같은 축의 서로 다른 pen·다른 색/순서·dash/alpha·부분 격자·미증명 원점의
  일반 상속, R2·P5·원격 반영은 남는다. `parser_allocation_bounds_proven=false`다.
