# Task M100 #2072 Stage 3 - 부트스트랩과 문서 이동 사전 통제

## 목표

Stage 0~2 검토에서 확인된 P1~P3를 파일 이동 전에 해소한다. 저장소 밖 개인 경로나 현재 세션 상태에
의존하지 않는 부트스트랩을 만들고, 권위 문서의 현행성·문서 참조·비 Markdown 자산·CLI 문서 역할을
명확히 한다.

## 범위

- 저장소 루트 `AGENTS.md`를 추가한다. 모든 경로는 저장소 상대 경로만 사용한다.
- `mydocs/README.md`에 canonical 후보 manifest와 문서 탐색 순서를 추가한다.
- 오래된 현재 브랜치 기록과 직접 `devel` push 지시를 `docs_and_git_workflow.md`에서 제거한다.
- Markdown 링크 검사를 루트 안내 문서와 이미지 링크까지 넓히고, 이동 후 이전 경로를 금지하는
  `--forbid-path` 사전 점검을 제공한다.
- 문서 전용 변경에도 링크 검사가 실행되도록 별도 GitHub Actions workflow를 추가한다.
- OWPML XML/PowerShell reference 자산의 역할과 진입점을 문서화한다.
- `cli_commands.md`의 중복된 HWP5 조사 섹션을 하나로 합친다. Skill 가이드와 세부 명령 매뉴얼은
  역할 분리만 기록하며, 예제까지 기계적으로 제거하지 않는다.
- `thorvg_decision.md`의 중복 POC 링크를 정리한다.

## 제외 범위

- `manual/verification/` 또는 `tech/*` 파일 이동과 redirect stub 생성
- 기존 문서 전부에 front matter를 일괄 추가하는 작업
- 개인용 전역 설정, 인증 토큰, 호스트명, 로컬 절대 경로 기록
- CLI 명령 구현이나 `--help` 문자열 변경

## CLI 문서 역할 판정

| 문서 | 역할 | 중복 처리 |
| --- | --- | --- |
| `cli_commands.md` | 전체 명령·옵션의 canonical reference | HWP5 §4/§5 중복 본문을 하나로 통합 |
| `rhwp_cli_skill_guide.md` | 자연어 요청, Skill 호출, 대표 예제와 디버깅 순서 | 옵션 열거를 복제하지 않으므로 유지 |
| `dump_command.md`, `export_png_command.md`, `ir_diff_command.md` | 각 명령의 상세 출력·환경·트러블슈팅 | canonical 문서의 상세 링크 대상으로 유지 |

## 검증

- `python3 -m py_compile scripts/check_markdown_links.py`
- `python3 scripts/check_markdown_links.py`
- `python3 scripts/check_markdown_links.py --forbid-path <이동 전 경로>`의 오류 동작 확인
- GitHub Actions YAML 구조 확인과 `git diff --check`

## 결과

- 루트 `AGENTS.md`는 저장소 상대 경로만 사용하며, 개인 경로·호스트·토큰·현재 세션 상태를 포함하지 않는다.
- `mydocs/README.md`가 canonical 후보, 역할, 생명주기, 확인일을 한곳에서 제공하고 `CLAUDE.md`가 이를
  진입점으로 가리킨다.
- `docs_and_git_workflow.md`의 종료된 `#1053` 현재 상태와 직접 `origin/devel` push 지침을 제거했다.
- 링크 검사는 루트 안내 문서, `manual`, `tech`, 이미지 링크를 기본 대상으로 포함한다. `--forbid-path`는
  옛 경로를 가리키는 새 Markdown 참조를 실패로 만든다.
- 문서 전용 push/PR에서 검사하는 `Documentation Link Check` workflow를 추가했다.
- OWPML XML Schema와 PowerShell 검증 스크립트를 reference 자산으로 색인했다.
- `cli_commands.md`의 HWP5 조사·내부 개발 도구 중복 절을 제거했다. Skill 가이드와 상세 명령 매뉴얼은
  역할이 달라 유지한다.

## 다음 단계

Stage 4에서 manifest의 canonical 후보별 현행성을 확인하고, 제한된 redirect allowlist를 확정한 뒤
문서 클러스터 이동을 수행한다.
