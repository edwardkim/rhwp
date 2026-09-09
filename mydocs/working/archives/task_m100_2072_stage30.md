# Task M100 #2072 Stage 30 - 최신 devel 동기화와 PR 준비

## 배경

#2072 문서 정보구조 리팩토링은 Stage 29까지 구현과 로컬 검증을 마쳤다. PR 준비 시점에
`upstream/devel`이 51개 커밋 앞서 있으므로, 작업 커밋을 보존한 전용 브랜치에서 최신 원격을 병합하고
문서 이동·메타데이터·부트스트랩 변경의 충돌을 현재 권위 문서 기준으로 해소해야 한다.

## 목표

- #2072 작업 30개 커밋을 전용 PR 브랜치에 보존한다.
- 최신 `upstream/devel`을 병합하고 문서 경로와 운영 기록 충돌을 해소한다.
- PR 생성 직전 최신 오늘할일에 #2072 완료 및 PR 준비 기록을 추가한다.
- 링크, redirect 이전 경로, 메타데이터, Python 구문과 diff 정합성을 다시 검증한다.
- 코드 동작 변경이 없는 문서 정보구조 PR로 최종 범위와 PR 본문 초안을 확정한다.

## 충돌 해소 원칙

- 최신 `upstream/devel`의 제품 코드와 다른 task 운영 기록을 보존한다.
- #2072가 이동·분류한 문서는 새 canonical 경로와 front matter를 유지한다.
- 같은 문서를 원격이 갱신한 경우 원격의 최신 내용과 #2072의 역할·경로 변경을 함께 반영한다.
- 일반 Markdown 변경마다 자동 CI를 실행하지 않으며, 로컬 검사 가이드를 최종 정책으로 유지한다.

## 검증 계획

- `python3 scripts/check_markdown_links.py`
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references`
- `python3 scripts/check_document_metadata.py`
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`
- `git diff --check`

## 구현 결과

- `codex/task2072-doc-ia-final-20260717` 전용 PR 브랜치를 생성했다.
- 최신 `upstream/devel` 51개 커밋을 충돌 없이 병합했다.
- 원격의 v0.7.19 준비, 제품 코드, 신규 샘플과 운영 기록은 그대로 보존했다.
- `mydocs/orders/20260717.md`에 #2072 완료 범위와 PR 생성 승인 대기 상태를 추가했다.
- PR diff는 문서 정보구조, 로컬 문서 검사 도구, 제품 소스의 문서 경로 주석 2곳으로 한정된다.

## 검증 결과

- `python3 scripts/check_markdown_links.py`: 384개 통과
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel
  --forbid-redirect-references`: 636개 Markdown, 변경 파일 639개, redirect 31개 통과
- `python3 scripts/check_document_metadata.py`: 379개 통과
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `git diff --check`: 통과
- 제품 동작 변경: 없음
- Cargo 빌드·테스트: 문서 구조와 로컬 검사 도구 변경이므로 수행하지 않음
