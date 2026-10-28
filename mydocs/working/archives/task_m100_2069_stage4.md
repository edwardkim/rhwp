# Task M100 #2069 Stage 4: OLE 첫 줄 재진입 Enter 흐름 보정

## 목표

`samples/한셀OLE.hwp`와 `samples/한셀OLE.hwpx`에서 OLE 오른쪽 Enter/Backspace 후 다시 첫 줄에서 Enter를 입력했을 때 한컴과 같은 문단부호 위치와 OLE 배치를 유지한다.

## 사용자 확인 차이

- Stage 3 기준 연속 Enter로 OLE 오른쪽 영역을 채운 뒤 본문 왼쪽으로 내려가는 동작은 한컴과 같아졌다.
- Backspace로 추가 빈 문단을 모두 제거해 원래 상태로 돌아가는 동작도 한컴과 같아졌다.
- 다시 첫 줄에서 Enter를 누르면 rhwp는 새 문단부호를 OLE 왼쪽 위 흐름으로 배치하지만, 한컴은 OLE 어울림/문단부호 흐름을 유지한다.
- 한셀 OLE 미리보기는 표처럼 보이지만 편집 가능한 표가 아니므로, 클릭/이동으로 OLE 내부 셀처럼 커서가 들어가면 한컴과 다르다.

## 분석 방향

1. Backspace 후 커서가 OLE control 앞/뒤 중 어디에 남는지 확인한다.
2. 첫 줄 Enter가 OLE host 문단을 앞쪽에서 split하는지, OLE anchor 문단 뒤에 빈 문단을 추가하는지 확인한다.
3. OLE가 포함된 비 글자 square-wrap host 문단은 커서 offset 0이라도 한컴 기준으로 OLE 오른쪽 Enter 흐름을 우선해야 하는지 HWP/HWPX를 함께 비교한다.
4. Stage 3의 wrap-zone 높이 경계 로직이 Backspace 후 재진입 시에도 같은 기준으로 적용되는지 검증한다.
5. OLE bbox 클릭은 본문 `hitTest`보다 먼저 OLE 개체 선택으로 처리해 표 셀/텍스트 커서 진입 분기까지 내려가지 않게 한다.

## 검증 계획

- HWP/HWPX focused test에 Enter 반복, Backspace 반복, 첫 줄 재진입 Enter 시나리오를 추가한다.
- 기존 Stage 3 테스트가 그대로 통과하는지 확인한다.
- `wasm-pack build --target web --out-dir pkg` 후 기존 `localhost:7700`에서 재현 흐름을 확인한다.
- 기존 `localhost:7700`에서 OLE 내부 클릭이 셀/텍스트 커서가 아니라 OLE 개체 선택으로 처리되는지 확인한다.

## 진행 결과

- OLE bbox 클릭을 일반 본문 `hitTest`보다 먼저 OLE 개체 선택으로 처리하도록 Studio 마우스 경로를 보정했다.
- `samples/한셀OLE.hwp`를 기존 `localhost:7700`에서 열고 OLE 노란 영역 내부를 클릭했을 때 `selected.type=ole`, `isCell=false`로 확인했다.
- OLE host caret 기준 좌표는 `x=513.4`이고 OLE bbox 오른쪽(`x=113.4 + w=400`)과 일치한다.
- `rhwp-studio/e2e/issue-2069-ole-object-selection.test.mjs`를 추가해 OLE 내부 클릭이 표 셀 편집으로 진입하지 않는 조건을 회귀 테스트로 고정했다.
- 검증: `CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node rhwp-studio/e2e/issue-2069-ole-object-selection.test.mjs --mode=headless` 통과.
- 검증: `npm --prefix rhwp-studio run build` 통과.
