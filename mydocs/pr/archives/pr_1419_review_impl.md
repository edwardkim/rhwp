# PR #1419 처리 계획 - 누름틀 양식 모드

## 1. 대상

- PR: #1419
- 이슈: #258
- base: `devel`
- head: `jangster77:task_m100_258`
- 초기 head: `a063ad49`
- 문서 커밋: 현재 PR head의 리뷰 문서 커밋

## 2. 커밋 구성

PR은 Task #258 단계별 커밋 30개와 PR 처리 문서 커밋 1개로 구성한다.

주요 구간:

1. Stage1~5: 누름틀/양식 모드 조사, MVP, 삽입 대화상자, 저장 라운드트립
2. Stage6~15: modal 동작, 안내문 표시, 입력/삭제/렌더 갱신
3. Stage16~24: 기존 샘플, 삭제 확인, 복사, 방향키 경계 이동
4. Stage25~29: 인접 누름틀 선택, 선택 색상, 붙여넣기, Home/End 보정
5. Stage30: PR 준비 검증 기록
6. PR 처리 문서: 리뷰 문서와 오늘할일 기록

## 3. 검증 전략

로컬 필수 검증은 PR 준비 단계에서 이미 완료했다.

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`
- `git diff --check`

작업지시자 지시에 따라 문서 커밋 시점에는 로컬 테스트를 재실행하지 않는다. 문서 변경 확인은 `git diff --check`와 변경 파일 범위 확인으로 수행한다.

## 4. GitHub 처리 순서

1. `task_m100_258` 브랜치를 `origin`에 push
2. `edwardkim/rhwp`의 `devel` 대상으로 PR #1419 생성
3. PR 리뷰 문서와 오늘할일 문서를 작성
4. 문서 커밋을 PR head에 push
5. PR diff에 문서 3건 포함 확인
6. GitHub Actions 재실행 완료 대기
7. required checks 통과 시 merge
8. #258 close 확인
9. `upstream/devel` 동기화
10. 리뷰 문서 archive 이동 후 후속 문서 정리

## 5. 후속 분리

이번 PR에서 제외한 항목:

- 사용자 정보, 문서 요약, 작성한 날짜, 파일 이름/경로 필드 탭
- Edit/CheckBox/RadioButton/ComboBox/PushButton 등 양식 개체 전체 상호작용
- 한컴 전체 양식 도구 UI와 스크립트 연동

## 6. 현재 판정

작업지시자 시각 검증과 Stage30 로컬 필수 검증이 완료되었으므로, 문서 커밋 후 GitHub Actions가 통과하면 merge 진행 가능하다.
