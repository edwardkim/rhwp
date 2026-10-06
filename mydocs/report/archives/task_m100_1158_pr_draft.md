# Task M100-1158 PR 초안

- 이슈: [#1158](https://github.com/edwardkim/rhwp/issues/1158)
- 로컬 브랜치: `local/task_m100_1158`
- 원격 브랜치 예정: `task_m100_1158`
- 작성일: 2026-06-16

## 1. PR 제목 후보

```text
rhwp-studio 다크테마 지원
```

## 2. PR 본문 초안

````markdown
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
````

## 3. PR 생성 전 확인 사항

1. 원격 브랜치는 `task_m100_1158` 이름으로 push해서 이미지 raw URL 경로와 맞춘다.
2. PR 생성 대상 저장소는 `edwardkim/rhwp`, base branch는 `devel`이다.
3. PR 생성 시 위 본문을 그대로 사용하고, 이미지가 렌더링되는지 PR 미리보기에서 한 번 더 확인한다.
