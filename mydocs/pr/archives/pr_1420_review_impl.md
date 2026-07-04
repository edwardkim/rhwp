# PR #1420 처리 계획 - rhwp-studio 다크테마 지원

## 1. 대상

- PR: #1420
- 이슈: #1158
- base: `devel`
- head: `jangster77:task_m100_1158`
- 현재 head: `667c47db`

## 2. 커밋 구성

현재 PR은 다음 묶음으로 구성한다.

1. Stage 1: 테마 설정 저장/적용 기반 추가
2. Stage 2: 다이얼로그 dark token 정리
3. Stage 3: 선택 오버레이 정리와 보고서 보강
4. Stage 4: 눈금자 dark tone 정정과 시각 검토
5. Stage 5: dark icon sprite 교체, desktop/mobile 시각 자료 확보
6. PR #1419 후속 문서 archive 정리

이번 문서 커밋은 위 변경 위에 PR 운영 문서를 archive 경로에 동반하는 단계다.

## 3. 검증 전략

로컬 필수 검증은 PR 준비 단계에서 이미 완료했다.

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`

문서 커밋 단계에서는 다음만 추가 확인한다.

- `git diff --check`
- 변경 파일 범위가 `mydocs/` 문서인지 확인

## 4. GitHub 처리 순서

1. `task_m100_1158` 브랜치를 `origin`에 push
2. `edwardkim/rhwp`의 `devel` 대상으로 PR #1420 생성
3. PR 리뷰 문서를 `mydocs/pr/archives/`에 직접 작성
4. 오늘할일 문서에 PR #1420 처리 항목 추가
5. 문서 커밋을 PR head에 push
6. PR diff에 archive 문서와 오늘할일 포함 확인
7. GitHub Actions 재실행 완료 대기
8. required checks 통과 시 merge
9. #1158 close 확인
10. `upstream/devel` 동기화

## 5. 후속 분리

이번 PR에 포함하지 않는 항목:

- dark mode 추가 세부 팔레트 미세조정
- 브라우저/OS별 시스템 테마 차이 대응
- mobile 터치 인터랙션 전수 검증 자동화

## 6. 현재 판정

PR 본문, 시각 자료, 로컬 필수 검증은 준비되었다. 문서 커밋을 push한 뒤 GitHub Actions가 통과하면
merge 진행 가능 상태로 본다.
