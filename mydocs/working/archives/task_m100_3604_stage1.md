# Task #3604 관련 Stage 1: client archive와 HWP 2020 MCP 사용법 현행화

Issue: #3604 관련

## 목표

최신 HWP 2020 MCP client archive 하나를 `tools/`에 배포하고 manual이 실제 암호 문서 변환 범위와
client 로컬 입력·출력·비밀 처리 규칙을 설명하게 한다.

## 시작 상태

- 기준 base는 `upstream/devel` `40ae572ce`다.
- 기존 tarball은 `hwp-convert-mcp-client-20260728-215936.tar.gz` 하나이며 manual도 이를 참조한다.
- 운영 MCP에서는 암호 HWP3 HWP 세 방향, 암호 HWP5 HWP 세 방향, 암호 HWPX 두 방향이 모두 성공했다.
  각 case는 client/server SHA-256 일치와 `response_finished` delivery를 확인했다.
- HWP5의 큰 HWPX 본문은 validator buffer 한계를 수정한 운영 server에서 정상 처리된다.

## 구현 계획

1. hwp-convert source의 최신 client archive와 manifest/help를 확인한다.
2. 이전 archive를 새 archive로 교체한다.
3. manual에서 archive path와 암호·timeout·cleanup·검증 설명을 갱신한다.
4. archive/help/문서 경로/diff 검증 뒤 code artifact와 문서를 함께 커밋한다.

## 성공 기준

- `tools/`에는 새 `hwp-convert-mcp-client-*.tar.gz` 하나만 있다.
- manual의 모든 archive reference가 새 이름을 사용하고, token·endpoint·비밀번호를 포함하지 않는다.
- 새 archive의 `hwp2020-mcp-convert --help`가 성공한다.
- PR은 #3604를 관련 이슈로만 언급하며 `devel`을 base로 한다.

## 테스트 결과

실행 시각: 2026-07-31 KST

| 검증 | 결과 |
| --- | --- |
| archive manifest | `kind=client`, source commit `4d67020`, Hancom Office runtime 불포함을 확인했다. |
| archive 비밀 파일 | `.env.local`은 없고 `.env.local.example`만 포함한다. |
| `npx --package=file:... hwp2020-mcp-convert --help` | 성공. `convert`, `start`, `status`, `download`와 HWP3/HWP5 HWP·ODF 암호 HWPX stdin 암호 입력 범위를 표시했다. |
| `npx --package=file:... hwp2020-mcp-bridge --help` | 성공. 네 MCP tool과 비동기 lifecycle을 표시했다. |
| archive 개수 | `tools/hwp-convert-mcp-client-*.tar.gz`는 새 archive 한 개다. |
| old archive 참조 | manual과 `tools/`에서 이전 archive ID 결과가 없다. |
| `python3 scripts/check_markdown_links.py mydocs/manual/mcp_hwp2020Convert_usage.md` | 성공. 내부 상대 링크 이상 없음. |
| `python3 scripts/check_document_metadata.py` | 성공. 428개 문서 metadata 이상 없음. |
| `git diff --check` | 성공 |

Cargo, renderer, fixture 검증은 실행하지 않았다. 이 변경은 rhwp Rust source·sample·기준 PDF를 바꾸지 않고,
외부 MCP client archive와 사용법만 갱신한다. 실제 변환 기능의 근거는 운영 MCP에서 이미 완료한 암호 문서
8방향 결과이며, 이 stage에서는 배포 artifact가 그 기능을 호출할 수 있는지 확인했다.
