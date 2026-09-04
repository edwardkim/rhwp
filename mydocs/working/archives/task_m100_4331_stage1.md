# Task M100 #4331 Stage 1 — 루트 README 릴리스 바이너리 도달 경로 신설

- 이슈: [#4331](https://github.com/edwardkim/rhwp/issues/4331)
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `task_m100_4331`
- 작성일: 2026-08-09 KST
- 상태: 문서 전용, 검증 완료

## 배경 (실측)

#4327 §4: `release-binary.yml`(#612)이 매 릴리스에 4플랫폼 바이너리+SHA256SUMS 를
첨부 중(v0.8.2 다운로드 1,300+)인데, 루트 README 설치 안내는 npm(웹 축)·소스
빌드·docker 뿐이라 Releases 로 가는 경로가 문서에 없었다. MCP 절 `"command":
"rhwp"` 와 파이썬 절 `RHWP_BIN` 이 전제하는 실행 파일 획득 경로가 소스 빌드로만
이어지는 상태. 트랙 D R38 문서(PR #4328 갱신)가 이 조각을 "채널 등재와 독립적인
선행 조각"으로 분리 명시했다.

## 변경 (README.md 3곳, 최소 범위)

1. "Quick Start (소스 빌드)" 앞에 **"설치 — 빌드 없이 CLI·MCP 쓰기"** 절 신설:
   Releases 최신 링크, 4플랫폼 자산·SHA256SUMS 안내, 해제→검증→`rhwp capabilities`
   첫 명령 3줄, PATH 배치가 MCP·파이썬 절의 전제를 채운다는 연결, 설치 관리자
   등재는 로드맵 추적 중임을 명시.
2. 파이썬 절 `RHWP_BIN` 줄에 대안 주석 1줄(릴리스 바이너리 경로 — 소스 빌드
   불필요).
3. MCP 절 `.mcp.json` 스니펫 아래에 획득 경로 연결 2줄.

## 검증

- `python scripts/check_markdown_links.py` — 이상 없음.
- `git diff --check` — 통과.
- 코드·렌더·워크플로 무변경(문서 전용). Cargo·WASM·시각 검증 해당 없음.
