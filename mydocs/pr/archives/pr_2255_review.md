# PR #2255 리뷰 - HWP 2020 MCP client 장문서 timeout 동기화

## PR 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2255](https://github.com/edwardkim/rhwp/pull/2255) |
| 관련 이슈 | [#2253](https://github.com/edwardkim/rhwp/issues/2253) |
| 작성자 | `jangster77` |
| base | `devel` |
| 작성 시점 참고값 | Open, 최초 head `62de42a47` |
| merge 전 조건 | 최신 PR head 기준 GitHub Actions 통과와 작업지시자 승인 |

## 변경 범위

- `tools/hwp-convert-mcp-client-20260713-075145.tar.gz`를 추가했다.
  - CLI와 stdio bridge는 MCP `callTool` 요청 대기 시간을 사용자가 지정한
    `timeout_seconds`에 120초를 더해 설정한다.
  - 이전 `20260709-231800` client tarball을 제거해 사용자가 오래된 package를 선택하지 않게 한다.
- [MCP 사용 매뉴얼](../../manual/mcp_hwp2020Convert_usage.md)은 새 tarball 경로와
  긴 변환 요청 timeout 동기화 규칙을 안내한다.
- [PR 리뷰 워크플로우](../../manual/pr_review_workflow.md)는 대형 문서 기준 PDF 생성 시
  새 client 이상을 사용하고, CLI 재호출로 로컬 수신을 확인하도록 정정한다.
- `mydocs/orders/20260713.md`에 collaborator self-PR 준비 기록을 추가했다.

## 검토 결과

수용 가능하다. 변경은 client 배포물과 사용 절차에 한정되며 renderer, WASM, Rust 소스,
테스트, 샘플, 기준 PDF를 수정하지 않는다. 따라서 시각 결과를 바꾸지 않아 visual sweep은
적용 대상이 아니고, cargo 빌드 및 회귀 테스트도 이 문서·배포본 변경의 직접 검증이 아니다.

tarball은 `.env.local`이 아니라 `.env.local.example`만 포함한다. 서버 URL/IP와 인증 토큰은
PR 본문, 매뉴얼, 검토 기록에 넣지 않았다.

## 검증

- `git diff --check` 통과
- SHA-256 확인
  - `9280c02f370641d9c7f5d8cffd6160af7c1e22334638344882323367f8210f82`
- tarball 파일 목록 확인
  - CLI, stdio bridge, 예시 설정, `.env.local.example`, package metadata를 포함
- `node --check` 통과
  - `tools/hwp2020_mcp_client_convert.mjs`
  - `tools/hwp2020_mcp_stdio_bridge.mjs`
- `npx --package=file:... hwp2020-mcp-convert --help` 통과
  - `--timeout-seconds`가 conversion 및 MCP request timeout임을 출력에서 확인

## 후속 처리

최신 head의 GitHub Actions가 통과하면 collaborator merge 후보로 검토한다. 이 PR은
`Closes [#2253](https://github.com/edwardkim/rhwp/issues/2253)`를 포함하므로 merge 후
이슈 자동 close 상태와 브랜치 정리만 확인한다.
