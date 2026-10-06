# Stage 5 완료 보고서 — Task M100-1422

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 브랜치: `local/task1422`
- 단계: Stage 5 — focused 회귀 가드 보강
- 완료 시각: 2026-06-17 02:09

## 1. 작업 요약

Stage 1~4에서 정리한 다크모드 색상 정책을 자동 회귀 테스트로 고정했다.
기존 `theme-mode.test.mjs`는 유지하고, 다이얼로그와 팝업의 핵심 computed style을 확인하는
`dialog-theme.test.mjs`를 신규 추가했다.

## 2. 신규 테스트 범위

- `view:grid-settings`
  - `.dialog-input` 배경/글자/테두리가 dark UI token 계열인지 확인
  - fieldset/legend/보조 버튼 대비 확인
- `insert:equation`
  - `.eq-preview` 배경은 문서 종이 흰색 유지
  - 스크립트 입력과 글자 크기 입력은 dark UI surface 유지
- `page:page-border`
  - 중앙 SVG와 내부 rect fill은 문서 종이 흰색 유지
  - preview guide stroke는 `#d0d0d0` 유지
  - 사방 버튼 배경/기호/테두리는 dark UI token 계열
  - dark에서도 사방 버튼 클릭 시 preview 선이 추가되는지 확인
- `format:para-shape`
  - 문단 preview는 흰 문서 배경과 검정 문서 텍스트 유지
  - 입력 필드와 active 정렬 버튼은 dark UI token 계열
- toolbar popup
  - 글머리표 popup surface/cell 색상 확인
  - 표 만들기 quick grid popup/cell/취소 버튼 색상 확인

## 3. 수정 파일

- `rhwp-studio/e2e/dialog-theme.test.mjs`
- `mydocs/orders/20260617.md`

## 4. 검증 결과

```bash
cd rhwp-studio && npm run build
```

- 통과

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7701 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

- 통과

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7701 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/dialog-theme.test.mjs --mode=headless
```

- 통과

## 5. 잔여 작업

- Stage 6에서 Chrome `Auto Dark Mode for Web Contents` 활성화 환경의 `보기 > 테마 > 밝게` 동작을 최종 점검한다.
- 이번 단계는 테스트 보강만 수행했으며 UI 구현 변경은 포함하지 않았다.
