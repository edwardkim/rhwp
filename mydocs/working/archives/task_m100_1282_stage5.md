# Task 1282 Stage 5 — PR 시각 검증 자료 보강

## 목적

Stage 3/4의 수치형 E2E 검증은 통과했지만, PR 본문에서 바로 확인할 수 있는 화면 증적이 부족했다.
이번 단계에서는 `samples/ta-pic-001-r.hwp`를 실제 rhwp-studio 화면으로 열고, 회전된 표 셀 그림을
리사이즈하기 전/후를 headless Chrome으로 캡처해 PR 검토 자료로 남긴다.

## 검증 흐름

- 대상 URL: `http://localhost:7700/`
- 대상 문서: `samples/ta-pic-001-r.hwp`
- 대상 객체: 첫 페이지 표 안 회전 picture
  - `cellPath=[{"controlIndex":2,"cellIndex":2,"cellParaIndex":0}]`
  - `paraIdx=0`, `controlIdx=0`
- 조작: `se` 리사이즈 핸들을 오른쪽 아래로 드래그

## Browser 확인

`build-web-apps:frontend-testing-debugging` 지침에 따라 Browser plugin 경로를 먼저 시도했다.

- Page identity: `http://localhost:7700/`, title `rhwp-studio`
- 앱 chrome DOM: 파일/편집/입력/서식 메뉴 확인
- console error/warn: 없음
- Browser screenshot: `Page.captureScreenshot` timeout으로 완료하지 못함

따라서 기존 rhwp-studio headless Chrome/Puppeteer 검증 경로로 fallback했다.

## Headless 캡처 결과

생성 파일:

- 전체 before: `mydocs/report/assets/task_m100_1282_resize_before.png`
- 전체 after: `mydocs/report/assets/task_m100_1282_resize_after.png`
- before crop: `mydocs/report/assets/task_m100_1282_resize_before_crop.png`
- after crop: `mydocs/report/assets/task_m100_1282_resize_after_crop.png`

수치 확인:

```text
picture height: 18160 -> 18712
owner cell height: 17476 -> 20367
required owner cell height after resize: 20367
```

판정:

- 회전된 picture의 선택 bbox가 드래그 후 실제로 커진다.
- owner cell height가 `picture.vertOffset + picture.height + padding` 이상으로 증가한다.
- 시각 crop에서 리사이즈 후 picture가 사라지거나 화면 bbox와 실제 그림이 분리되는 현상은 보이지 않는다.
- PR 전 필수 전체 검증과 clippy는 아직 별도 수행이 필요하다.
