## 요약

- rhwp-studio에 `system | light | dark` 테마 설정과 저장/복원 흐름을 추가했습니다.
- 메뉴바, 툴바, 서식바, 상태바, 작업영역, 주요 다이얼로그를 semantic token 기반으로 정리했습니다.
- dark mode에서 검정 스프라이트 아이콘이 묻히던 문제를 dark 전용 스프라이트 교체 방식으로 정리했습니다.
- 편집 용지는 계속 흰색으로 유지하고, 눈금자와 앱 chrome만 dark palette를 적용했습니다.
- 추가로 PR #1419 처리 후 남아 있던 리뷰 문서/오늘할일 기록을 archive 경로로 후속 정리했습니다.

## 검증

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `git diff --check`

## 시각 자료

### 데스크톱

![다크 툴바](https://raw.githubusercontent.com/jangster77/rhwp/task_m100_1158/mydocs/report/assets/task_m100_1158_dark_toolbar_top.png)
![다크 파일 메뉴](https://raw.githubusercontent.com/jangster77/rhwp/task_m100_1158/mydocs/report/assets/task_m100_1158_dark_menu_file.png)

### 모바일

![다크 모바일 메인 화면](https://raw.githubusercontent.com/jangster77/rhwp/task_m100_1158/mydocs/report/assets/task_m100_1158_dark_mobile_main.png)
![다크 모바일 파일 메뉴](https://raw.githubusercontent.com/jangster77/rhwp/task_m100_1158/mydocs/report/assets/task_m100_1158_dark_mobile_file_menu.png)

Closes #1158
