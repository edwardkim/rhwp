# Task M100 #658 단계 4 완료보고서

## 단계명

통합 검증, 시각 판정, 최종 보고

## 작업 범위

단계 2의 native selection rect 정합화와 단계 3의 `rhwp-studio` 선택 하이라이트 렌더링 비용 완화를 통합 검증했다. 또한 작업지시자 직접 테스트 중 새로 확인된 드래그 중 커서/스크롤 위치 튐 증상은 #658과 별도 원인으로 판단하여 후속 이슈 #661로 분리했다.

## 검증 결과

### native rect 회귀 테스트

```bash
cargo test --test issue_658_text_selection_rects
```

결과:

- 2개 통과
- 실패 0개

검증 의미:

- `samples/exam_social.hwp` 오른쪽 자료 박스 셀 내부 선택 rect가 페이지 폭을 넘지 않는다.
- 줄 경계 offset에서 다음 줄 y 좌표를 이전 줄에서 잘못 재사용하지 않는다.
- 본문 다중 줄 선택도 같은 경계 모호성 회귀가 없다.

### 전체 Rust 회귀 테스트

```bash
cargo test --lib --release
```

결과:

- 1141개 통과
- 실패 0개
- ignored 2개
- 기존 warning 5개 유지

기존 warning은 이번 작업과 무관한 테스트/네이밍 경고이다.

### rhwp-studio 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

비고:

- 기존 Vite chunk-size warning 유지
- WASM 번들: `dist/assets/rhwp_bg-DktPFjjh.wasm`
- JS 번들: `dist/assets/index-DXsqB3Kv.js`

### native 진단 예제

```bash
cargo run --example inspect_658_selection
```

결과: 관찰 대상 selection rect 모두 `overflow_count=0`

주요 결과:

```text
--- data table p16 c0 paragraph 0 ---
overflow_count=0

--- data table p16 c0 paragraphs 0..6 ---
overflow_count=0

--- data table p16 c0 lower dialog ---
overflow_count=0
```

오른쪽 자료 박스 전체 선택 대표 범위:

```text
#00 p=1 x=558.4 y=223.8 w=398.7 h=12.7 right=957.1
#17 p=1 x=589.7 y=481.8 w=176.4 h=12.7 right=766.1
```

페이지 폭은 `1028.0px`이며 모든 rect가 페이지 안에 머문다.

### 웹 선택 레이어 계측

커밋하지 않는 임시 Puppeteer 스크립트(`/private/tmp/rhwp_658_selection_drag_check.mjs`)로 새 WASM과 `rhwp-studio` dev server 기준 선택 레이어를 계측했다.

환경:

- URL: `http://127.0.0.1:7701/`
- 샘플: `samples/exam_social.hwp`
- 대상: 2/4쪽 오른쪽 자료 박스

결과:

```json
{
  "fullRectCount": 18,
  "shortRectCount": 3,
  "fullRectBounds": {
    "minLeft": 558.4,
    "minTop": 223.8,
    "maxRight": 956.5999999999999,
    "maxBottom": 494.5
  },
  "shortRectBounds": {
    "minLeft": 558.4,
    "minTop": 223.8,
    "maxRight": 956.5999999999999,
    "maxBottom": 266.9
  },
  "afterFull": { "visible": 18, "total": 18 },
  "afterShort": { "visible": 3, "total": 18 },
  "afterSameShort": { "visible": 3, "total": 18 },
  "afterClear": { "visible": 0, "total": 18 }
}
```

의미:

- 전체 선택 rect의 오른쪽 끝이 `956.6px`로 페이지 폭 `1028.0px` 안에 있다.
- 선택 rect 수가 18개에서 3개로 줄어도 DOM 노드는 재사용되고 visible 수만 줄어든다.
- 동일 선택 상태 반복 갱신에서 하이라이트 div가 증가하지 않는다.
- 선택 해제 시 visible 하이라이트는 0개가 된다.
- 계측 중 console warn/error는 없었다.

## 작업지시자 시각 판정

작업지시자가 새 WASM 기준 로컬 웹에서 직접 테스트했고, 다음 판정을 제공했다.

```text
드래그가 텍스트를 넘어가는 문제는 해결되었어.
```

이는 #658의 핵심 완료 기준인 “선택 하이라이트가 텍스트/페이지 영역 밖으로 튀지 않음”에 대한 직접 시각 판정으로 본다.

## 후속 분리 이슈

직접 테스트 과정에서 “특정 상황에서 드래그를 하려 할 때 커서 및 페이지 스크롤 위치가 튀는” 별도 증상이 확인되었다.

분석 결과, 이 증상은 selection rect 오버런이 아니라 다음 영역과 관련된 별도 문제로 판단했다.

- 드래그 중 rAF 내부에서 원본 `MouseEvent`를 뒤늦게 재해석하는 경로
- 드래그 중 `scrollCaretIntoView()`가 caret 기준 자동 스크롤을 매 프레임 수행하는 경로
- `CursorState.updateRect()`의 `hitTest` pageIndex와 WASM `getCursorRect*` pageIndex 불일치 폴백
- Rust `getCursorRect*`가 현재 page hint 없이 후보 페이지를 앞에서부터 탐색하는 경로

이에 따라 새 이슈를 등록했다.

- #661: `rhwp-studio: 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상`

## 결론

#658 범위의 선택 하이라이트 rect 오버런과 드래그 중 DOM churn 문제는 수정 및 검증을 완료했다.

커서/스크롤 위치 튐은 별도 후속 이슈 #661로 분리했으며, #658 PR에는 포함하지 않는 것이 적절하다.
