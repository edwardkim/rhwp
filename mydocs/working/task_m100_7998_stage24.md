# #7998 같은 축의 서로 다른 굵기 — 평행 경계의 행·열 상속

- 시작 SHA `127796ea9`, 2026-10-19 다음 절편 진행 승인.
- 같은 축의 모든 선을 같은 굵기로 제한한 기존 가정을 제거한다. 첫 경계는 세로선마다
  굵기가 다를 때의 행 삽입·삭제, 가로선마다 다를 때의 열 삽입·삭제다. 기존 물리 변을
  길게 이어가는 것으로, 새 공유 변에서 서로 다른 원본 스타일이 충돌하는 방향은 원자적으로
  거절한다. 연속 불투명·같은 색·완전한 직사각 격자와 기존 원점/순서 증명을 유지한다.
- 독립 입력은 공개 `nonuniform-axis.pdf`의 세로 0.90/0.70pt·가로 0.50pt다. 수평 대조군은
  동일 생성 방식에서 가로선만 서로 다르게 선언한다. 원본 선언 폭과 자식/본문 보존, 양 저장,
  실제 행·열 API·삭제·Snapshot 복원 및 독립 한컴 Print로 기대 결과를 확인한다.
- 소비: source ink → `Proof::complete`의 물리 edge → 경계선별 pen 배열 → 실제 Table 구조
  편집 → `border_paint_bands::apply`의 셀 안/바깥 반쪽 → RenderTree·HWP/HWPX 저장이다.
  줄 구성·높이·pagination은 변경하지 않는다. 같은 경계선의 interval마다 다른 pen·색·
  dash/alpha·부분/병합/가림·미증명 원점과 R2/P5는 남긴다.
- 실행 증거는 ignored `output/pr-review/issue7998/p4-variable-border-inheritance/`에 보존한다.
  작은 수정 전 FAIL/수정 후 PASS와 영향 범위의 Native/fresh WASM 전쪽 90%·직접 판독 뒤
  정식 회귀를 등록한다. 긴 전체 검증·최신 devel 통합·원격 제출은 P5 경계다.
  `parser_allocation_bounds_proven=false`를 유지한다.

## 작은 경계와 원본 단위 진단

- `before.log`에서 stage23 runtime이 동일 공개 입력의 행 삽입을 미지원으로 거절했다.
  실제 API의 의도한 원인 FAIL이며 빌드 실패가 아니다.
- 첫 per-line 구현은 원본 0.50pt의 서로 다른 crop을 정수화한 뒤 49/50HU로 구별해
  다시 거절했다(`after-first.log`, `contracts.log`). 독립 crop/PNG frame 투영에서
  49.342105 / 50.223214HU를 확인했다(`projected-source-pens.json`). 원래 1HU 안의
  같은 pen은 정수 반올림 **전**에 축의 source 대표값과 대조해야 한다. 기존 오차 한도를
  늘리거나 편집 후 보이는 폭을 clamp하지 않는다. 보완 뒤 최종 runtime을 다시 준비한다.
- 첫 GDB 진단은 stripped release-test 바이너리에 소스 줄 심볼이 없어 중단점을 잡지
  못했다. 이를 제품 원인 증거로 사용하지 않았으며 공개 source witness의 저장 crop과
  원본 frame을 직접 읽어 관측을 남겼다.
- MCP initialize·tools/list는 HTTP 200이며 아직 변환 실행·Print 성공을 입증한 것은 아니다.

## 최종 Native와 Print 작업

- 보완 후 `after-final.log`의 같은 행 삽입이 PASS다. 실제 6종 편집의 양 저장 12개와
  재열기 후 반복 삽입·삭제, source pen·빈 셀/자식·본문 보존·Snapshot 복원이 통과했다
  (`contracts-final.log`). 지원하지 않는 축 변경/길이 방향 pen 변화의 12건은
  `reject-final.log`에서 바이트 동일한 원자 거절을 유지했다.
- 기존 정식 3모듈은 31건 통과·1건 제외다. 제외한 기존 nonuniform-axis 일괄 거절 검사는
  이번에 지원하는 평행 방향까지 거절하도록 요구하므로, 독립 시각 선행 검증 후 적용/비적용
  방향을 나눠 갱신한다. 이를 모든 기존 검사가 통과한 것으로 기록하지 않는다.
- 기존 축별 일정 pen 대조군 14개의 새 Native 편집 저장본은 stage23 수용본과 바이트 동일하다
  (`uniform-compat/compared-exports.json`). 이전 제품 결과와 새 실행을 구별한다.
- Native의 새 원본 3개는 전쪽 각 1쪽, raw 최저 99.98109%다. 편집 저장본은 별도 Print
  비교를 계속한다. 새 fixture·정식 회귀는 아직 등록하지 않는다.
- MCP 첫 HWPX Print가 succeeded이며 2020 11.0.0.9136·print_method 0·한 쪽씩 driver 경로를
  확인했다. 같은 요청을 재제출하지 않고 양 형식 12개를 독립 인쇄로 연결한다.

## 최종 검증과 직접 판정 준비

- 최종 Native SHA-256 `a3d840a38d21bfb9f642a042e138623d937a43122c5477ce1a8fdfe25f32c35b`,
  fresh Docker WASM 19,604,962바이트의 SHA-256은
  `a2caa4a364a97506621f5f82b24e491b9f0e34558a0479d53af3101ffa274415`다.
  glue는 기존 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`와
  같다. Native와 Docker가 같은 두 제품 파일을 소비했고 `pkg`/Studio public/보존 패키지
  바이트를 확인했다(`source-runtime.json`). 최종 제품 코드 이후에는 fixture·테스트·문서만
  변경했으므로 빌드를 반복하지 않는다.
- Native/fresh WASM 각각 **29입력·29쪽**이다. 원본 3쪽·새 편집 양 형식 12쪽·기존 일정
  pen 편집 대조군 14쪽이며 최저는 **97.45731% / 96.19767%**, 누락·쪽수 차이·미달 0이다.
  새 편집 12쪽 최저는 **98.27716% / 97.20590%**다(`final-validation-summary.json`).
  기존 대조군의 현재 재출력 28개 PNG와 Native/실제 WASM 저장본 14개는 stage23 수용본과
  바이트 동일하다. 이는 현재 실행의 같은 backend 비교이며 최신 devel/P5 비교를 대신하지 않는다.
- 실제 Studio의 새 6종 SnapshotCommand·문자 삽입/삭제·undo/redo·양 저장·재열기가
  통과했다(`studio-result.json`). 새 저장본 12개는 Native와 바이트 같다. 기존 일정 pen
  7종도 같은 실제 소비에서 통과했다(`uniform-compat/studio-result.json`). 직접 문자
  역삭제 원점의 R2 완료를 뜻하지 않는다.
- 실제 앱의 `open-document-bytes` 수명주기와 마우스 클릭에서 두 원본·행/열 저장본의
  중첩 셀 Picture는 0개, 글자 캐럿은 2단계 cellPath로 들어갔다. 실제 2×2 내용 그림은
  선택 가능하다(`click-after.json`). 사용자 기존 탭/WIP는 보존하고 네 새 탭만 열었다.
- MCP의 첫 HWP는 출력 안정화 60초 제한으로 실패했지만 같은 입력의 별도 후속 job이
  성공했다. 다른 HWP도 성공했으므로 첫 실패를 파일 손상으로 단정하지 않는다. 한 번의
  대기열 8개 초과 거절에는 job ID가 없었고, 원래 실패/거절 증거와 후속 job을 구별했다.
  최종 **HWP 6개·HWPX 6개 모두** 실제 2020 Print/1-up 성공·입력 SHA·독립 PDF SHA를
  확인했다(`print-provenance.json`). 이전 단계의 Print PDF로 새 입력을 대신하지 않았다.
- 독립 Print와 Native/fresh WASM의 행·열 삽입/삭제, 자식 표·앞뒤 문장·빈 새 셀을 같은
  영역에서 확대 판독했다. 외곽·공유 선 위치와 내용 보존은 일치하며 Helvetica 대체 글꼴과
  기존 가는 선/공유 선 농도 차이는 남긴다. 대표 compare·standalone overlay·review와
  2배 확대를 준비했고 점수만으로 직접 판정을 대체하지 않는다.
- 새 정식 회귀 2개와 기존 nonuniform-axis 거절의 방향별 수용 조건을 기존 자동 수집
  파일에 연결했다. 같은 선의 길이 방향 pen 변화와 새 공유 변 충돌은 원자적으로 거절하고,
  평행 방향의 새 셀·원본 pen·자식/본문·양 저장·반복 삽입/삭제·Snapshot 복원을 검사한다.
  원래 row 삽입 FAIL은 `before.log`, 보정 후 같은 API PASS는 `after-final.log`다.
- 첫 새 검사는 원본 IR과 HWP의 전체 JSON을 비교해 합성된 raw header/cache 바이트까지
  같아야 한다고 요구했다. 실제 소유/기하·문자/LineSeg는 같았으며, 형식 간에는 이 계약을
  검사하고 형식 안의 후속 편집에는 전체 재열기 subtree 보존을 검사하도록 바로잡았다.
  제품 코드는 바꾸거나 기준값·오차를 완화하지 않았다. 원래 실패는 `guards-final.log`,
  최종 기존 3모듈의 **34 passed / 0 failed / 0 filtered**는 `guards-final-normalized.log`다.
  CI 호출과 전체 suite는 추가 실행하지 않았다.
- 새 두 PDF와 기존 네 PDF는 공개 생성기로 별도 폴더에서 바이트까지 재현했다. 모든
  원본/편집 범위가 양 backend 90% 이상이고 위 직접 확대를 마친 뒤에만 회귀/fixture를
  등록했다. ±2px 핀이나 전체 SVG hash를 새로 고정하지 않았다.
- 이동 가능한 비교는 `E:\tmp\issue7998-parallel-border-review\index.html`이다.
  네 Studio 새 탭은 `?review=issue7998-parallel-{vertical-source,vertical-row,
  horizontal-source,horizontal-column}&lang=ko`다. 모두 최종 WASM SHA·1쪽·중첩 4셀/빈 그림
  0개를 확인했다(`user-cdp-tabs.json`). 메인테이너 직접 시각·편집 구조 판정을 요청했다.

2026-10-19 메인테이너가 **“시각·편집 구조 통과”**로 수용했다. 이번 절편을 R1/P4
전체 완료로 표시하지 않으며, 같은 축
자체를 바꾸는 상속 충돌·다른 스타일·병합/지역 축·미증명 원점과 R2/P5·원격 제출은 남는다.

수정 전의 보존된 stage23 WASM으로도 두 새 원본을 실제 Studio에서 각각 열어 동일 행/열
API를 실행했다. 두 평행 방향 모두 의도한 PDF 경계 오류로 원자 거절됐다
(`before-wasm.json`, `before-wasm.log`). 현재 WASM의 동일 두 입력은 위 6종 명령에서
성공했다. 보존된 이전 패키지를 재사용했으며 이전 소스의 빌드를 반복하지 않았다.

## 메인테이너 판정과 단계 보존

2026-10-19 새 네 탭·검토 화면의 **시각·편집 구조 통과**를 기록했다.
`maintainer-verdict.json`에 새 저장본 12개의 SHA·같은 source/runtime·집중 34건을 연결한다.
제품 코드는 검증 뒤 그대로이며, 통과한 빌드·캡처·집중 검사를 다시 실행하지 않는다.
소스·정식 회귀·독립 기준 PDF·계획/보고서를 같은 로컬 단계 커밋으로 보존한다.
새 공유 변의 스타일 충돌·다른 스타일/병합/지역 축·미증명 원점과 R2/P5는 다음 범위다.
