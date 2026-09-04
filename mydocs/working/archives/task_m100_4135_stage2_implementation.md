# M100 #4135 Stage 2 — 문맥 라우팅 구현 및 브라우저 GREEN

## 구현

`Ctrl/Cmd+Shift+S` 전역 후보 목록을 넓히지 않고 F5 셀 블록에 한정한 순수 라우터를
추가했다.

- full + 셀 블록 + 블록 합계 활성: `table:block-sum`
- full + 셀 블록 + 블록 합계 비활성: `file:save-as`
- embed처럼 Save As 미등록: 다른 명령으로 폴스루하지 않고 이벤트만 소비
- 셀 블록 밖: 기존 `shortcut-map`에 양보

키보드 입력 처리에서는 이 라우터를 IME 조기 반환과 셀 선택 모드의 `M`/`S` 처리보다
먼저 실행한다. 수정자 없는 셀 합치기/나누기 분기에는 Ctrl/Meta/Alt 가드를 추가했다.
일반 단축키 맵은 Save As의 물리 `KeyS`를 인식하도록 보강하고 도달 불가였던
`table:block-sum` 중복 항목을 제거했다.

## 집중 회귀

실행:

```bash
cd rhwp-studio
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/shortcut-map.test.ts \
  tests/chrome-mode.test.ts
```

결과: 31 pass, 0 fail.

확인 범위:

- Ctrl/Command 영문 키
- 한글 자모 `ㄴ`과 IME `Process/KeyS`
- 셀 블록 밖 Save As 양보
- modifier 없는 `S`
- embed consume-only 계약
- 단축키 맵 exact 슬롯 중복 부재

## 실제 브라우저 GREEN

최신 모듈을 새로 로드한 Vite 개발 서버(`http://127.0.0.1:7715/`)에서 새 문서를 열고
2행 3열 표를 만들었다. `F5`, `F5`, `ArrowRight`로 복수 셀 블록을 만든 뒤 확인했다.

| 입력 | 관찰 |
| --- | --- |
| 셀 블록 + `Ctrl+Shift+S` | 셀 나누기/Save As 대화상자 없이 블록 계산 실행 |
| 셀 블록 + `Cmd+Shift+S` | 셀 나누기/Save As 대화상자 없이 블록 계산 실행 |
| `Escape` 후 `Ctrl+Shift+S` | 다른 이름으로 저장 대화상자 표시 |
| 셀 블록 + 수정자 없는 `S` | 셀 나누기 대화상자 표시 |

## embed 회귀

설치된 Chrome 실행 경로와 이미 실행 중인 Vite URL을 명시해 기존 iframe 전송 E2E를
실행했다.

```bash
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
VITE_URL='http://127.0.0.1:7715' \
npm run e2e:embed
```

결과: 공개 load/export/diagnostics, forged peer 차단, destroy, legacy 경로를 포함한
17개 판정 모두 PASS.

## 범위 메모

기존 `table:block-sum` 구현은 현재 셀에 `=SUM(above)`를 적용한다. 이번 작업은 #4135의
도달 불가 단축키 라우팅만 고쳤으며, 선택 범위 자체를 계산식 피연산자로 바꾸는 동작은
포함하지 않았다.
