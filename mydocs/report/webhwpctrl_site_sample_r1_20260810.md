# 웹한글컨트롤 실사이트 표본 r1 — GitHub 전수 스윕과 첫 호출 프로파일

- 날짜: 2026-08-10
- 목적: [계획서 §6.3.3 U0](../plans/hwpctrl_ocx_full_compat.md)의 "대상 사이트 표본 채집" 1차 실행.
  "완성"의 실질 기준은 원장 484 전 항목이 아니라 **실사이트가 부르는 부분집합**이다 — 그
  부분집합의 첫 실측이다.
- 결론 요약: **채집은 가능하다.** 실표본 3건 + 한컴 SDK 원본 1건을 확보했고, 첫 호출
  프로파일은 **필드 왕복이 압도적**(기안기 패턴)이며 대부분 이미 `verified` 다. UI 면
  (`Show*` 3종)과 '관측창 없음' 갈래(`TableResizeCellLeft`)의 **실수요가 표본으로 확인**됐다.
  실물 컨트롤의 임베드 메커니즘은 **same-origin iframe + 동기 Impl 참조**로 실측됐다 —
  채널 ② 설계를 직접 뒷받침한다.

## 1. 방법

GitHub code search 로 지문 10종을 전수 스윕했다(2026-08-10, `gh api search/code`).

지문: `"webhwpctrl.js"` · `"hwpCtrlApp.js"` · `"hwpctrlframe.html"` · `"hwpctrlmain.html"` ·
`"hwpCtrlIntf"` · `"hwpCtrlImpl"` · `"PutFieldText" "MoveToField"` · `"GetFieldText"
"PutFieldText"` · `"HwpCtrl" "InsertFieldTemplate"` · `"HwpCtrl.ocx" classid`

원시 117매치 → 중복 제거 후 저장소 23개. rhwp 계열(본체·포크·미러) 6개를 빼고 17개를 분류했다.

**표본 취급 규칙** — 표본 코드는 이 저장소에 들이지 않는다. 좌표(저장소/경로)와 **API 호출
집계만** 기록한다. 특히 아래 표본 1은 내부 시스템 소스가 공개된 것으로 보이므로 코드 인용을
하지 않는다.

## 2. 표본 분류

### 웹한글컨트롤 실표본 (3 + SDK 원본 1)

| # | 좌표 | 성격 |
|---|---|---|
| 1 | `cysong98/s-systemAPI` — `S-system-FE/ssystem/src/components/dct/HwpCtrl.vue`(970줄)·`HwpCtrlPopup.vue` | **공공기관(KINAC 계열) 전자문서 시스템 프론트엔드.** 한컴 SDK 셸을 싣고 앱 코드가 컨트롤을 실제 스크립팅 |
| 2 | `letsbe-x/hwpViewer` — `js/webhwpctrl.js`(26KB)·`js/hwpCtrlApp.js`(93줄) | **한컴 SDK 원본.** `HwpCtrl` 파사드(메서드 96 + 속성 24 = 120 — 우리 스펙의 API 122 와 맞물린다) + 임베드 부트스트랩 |
| 3 | `chrisryugj/edoc-summarizer` — `background.js` | **살아 있는 기안기 페이지에 주입하는 Chrome 확장.** `window.HwpCtrl`/`WebHwpCtrl` 전역과 iframe `contentWindow` 를 뒤져 컨트롤을 잡고 `GetTextFile` 을 부른다 — **채널 ②(확장 드롭인) 발상의 독립 선례** |

### COM 자동화 군집 (11) — 다른 축의 수요 신호

`JSJeong-me/Python_RPA`·`JunDamin/hwpapi`·`YongJun-Lee-98/cpyhwpx`·`twoLoop-40/HwpAutomation`
·`zarathucorp/openhwpsdk` 등. 웹 컨트롤이 아니라 데스크톱 COM(`HWPFrame.HwpObject`) 자동화다.
필드 채우기·Action 실행이라는 **수요의 모양은 같지만** 이 캠페인의 대조 축이 아니므로 좌표만
남긴다. 파서·SDK 프로젝트(`disjukr/hwpkit`·`b612nightsky/gohwp`)도 여기 묶는다.

## 3. 호출 프로파일 — 표본 1 (실 스크립팅 코드)

`HwpCtrl.vue` + `HwpCtrlPopup.vue` 의 호출 집계:

| API | 호출 수 | 원장 상태 |
|---|---:|---|
| `Run` (액션 9종) | 32 | 액션별 아래 참조 |
| `GetFieldText` | 30 | verified |
| `PutFieldText` | 26 | verified |
| `MoveToField` | 19 | verified |
| `EditMode` | 15 | verified |
| `CreateAction` | 9 | verified |
| `IsModified` | 7 | substituted |
| `Open` | 6 | verified |
| `GetFieldList` | 3 | verified |
| `InsertPicture` | 3 | verified |
| `ShowToolBar` / `ShowStatusBar` / `ShowRibbon` | 각 3 | **unimplemented — UI 면** |
| `SaveAs` | 1 | verified |
| `Clear` | 1 | verified |

`Run` 액션 분해: `Delete` 12 · `TableRightCell` 8 · `Cancel` 3 · `TableLeftCell` 2 ·
`TableCellBlockExtend` 2 · `TableCellBlock` 2 · `TableUpperCell` 1 · `TableCellBlockCol` 1 ·
**`TableResizeCellLeft` 1**. 원장 대조 결과 아홉 중 여덟이 `verified` 고 `TableResizeCellLeft`
하나만 `unimplemented`(관측창 없음)다.

표본 3 은 `GetTextFile`(verified) 하나를 부른다 — 읽기 전용 소비자.

**합산하면** — 표본 1 의 호출 표면은 API 15종 + 액션 9종 = 24종이고, 그중 `verified` 20 ·
`substituted` 1(`IsModified`) · `unimplemented` **4**(`Show*` 3종 + `TableResizeCellLeft`)다.
즉 이 실사이트 하나를 기준으로 하면 **호환 층은 이미 24분의 20**이고, 남은 넷 중 셋이 §6.3
U1 의 UI 면이다.

### 읽는 법

1. **기안기 패턴이 그대로 나온다.** 필드 왕복(`MoveToField`→`GetFieldText`/`PutFieldText`) +
   표 셀 이동·블록 — 계획서 P1~P2 를 "실사용 전환점"으로 본 판단과 정확히 일치하고, 그 축은
   이미 `verified` 다.
2. **UI 면의 실수요가 확인됐다.** `Show*` 3종을 실코드가 부른다(기안기를 문서 영역만 남기고
   임베드하는 용도). §6.3 U1 의 가치가 표본으로 뒷받침된다.
3. **'관측창 없음'에도 실수요가 있다.** `TableResizeCellLeft` 는 COM 오라클로 관측창이 없어
   막아 둔 갈래인데 실사이트가 부른다. 갈래 판정(못 잰다)과 수요 판정(안 쓰인다)은 다른
   문제라는 증거 — 이런 항목은 `web-contract` 계층에서 다시 살필 후보다.
4. 표본 1건의 호출 종수는 **15종 + 액션 9종**이다. 484 의 5% 남짓으로 실사이트 하나가
   돌아간다. 표본이 몇 건 더 쌓이면 부분집합의 상한이 빠르게 수렴할 것이다.

## 4. 임베드 메커니즘 실측 — 채널 ② 에 주는 것

`hwpCtrlApp.js`(93줄)가 하는 일: 지정 요소 안에 **iframe** 을 만들어
`{웹한글서버}/hwpctrlmain.html` 을 싣고, `HwpCtrl` 파사드는 그 iframe 의
`hwpCtrlApp.hwpCtrlImpl` 을 **동기로** 참조한다(`hwpctrlframe.html` 의 `<base href>` 가
사이트 자체 웹한글 서버를 가리킨다 — 표본 1 실측).

즉 **실물 웹한글컨트롤도 iframe 이되 same-origin 이라 동기 API 가 성립**한다. 계획서
§6.3.2 의 동기 제약 분석과 정합하고, 채널 ② 에 선택지를 하나 더 준다: MAIN 월드 직주입
외에, **same-origin(srcdoc) iframe 으로 DOM 을 격리하면서 동기 참조를 유지**하는 형태도
실물과 같은 모양이다.

## 5. 웹한글 기안기 공식 자료

- [개발 가이드](https://developer.hancom.com/webhwp/devguide) — 8절: 구성·**한글 컨트롤
  (ActiveX) 대비 API 변경 사항**·기본 사용법·Action·CtrlCode·ParameterSet·ParameterArray·
  HwpCtrl. 마이그레이션 절은 우리 대체 계약표(§1.1)의 공식 대응물이다.
- [개요](https://developer.hancom.com/webhwp/overview) · API 개별 페이지
  (예: [PutFieldText](https://developer.hancom.com/webhwp/devguide/hwpctrl/methods/putfieldtext))
- [한컴디벨로퍼 포럼 — 한글 컨트롤 분류](https://forum.developer.hancom.com/) — 실사용자
  질문(버전별 이슈 포함)
- SDK 실물: 표본 2 (`webhwpctrl.js` 파사드 120 멤버)

## 6. 한계와 다음 일

- **인트라넷 미포섭.** 주력 배치(전자결재·기안기)는 로그인 뒤라 크롤링으로 못 본다.
  검색엔진은 페이지 JS 를 색인하지 않아 `site:go.kr` 직접 검색은 0건이었다(실측). GitHub
  표본은 유출·공개된 소수라 대표성이 제한적이다 — 다만 한컴 SDK 가 통합 형태를 강제하므로
  (모든 사이트가 같은 셸), 소수 표본의 API 프로파일이 일반화될 여지는 크다.
- 다음 일 셋:
  1. 지문을 넓혀 재스윕(`GetTextFile` 조합, `InsertFieldTemplate`, 포럼·블로그의 통합 예제)
     — 표본 5건 이상이 목표.
  2. 나라장터 RFP·과업지시서에서 "웹한글" 요구 시스템 목록 채집(간접 증거).
  3. 표본 프로파일 합집합을 원장에 대조해 **실수요 부분집합 표**를 원장 옆에 두고, U1 범위
     확정에 쓴다.
