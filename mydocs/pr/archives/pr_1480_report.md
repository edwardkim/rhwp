# PR #1480 처리 보고서 — Chrome 다운로드 filename interceptor 부작용 수정

- PR: https://github.com/edwardkim/rhwp/pull/1480
- 제목: `Task #1471: Fix Chrome download filename interceptor side effect`
- 작성자: postmelee (collaborator)
- 연결 이슈: #1471 (Refs → 수동 close)
- base ← head: `devel` ← `postmelee:task_m100_1471`
- 처리 경로: collaborator self-merge 후보 (review/impl 문서 PR head 포함)
- 처리일: 2026-06-22

## 1. 처리 결정

**admin merge.** Chrome `downloads.onDeterminingFilename` 리스너 등록 자체가 다른 확장의
`download({filename})` 하위폴더 저장을 무효화하던 전역 부작용을 해소. CI 통과 + 로컬 검증 통과
+ 충돌 0건. 0.7.17 확장 0.2.6 에 포함.

- merge commit: `8da4f8fe`
- PR head → `git branch -r --contains` 로 origin/devel 포함 확인.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `rhwp-chrome/sw/download-interceptor.js` | `onDeterminingFilename.addListener` 제거 → `onCreated`/`onChanged` 관찰자 전환. `handled` 집합 중복 방지, `downloads.search({id})` 재판정. 기존 autoOpen·대용량 경고·file:// 억제 유지 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | Chrome downloads mock 테스트 6종 |
| `mydocs/...task_m100_1471*` + `pr_1480_review*` | Hyper-Waterfall 문서 + self-merge review 문서 |

## 3. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI | 4 pass |
| 충돌 시뮬레이션 | 0건 |
| SW mock 테스트 (`download-interceptor.test.mjs`) | 6 pass / 0 fail (onDeterminingFilename 미등록·비-HWP blob 무개입·HWP 자동열기·onChanged 재판정·file:// 억제) |
| chrome 확장 빌드 (`npm run build`) | OK |
| 빌드 산출물 `onDeterminingFilename.addListener` 잔여 | 없음 (설명 주석만 잔존) |

## 4. 의의

- 다른 확장의 다운로드 filename/하위폴더 결정에 더 이상 개입하지 않음 — 실사용 충돌 해소.
- Firefox 구현(onCreated/onChanged)과 동형 구조로 수렴.

## 5. 산출물

- review/impl 문서: PR head 동봉 (`mydocs/pr/archives/pr_1480_review.md`, `pr_1480_review_impl.md`)
- 본 처리 보고서: `mydocs/pr/archives/pr_1480_report.md`
