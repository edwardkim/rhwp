# PR #1424 처리 계획 - rhwp-studio 다크모드 잔여 UI 대비 정리

## 1. 대상

- PR: #1424
- 이슈: #1422
- base: `devel`
- head: `edwardkim:task_m100_1422`
- 현재 head: `359fbd3f` (리뷰 문서 커밋 전 기준)
- 처리 방식: GitHub PR merge
- merge 수행: 작업지시자가 GitHub merge 버튼으로 직접 수행

## 2. 커밋 구성

현재 PR은 다음 묶음으로 구성한다.

1. Stage 1: 공통 dialog/control dark token 정리
2. Stage 2: 표/셀 속성 및 셀 테두리/배경 dark token 정리
3. Stage 3: 수식 편집 및 쪽 테두리/배경 dark token 정리
4. Stage 3 보정: 쪽 테두리 preview guide 대비 보정
5. Stage 4: table quick grid, endnote, para preview, bullet popup, validation, grid sweep
6. Stage 5: dialog theme focused regression guard 추가
7. Stage 6: Chrome Auto Dark Mode에서 명시적 light/dark 의도 보존
8. Stage 7: 저장된 dark 테마를 stylesheet 전 bootstrap으로 선반영
9. Final report: #1422 최종 보고서와 주문서 갱신

이번 문서 커밋은 위 변경 위에 PR 운영 문서를 archive 경로에 동반하는 단계다.

## 3. 검증 전략

로컬 필수 검증은 PR 준비 단계에서 완료했다.

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-bootstrap.test.mjs --mode=headless`
- `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless`
- `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/dialog-theme.test.mjs --mode=headless`
- `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' CHROME_EXTRA_ARGS='--enable-features=WebContentsForceDark' node e2e/theme-auto-dark.test.mjs --mode=headless`
- `git diff --check upstream/devel...HEAD`

문서 커밋 단계에서는 다음만 추가 확인한다.

- `git diff --check`
- 변경 파일 범위가 `mydocs/` 문서인지 확인
- PR 본문 screenshot markdown 보존 확인
- GitHub Actions 재실행 상태 확인

## 4. GitHub 처리 순서

1. `mydocs/pr/archives/pr_1424_review.md` 작성
2. `mydocs/pr/archives/pr_1424_review_impl.md` 작성
3. `mydocs/orders/20260617.md`에 PR #1424 리뷰 문서 추가 상태 기록
4. `mydocs/report/task_m100_1422_report.md`의 변경 파일 목록 정합 보정
5. 문서 커밋을 `task_m100_1422` PR head에 push
6. PR 본문을 실제 리뷰 문서명과 stage별 요약으로 갱신하고, 작업지시자가 추가한 before/after 스크린샷을 보존
7. GitHub Actions 재실행 완료 대기
8. checks 통과 확인 후 draft를 ready 상태로 전환
9. 작업지시자가 GitHub merge 버튼으로 PR #1424 merge
10. #1422 auto close 여부 확인
11. `upstream/devel` 동기화

## 5. merge 후 추가 merge 방지 방침

#1420 선례처럼 PR 운영 문서는 merge 전에 PR diff에 포함한다. 따라서 merge 후에는 별도 저장소 문서 커밋을 만들지 않는다.

merge 후 필요한 작업은 저장소 변경 없이 다음 상태 확인으로 제한한다.

- PR #1424 merged 상태 확인
- #1422 closed 상태 확인
- 로컬 `devel`/`local/devel` 동기화
- 필요 시 작업지시자에게 GitHub 상태 요약 보고

별도 `mydocs/pr/pr_1424_report.md`를 새로 만들지 않는다. merge 판단과 처리 기록은 다음 문서로 충분히 남긴다.

- `mydocs/pr/archives/pr_1424_review.md`
- `mydocs/pr/archives/pr_1424_review_impl.md`
- `mydocs/report/task_m100_1422_report.md`

## 6. 후속 분리

이번 PR에 포함하지 않는 항목:

- Chrome Auto Dark Mode 실험 기능의 브라우저 버전별 전수 검증
- dialog-theme runner cleanup 지연 원인 별도 분석
- 다크모드 팔레트 전면 재설계
- 문서 본문/편집 종이/문서 렌더링 색상 반전

## 7. 현재 판정

PR 본문, 시각 자료, 로컬 검증, GitHub Actions 1차 통과 상태는 준비되었다.
문서 커밋과 PR 본문 갱신 후 GitHub Actions가 다시 통과하면 작업지시자 merge 진행 가능 상태로 본다.
