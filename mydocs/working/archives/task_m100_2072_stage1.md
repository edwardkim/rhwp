# Task M100 #2072 Stage 1 - 문서 지도와 권위 문서 표

## 목표

파일 이동 없이 `mydocs/manual/README.md`와 `mydocs/tech/README.md`를 추가해 새 세션과
기여자가 현재 권위 문서와 상세·조사 문서의 관계를 찾을 수 있게 한다.

## 선행 조건

- Stage 0 커밋 `102e40294`에서 내부 Markdown 상대 링크 검사와 기준선 정리를 완료했다.
- 기본 검사 명령 `python3 scripts/check_markdown_links.py`는 `manual`·`tech` 247개 문서를 검사한다.

## 이 단계의 범위

- 문서 역할과 생명주기 메타 스키마를 안내 문서에 정의한다.
- PR, 개발/검증, CLI, 시각 검증, HWP 2020 MCP 사용법의 권위 문서를 명시한다.
- HWP 스펙, IR, 렌더링, 표, 폰트, 편집 아키텍처, 기술 결정·조사 문서의 진입점을 명시한다.
- 기존 파일의 경로와 본문은 이동하거나 대량 수정하지 않는다.

## 제외 범위

- `workflow/`, `cli/`, `verification/`, `spec/`, `architecture/`, `investigations/` 디렉터리 생성과 `git mv`
- redirect stub allowlist와 이전 경로 신규 참조 금지 규칙
- 문서별 front matter 일괄 추가
- GitHub Actions의 문서 링크 검사 hard gate

## 다음 단계

1. 문서 클러스터별 현행성 감사를 진행한다.
2. 이동 대상은 파일명 접두어가 아니라 내용과 참조 빈도를 기준으로 분류한다.
3. 이동 전용 stage에서 링크 갱신과 제한된 redirect allowlist를 함께 검증한다.
