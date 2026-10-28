# Task M100 #2072 Stage 29 - 문서 링크 검사 수동화

## 배경

문서 추가·수정만으로 GitHub Actions가 매번 실행되는 것은 저장소 운영 비용에 비해 이득이 작다.
문서 링크와 메타데이터 검사 도구는 유지하되 자동 CI workflow에서는 제거하고, 대규모 문서 이동이나
문서 구조 리팩토링처럼 필요한 시점에만 로컬에서 실행한다.

## 목표

- `.github/workflows/docs-link-check.yml`을 제거한다.
- 문서 링크·메타데이터 검사 도구의 로컬 사용법을 전용 manual에 정리한다.
- `AGENTS.md`, 문서 지도와 #2072 최종 보고서에서 자동 CI라는 설명을 제거한다.
- 과거 stage 문서는 당시 수행 기록이므로 소급 수정하지 않는다.

## 검증 계획

- 기본 링크 검사
- `upstream/devel` 이후 변경 문서와 redirect 재참조 검사
- 문서 메타데이터 검사
- Python 구문 검사와 `git diff --check`

## 구현 결과

- `.github/workflows/docs-link-check.yml`을 제거했다.
- `mydocs/manual/markdown_link_check_guide.md`에 실행 시점, 기본·파일 단위·변경분·redirect·메타데이터
  검사 방법을 정리했다.
- `AGENTS.md`와 문서 지도는 검사 명령을 중복하지 않고 새 manual을 가리킨다.
- 일반 Markdown 추가·수정은 자동 GitHub Actions 대상이 아님을 명시했다.

## 검증 결과

- `python3 scripts/check_markdown_links.py`: 384개 통과
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel
  --forbid-redirect-references`: 633개 Markdown, 변경 파일 636개, redirect 31개 통과
- `python3 scripts/check_document_metadata.py`: 379개 통과
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `git diff --check`: 통과
- `.github/workflows/`에 문서 링크 검사 workflow 없음
