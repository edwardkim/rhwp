# #7445 보정33 — 생성 TAC 줄의 새 로컬 종료 높이 공유

Issue: #7445. 부모02e5d4ac72, base80968497. 이 단계의 출력·회귀만 정리하며 전체 PR 준비 완료가 아니다.

## 원인과 실제 소비

수도시설3쪽의 표 호스트는 원본 linesegarray가 없어 현재 줄을 생성한다. 해당 표를 새 쪽에 배치할 때 호스트의 옛 누적vpos와 후속 원본 줄의 로컬vpos를 혼용해 뒤 간격1052HU 중526HU만 남겼다. 본문 전체가7.01333CSSpx 위로 이동했다. 글꼴·표 외곽을 바꾼 결함이 아니다.

실제 호출은 `try_place_stored_tac_paragraph → stored_tac::prepare_computed → commit_stored_tac_control → InlineBoxPlacement.advance_end → layout`다. 일반TAC 후처리보다 먼저 확정된 끝점이 paint를 소유했다. 별도 일반 상한 수정으로 해결되지 않는 것을 V7/V8 실제 실패로 확인하고 해당 후보는 수용하지 않았다.

공통 Query는 단일 생성 RowBreak TAC의 바깥 상자와 다음 원본 줄의 닫힘만 확인한다. 다음 스타일 앞 간격도 전달한다. 실제 측정 상자가 같은 줄 높이와1HU 이내로 일치하는 경우 확정 계획의 끝에 전체 간격을 한 번 남긴다. 계획의fit·lazy축·paint가 동일한끝을 소비한다. 원본/합성 사다리, 편집 여부, 가시 본문·캡션·다른 개체 줄은 기존 경로를 유지한다. 문서명·쪽·관측px 분기가 없다.

## 독립 원본과 계보

- 입력: samples/issue6776/36367506-water-facility-approval.hwpx, SHA62ee54efc7fd9f5fd06cbfdb5843edcdbf84074b05ede5571c15078d4f1b14ab.
- 한컴2020 Print: pdf/36367506-water-facility-approval-2020.pdf, SHAfe82e237046659fdd770811dbd30c8f199d155d9737f1a8d0732f19f2049deda.
- 원본 다음 빈 줄12400HU와 제목15100HU, 현재 개체 상자10782+283+283=11348HU. 두 독립 원점이 전체1052HU 간격을 증명한다. 누적 좌표로 원본을 다시 쓰지 않는다.
- 독립 Print 제목cm=[.12,0,0,-.12,0,841], tm.y=1979 → 기준선316.64CSSpx. 잉크 bbox와 baseline을 구분한다.
- #6776의 수도시설 음성 대조는 비TAC그림의 칸 회계를 보호했다. #7642 단계789 및d876fec9ef의 원본 TAC 재시작 간격을 참고하되 생성 줄을 원본 저장 줄로 승격하지 않았다.

## Native 실제 결과

전3쪽99.99211/98.35493/100.00000%, 최저98.35493%, 미달0, PDF와3쪽 일치. 3쪽수정전71.85405→100.00000%. review/overlay에서표뒤제목·후속본문·붙임항목복구를직접확인했다. 엄격RGB 완전일치라는뜻은아니다.

RowBreak HWPX 대조군 전18쪽 최저92.85921%, 미달0. source02e5와동일전쪽결과. fixed809대비4·8쪽기존하락은이번단계해결로주장하지않으며별도보정이남는다.

## fresh WASM과 실제 소비

같은 원본 전3쪽 fresh WASM100.00000/98.36283/100.00000%,최저98.36283%,미달0·쪽수3일치. RowBreak 대조 전18쪽 최저92.82532%,미달0. Native/fresh 각각21쪽은 수정 전02e5 대비0.1%p 이상 하락0쪽이다. 고정809 base 대비 수도시설3쪽도 양쪽하락0이다. RowBreak의 기존 base 하락은 Native4·8쪽, WASM2·4·8·12·17쪽에 남아 전체 승인으로 상쇄하지 않는다.

Mac locked wrapper `--target web --out-dir pkg --no-opt` exit0,약420초의 로컬 대체 빌드다. Docker 최적화 통과가 아니다. Root pkg/Studio public/frozen JS·WASM hash가 같으며, 실제 Studio7736 새 탭 초기화 응답도각HTTP200·frozen hash일치·wasmInitialized=true·오류0이다. 이는 실제문서paint·폰트준비·CanvasKit/Actual36 완료를 뜻하지 않는다.

|생산물|SHA256|
|---|---|
|Native CLI|b08d73c175c99c3b479d51e2d15ad1502d39f80adbcb3f5a35193576dcb4fb34|
|Native library|243ce3413c0a742c49855d403fa8e47384adbdf3cf562dfe5df573a3bc5c4620|
|fresh WASM|cebfd65691287725f707b1ff657511ccd0b7c55a50ee7bb29c1b9d236c51fbb4|
|fresh JS|d7d8cf9b5f7ce8a0f00b49a39aa1fee49e331cce6774e7ba57c3e9d078cd7030|

## 기존 회귀 보완과 실제 실행

`tests/cases/issue_6776_cell_tac_picture_accounting.rs`의 기존 수도시설 음성 대조에 독립 간격·Print 기준선과 뒤 문단 보존 검사를 보완했다. 기존 그림 회계·종이 안 포함·3쪽수 검사는 유지했다. 새 fixture·test source·test attribute·CI 개별 호출은 없다.

- 표는 원본 내용 `구경산출`로 식별한다. 다음 원본 문단의12400HU에서 개체 줄11348HU를 뺀1052HU가 실제 바깥여백 뒤에 온전히 남는지 검사한다. 화면 px로 표를 선택하지 않는다.
- 후속 각 문단의 원문 내용·순서·누락·중복과 같은3쪽/줄 소속을 검사한다. Native/fresh3쪽 raw100%와 직접 review/overlay 판독 후 `조치의견`의 실제 Print PaintOp 기준선을 독립 PDF의316.64CSSpx ±2로 보호했다.
- 최종 정식 source SHA256은 `8a97a1545e2aad33a61ca44ee223704aeb71bba542592bd47cae6d97a622bbc3`다. 같은 파일로 frozen 수정 전2실행1PASS/1FAIL → 수정 후2PASS를 확인했다. 실패는 반쪽 간격에서 발생한다. helper 반환값만 검사한 결과로 대신하지 않는다.
- 기존 관련3case19PASS: #6776의2건, #7235의2건, #7482의15건. 보완한 현재 자동 수집 경로 `regression_suite_014::issue_6776_cell_tac_picture_accounting`도 실제2PASS/258skip,1.239초이며 컴파일18.96초 포함 exit0이다. 기존19건과 재실행2건을 고유21건으로 합산하지 않는다.
- `--prepare`:1684sources/7410static attrs/28suites+20exceptions=48targets. Source 길이 변경으로 자동 배정이002→014로 바뀌었고 실제 현재 경로를 실행했다. 생성 suite/manifest는 stage하지 않는다. fmt와 diff check도 확인한다.

법령 양성 대조는 수정 전후 동일하게 원시 layout63쪽14.6px overflow 경고를 낸다. 실제 Print profile/고정 base와의 분류를 전체 검증 잔여에 남긴다. 그림의 종이 안 포함 검사 PASS로 해당 본문의 실제 paint 완료를 주장하지 않는다.

## 전쪽 TSV와 직접 판독

- [Native 수도시설3쪽](native-water-all3.tsv), [fresh WASM 수도시설3쪽](wasm-water-all3.tsv)
- [Native RowBreak18쪽](native-rowbreak-all18.tsv), [fresh WASM RowBreak18쪽](wasm-rowbreak-all18.tsv)

|출력|review|overlay|
|---|---|---|
|수정 전Native3쪽|[review](native-p003-before-review.png)|[overlay](native-p003-before-overlay.png)|
|수정 후Native3쪽|[review](native-p003-review.png)|[overlay](native-p003-overlay.png)|
|수정 후fresh WASM3쪽|[review](wasm-p003-review.png)|[overlay](wasm-p003-overlay.png)|

같은 배율에서 표 외곽·표 뒤 빈 줄·제목·본문·붙임 목록을 직접 판독했다. 후속 본문의7.01333px 이동을 복구했으며 표 안 내용과 앞뒤 소속을 보존한다. raw100%는2px 이웃 실루엣 지표이며 엄격 RGB 픽셀 완전 일치가 아니다. 작은 글리프 프린지와 점선 테두리 차이는 남는다. 초기 복사 드라이버의v6 표기는 이전 경로에 보존하고, 제출 패널은 원본 PNG 해시를 유지한 채 현재v9 캡션으로 정본 compositor에서 다시 만들었다. TSV와 producer는 새V9 실제 출력이다.

## 남은 범위

이번 단계는 수도시설 회귀의 원인 보정이다. sample16 WMF18쪽과 편람·언어·교육·법령 등의 품질 하한 및 고정 base 하락, 전체 필수 nextest/lint/build/Skia/manifest/unit tiers/Actual36 검증은 계속한다. #8064 Native179조건의V6 실행은 발견용 이력이며 새 후보 전체 통과로 재인용하지 않는다. 전체 PR 준비 완료가 아니며 원격 push/PR 생성은 하지 않았다.

로그·중간 JSON·진단 source·미수용V7/V8 증거는 ignored `output/pr-review/7445-completed-20261009/rebase-80968497-20261021/`에 보존했다. 원본·PDF·글꼴·다른 작업장·공유 target·지침은 변경하지 않았다.
