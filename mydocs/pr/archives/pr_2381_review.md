# PR #2381 검토 — 영문 README 확장 스토어 배지 추가 (kevin9327 2번째)

- PR: https://github.com/edwardkim/rhwp/pull/2381 — docs-only (+6/−0)

## 변경 본질

ko README 에만 있던 브라우저 확장 스토어 배지 3종(Chrome/Edge/Firefox)을
en README 동일 위치에 미러 — 제목 42개 대칭 구조 분석으로 누락을 판별한
근거 제시. 영어권 사용자에게 스토어 출시 사실 노출.

## 로컬 재실증 (merged tree)

충돌 0 · 배지 3종 URL ko/en **1:1 일치 확인** · 상대 링크 검사 green ·
코드 diff 0. CI 는 run 미생성(문서 1파일) — 로컬 검사로 갈음.

## 판단

**merge 권고.** 이번에도 근거(대칭 구조 분석) 있는 정확한 문서 보수.
