# Stage 5 — 다크모드 툴바 검정 아이콘 가시성 정정

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16
- 선행 커밋: `6547ca40` (`task 1158: 눈금자 다크톤 정정과 시각 검토`)

## 1. 문제

다크모드에서 상단 툴바와 메뉴 드롭다운의 스프라이트 아이콘이 검정 바탕에 그대로 검정색으로 보여
가시성이 크게 떨어진다.

대표 예:

- 오려두기
- 복사하기
- 붙이기
- 표 / 도형 / 하이퍼링크
- 머리말 / 꼬리말 / 각주 / 미주
- 파일 메뉴의 저장 / 편집 용지 / 인쇄 / 제품 정보

## 2. 현재 판단

- 검정 단색만 CSS filter로 밝히는 방식은 일부 해결에는 되지만, 색상이 섞인 아이콘의 검정 외곽선까지
  함께 보정하기 어렵다.
- 메뉴 드롭다운의 `md-icon`도 같은 스프라이트 자산을 공유하므로 툴바만 따로 손보면 빠지는 아이콘이 생긴다.
- 따라서 dark theme에서는 `icon_small_ko.svg` 대신 검정 성분을 연한 회색으로 치환한
  `icon_small_ko_dark.svg`를 공통 스프라이트로 교체하는 방식이 더 안정적이다.

## 3. 예정 수정

- `rhwp-studio/src/styles/base.css`
- `rhwp-studio/src/styles/menu-bar.css`
- `rhwp-studio/src/styles/toolbar.css`
- `rhwp-studio/public/images/icon_small_ko_dark.svg`

## 4. 예정 검증

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

시각 확인:

- `http://localhost:7700/?url=/samples/para-001.hwp&filename=para-001.hwp`
- dark mode에서 상단 툴바 아이콘의 검정 성분이 구글 독스처럼 연한 회색 톤으로 보이는지 확인
- dark mode에서 `파일` 메뉴를 열었을 때 메뉴 아이콘도 같은 톤으로 보이는지 확인
- mobile viewport에서도 같은 톤과 대비가 유지되는지 확인

## 5. 실행 결과

- dark 전용 스프라이트 `icon_small_ko_dark.svg`를 생성했다.
- `base.css`에 공통 sprite URL 토큰을 추가하고, dark theme에서 다크 전용 스프라이트를 사용하도록 연결했다.
- `toolbar.css`, `menu-bar.css`가 같은 sprite URL 토큰을 사용하도록 정리했다.
- `npm run build` 통과
- `node e2e/theme-mode.test.mjs --mode=headless` 통과
- 로컬 시각 확인 자료:
  - `/tmp/rhwp-dark-toolbar-top.png`
  - `/tmp/rhwp-dark-menu-file.png`
  - `mydocs/report/assets/task_m100_1158_dark_mobile_main.png`
  - `mydocs/report/assets/task_m100_1158_dark_mobile_file_menu.png`
- Chrome headless 모바일 캡처에서 dark chrome과 파일 메뉴 아이콘 가시성을 다시 확인했다.

## 6. 작업지시자 확인 대기

- desktop/mobile 시각 자료 반영 후 PR 준비 진행
