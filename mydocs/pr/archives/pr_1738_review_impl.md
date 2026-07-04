# PR #1738 처리 계획 — #1735 방점 dotted-circle 아티팩트 제거

## 대상

- PR: #1738
- 작성자: @johndoekim
- 관련 이슈: #1735
- 문서 작성 시점 PR head: `00dc17ef962b39e1248297e8c80e8dc7ddeb2da1`
- 처리 판단: admin merge 완료
- merge commit: `18720d08a63da6fb396a9ad1effbd01833cca300`

## 커밋

1. `4da146a3a4af2e8ca0443ff5e7fb3af3f304892f`
   - `Task #1735: 방점(U+302E/U+302F) 렌더 치환으로 dotted-circle 아티팩트 제거 (Stage 1)`
   - 렌더 치환 및 composer 테스트 추가
2. `d6cbe5c5265204beb574c9bb3ee37ce1ae6abf3b`
   - `Task #1735: 방점 측정 폭을 narrow(0.3em)로 정합 (Stage 2)`
   - 측정 폭 분류 및 text measurement 테스트 추가
3. `5dfc9a230d23938e13cc91bd36cd242940b678cf`
   - `Task #1735: 최종 결과보고서`
   - 결과 보고서 추가
4. `00bc0428173c9f413171f3b070c923089320d050`
   - `Task #1735: 방점 렌더 회귀 샘플 + 한컴 정답지 PDF 추가`
   - 샘플 HWP/PDF 추가
5. `00dc17ef962b39e1248297e8c80e8dc7ddeb2da1`
   - `Merge branch 'devel' into local/task1735`
   - 최신 `devel` 동기화용 merge commit

## 검토 단계

### Stage 1. PR 메타 확인

- base branch: `devel`
- draft: false (작성 시점 참고값)
- mergeable: `MERGEABLE` (작성 시점 참고값)
- mergeStateStatus: `BLOCKED` (GitHub required check 일부 진행 중인 작성 시점 참고값)
- maintainerCanModify: `true`
- 규모: 10 files, +258/-2

### Stage 2. 이전 SHA run 강제 취소

완료.

- 대상 이전 SHA: `00bc0428173c9f413171f3b070c923089320d050`
- CI run `28517498819`: force-cancel 후 `cancelled`
- CodeQL run `28517498979`: force-cancel 후 `cancelled`
- Render Diff run `28517498850`: `cancelled`
- `mydocs/manual/pr_review_workflow.md`에 update branch 이후 이전 SHA run은 처음부터 force-cancel API를 쓰도록 반영

### Stage 3. merge 시뮬레이션

완료.

- `upstream/devel` 기준 `local/pr1738` 병합 시뮬레이션
- 충돌 없음

### Stage 4. 변경 내용 검토

완료.

- 렌더 전용 display 확장 경로에서만 U+302E/U+302F를 spacing glyph로 치환
- U+302E/U+302F 측정 폭을 narrow punctuation으로 정합
- 재현 HWP와 한컴 PDF 기준 파일 포함
- 새 코드 주석의 "전각 분류" 표현은 최종 구현과 맞지 않으므로 비차단 정리 후보로 기록

### Stage 5. 로컬 검증

완료.

- cargo 검증 전 `target` 하위 항목 삭제
- `CARGO_INCREMENTAL=0 cargo test --lib`: 2046 passed, 0 failed, 7 ignored
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `CARGO_INCREMENTAL=0 cargo test --test svg_snapshot`: 8 passed
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과

### Stage 6. 시각 검증

완료.

- `export-svg`: U+302E/U+302F/U+25CC 미출현, 선두 방점은 `<circle>` 1개로 렌더
- `export-pdf`: 기준 PDF와 동일하게 독립 점이 보이고 dotted-circle 아티팩트 없음
- `dump-pages`: 1페이지 유지

### Stage 7. merge 전 대기 조건

- GitHub Actions `Build & Test`, `Analyze (rust)` 최종 success 확인 완료
- 작업지시자 승인 완료

## merge 후 필수 후속 처리

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. merge 직전 최신 GitHub Actions와 head SHA 재확인: 완료
2. `gh pr merge 1738 --repo edwardkim/rhwp --merge --admin`: 완료
3. `git checkout devel && git pull --ff-only upstream devel`: fast-forward sync 완료
4. #1735 이슈 close 여부 확인: 자동 close 실패 확인, 수동 close 완료
5. PR 감사 코멘트: 완료
6. 리뷰 문서를 `mydocs/pr/archives/`로 이동: 후속 문서 PR에서 반영
7. `mydocs/orders/20260701.md` 오늘할일에 #1735/#1738 처리 내용 반영: 후속 문서 PR에서 반영
8. `mydocs/manual/pr_review_workflow.md` force-cancel 절차 변경을 같은 후속 문서-only PR에 포함: 후속 문서 PR에서 반영

## 후속 코멘트 요지

- U+302E dotted-circle 아티팩트 제거를 SVG/PDF로 확인
- 한컴 기준 PDF와 rhwp PDF 모두 선두 독립 점으로 보임
- 로컬 `test --lib`, `clippy --all-targets`, `svg_snapshot` 통과
- 이전 SHA run은 force-cancel로 정리 완료
- U+302F는 best-effort이며 기준 샘플 부재 한계는 유지
