# PR #1449 처리 보고서 — PR 리뷰 워크플로 문서 규칙 분리

- PR: https://github.com/edwardkim/rhwp/pull/1449
- 제목: `Task #1425: PR 리뷰 워크플로 문서 규칙 분리`
- 작성자: `postmelee` (collaborator, 누적 30 PR)
- 연결 이슈: #1425 (Closes)
- base ← head: `devel` ← `edwardkim:task_m100_1425`
- 처리 경로: collaborator self-merge 후보 예외 경로
- 처리일: 2026-06-21

## 1. 처리 결정

**admin merge 후 close.**

문서 전용 변경(7 files, +593/-37, 소스 변경 0)이며 이슈 #1425 요구사항 3건을 모두
충족한다. BEHIND 상태였으나 merge-tree 충돌 시뮬레이션 0건을 확인하여 작업지시자 판단으로
`gh pr merge --merge --admin` 으로 즉시 반영했다.

## 2. 검토 결과 (이슈 #1425 요구 대비)

| 요구사항 | 충족 | 근거 |
|---|---|---|
| maintainer 일반 경로 복구 | ✅ | 3장 active 경로(`pr/pr_{N}_review*.md`) 작성 → 7.5절 `archives/` 이동 (PR #1420 이전 기준) |
| collaborator self-merge 예외 분리 | ✅ | 8장 신설 (적용 조건/문서 경로/`upstream` push/merge 전 조건), PR #1420 추가분 보존 |
| volatile 상태값 규칙 강화 | ✅ | 3.3절 신설 (`draft`/`mergeable`/`head SHA`/`CI 상태` 단정 금지 + 권장 표기 + 금지 예시) |

부가 확인:

- 섹션 번호 재정렬(8→9 … 11→12) 일관.
- 산출 문서 파일명 규칙 정합 (`task_m100_1425.md`/`_impl.md`/`_report.md`, report/ 위치).
- orders 충돌 해소: PR #1447 항목 보존 + #1425 항목 추가 (자동 병합).
- 자기검열 통과 (비교/최상급/공공기관 오인 표현 없음).
- CI: 문서 전용이라 워크플로 미트리거 (정상). 소스 무변경으로 빌드/테스트 영향 없음.

## 3. 특이사항 — 자기참조 구조

postmelee 가 본 PR 에서 새로 정의한 8장 "collaborator self-merge 후보 예외 경로" 규칙을
자신의 PR 에 그대로 적용했다. review/impl 문서를 PR head 의 `pr/archives/` 에 포함하여
merge 후 추가 문서 커밋이 불필요하다. 신규 규칙과 정합한다.

## 4. 후속 검증

- merge commit `e211e453` → `git branch -r --contains` 로 origin/devel 포함 확인.
- 로컬 `devel`·`local/devel` 을 origin/devel 로 동기화.
- 이슈 #1425: devel 이 default branch 가 아니라 `Closes` 자동 동작이 안 되어 수동 close
  (작업지시자 승인 후).

## 5. 산출물

- review/impl 문서: PR head 에 포함 (`mydocs/pr/archives/pr_1449_review.md`, `pr_1449_review_impl.md`)
- 본 처리 보고서: `mydocs/pr/archives/pr_1449_report.md`
