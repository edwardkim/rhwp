# HWP 2020 변환 client 사용법

이 문서는 rhwp 작업에서 로컬 HWP/HWPX 파일을 원격 한컴 변환 서버로 보내고, 변환 결과를 다시 로컬에 저장하는 절차를 정리한다.

PR review와 자동화에서는 `hwp2020-mcp-convert` CLI 사용을 권장한다. 이 방식은 로컬 MCP stdio server를 띄우거나 VS Code MCP 서버를 등록할 필요가 없다.

## 개요

- 권장 실행 방식: `hwp2020-mcp-convert` CLI
- 선택 실행 방식: VS Code MCP 서버 `hwp2020Convert` / tool `convert_local_document`
- 지원 입력: `.hwp`, `.hwpx`
- 지원 출력: `pdf`, `hwpx`, `hwp`
- PDF 생성은 서버 구현의 현재 방식, 즉 인쇄 설정을 명시한 Print 방식 기준이다.
- CLI의 `--input`과 `--output-dir`은 모두 client가 실행되는 로컬 Mac 경로다.

서버 URL/IP와 인증 토큰은 공개 문서, GitHub issue/PR/comment, 로그에 기록하지 않는다. 인증된 collaborator만 비공개 채널로 공유받고, 필요하면 `@jangster77`에게 요청한다.

## 로컬 준비

MCP client tarball은 rhwp 저장소의 `tools/` 아래에 둔다. 서버 URL/token을 담은 `.env.local`은 사용자 PC의 로컬 client 디렉터리에 둔다.

예시:

```text
/Users/me/rhwp/
  tools/hwp-convert-mcp-client-20260709-231800.tar.gz

/Users/me/Devel/hwp-convert/
  .env.local
```

`.env.local` 예:

```env
HWP2020_MCP_SERVER_URL=http://<server-ip>:3001/mcp
HWP2020_MCP_AUTH_TOKEN=<token>
```

`.env.local`은 Git에 커밋하지 않는다.

## 권장: CLI 변환

`hwp2020-mcp-convert` CLI가 로컬 파일을 읽고 원격 HTTP MCP 서버를 호출한 뒤 결과를 로컬 `output_dir`에 저장한다. 따라서 일반 변환, PR review 기준 PDF 생성, 자동화 스크립트에서는 이 방식을 사용한다.

도움말:

```bash
/opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz \
  -- \
  hwp2020-mcp-convert --help
```

변환 예:

```bash
/opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz \
  -- \
  hwp2020-mcp-convert \
  --env-file /Users/me/Devel/hwp-convert/.env.local \
  --input /Users/me/rhwp/samples/example.hwp \
  --target pdf \
  --output-dir /Users/me/rhwp/pdf \
  --output-filename example-2020.pdf \
  --timeout-seconds 240
```

성공하면 JSON으로 `status`, `output_path`, `size`, `sha256`, `server.job_id`, `server.validation` 등이 출력된다.

페이지 수가 많거나 거대 표/중첩 표/성능 검증 샘플처럼 오래 걸릴 수 있는 문서는 `--timeout-seconds`를 900~1800초 범위로 충분히 크게 잡는다.

## 선택: VS Code MCP 등록

VS Code Chat에서 자연어로 `hwp2020Convert` tool을 호출하고 싶을 때만 MCP stdio 등록을 사용한다. 이 경우에도 사용자가 터미널에서 MCP client를 계속 띄워 둘 필요는 없다. VS Code가 `mcp.json`의 `command`를 실행해 MCP stdio server process를 시작한다.

`hwp2020-mcp-bridge`는 client tarball 내부의 MCP stdio 진입점 이름일 뿐이다. CLI 변환에는 이 경로가 필요 없다.

VS Code MCP 로그에서 다음 순서가 보이면 tool discovery 자체는 성공한 것이다.

```text
Connection state: Starting
Connection state: Running
Discovered 1 tools
```

`Stopping server - hwp2020Convert`는 VS Code reload, MCP 서버 재시작, 사용자가 서버를 중지한 경우에도 남을 수 있다. 과거 로그의 마지막 줄이 `Stopping server`라고 해서 항상 현재 서버가 고장난 것은 아니다.

반대로 아래 상태가 계속 반복되면서 `MCP: List Servers`에 `hwp2020Convert`가 보이지 않거나 tool 호출이 실패하면 비정상으로 본다.

```text
Connection state: Error Process exited with code null
```

판정 기준:

- `MCP: List Servers`에서 `hwp2020Convert`가 보임: 등록 정상
- 로그에 `Discovered 1 tools`가 있음: `convert_local_document` discovery 성공
- tool 호출로 PDF가 생성됨: end-to-end 정상
- `Error` 뒤에 다시 `Running`/`Discovered 1 tools`로 돌아오지 않음: 설정 또는 runtime 문제
- 현재 프로세스 확인 시 `hwp2020-mcp-bridge`가 살아 있음: VS Code가 MCP stdio process를 실행 중

## 선택: VS Code 변환 요청 예

VS Code Chat 또는 MCP를 사용할 수 있는 클라이언트에서 다음처럼 요청한다.

```text
hwp2020Convert를 사용해서 /Users/me/rhwp/samples/example.hwp 파일을 pdf로 변환하고 /Users/me/rhwp/pdf 에 저장해줘.
```

직접 tool 인자로 표현하면 다음과 같다.

```json
{
  "input_path": "/Users/me/rhwp/samples/example.hwp",
  "target": "pdf",
  "output_dir": "/Users/me/rhwp/pdf",
  "output_filename": "example-2020.pdf",
  "timeout_seconds": 240
}
```

주요 선택 인자:

```json
{
  "clear_distribution": false,
  "table_patch_last_row_count": false,
  "disable_pdf_hwpx_fallback": true,
  "allow_sibling_fallbacks": false
}
```

## rhwp PR 리뷰에서의 저장 규칙

PR 리뷰나 시각 검증 기준 PDF로 사용할 출력은 `output/` 아래에만 두지 않는다. 50MB 미만이면 저장소의 `pdf/` 아래에 둔다.

예:

```text
samples/task2097/1730000_selection_report.hwp
pdf/task2097/1730000_selection_report-2020.pdf
```

하위 디렉터리가 있는 샘플은 `pdf/` 아래에서도 같은 하위 구조를 유지한다.

## 성공 확인

CLI 변환 결과에서 다음을 확인한다.

- `status: success`
- `server.run_status: 0`
- `server.validation: ok`
- 출력 PDF가 로컬 `output_dir`에 존재
- `pdfinfo`로 페이지 수 확인 가능

예:

```bash
pdfinfo /Users/me/rhwp/pdf/task2097/1730000_selection_report-2020.pdf | rg '^(Pages|Page size):'
file /Users/me/rhwp/pdf/task2097/1730000_selection_report-2020.pdf
shasum -a 256 /Users/me/rhwp/pdf/task2097/1730000_selection_report-2020.pdf
```

페이지 수가 많거나 거대 표/중첩 표/성능 검증 샘플처럼 오래 걸릴 수 있는 문서는 `--timeout-seconds`를 900~1800초 범위로 충분히 크게 잡는다.

## 문제 해결

CLI 도움말 또는 변환 호출이 실패하면 다음을 확인한다.

- `tools/hwp-convert-mcp-client-*.tar.gz`가 repo에 존재하는지
- `--package=file:/.../tools/hwp-convert-mcp-client-*.tar.gz`처럼 `file:` 스킴과 절대경로를 사용했는지
- `/opt/homebrew/bin/npx`가 실제 `npx` 경로와 맞는지
- `--env-file`이 절대경로인지
- `.env.local`에 `HWP2020_MCP_SERVER_URL`과 `HWP2020_MCP_AUTH_TOKEN`이 있는지
- 서버 URL이 `/mcp` endpoint까지 포함하는지
- `--input` 파일이 로컬에 존재하는지
- `--output-dir`의 상위 경로에 쓰기 권한이 있는지
- 큰 문서에서 `--timeout-seconds`를 충분히 크게 잡았는지

VS Code MCP 등록을 사용할 때만 다음을 추가로 확인한다.

- `MCP: Reset Cached Tools` 명령이 없는 VS Code 버전에서는 `Developer: Reload Window` 또는 VS Code 재시작으로 대체한다.
- `envFile`이 절대경로인지
- `env.HWP2020_MCP_CLIENT_PACKAGE`가 repo의 `tools/hwp-convert-mcp-client-*.tar.gz` 절대경로를 가리키는지
- `command`가 GUI 환경에서 접근 가능한 wrapper 절대경로인지
- wrapper에 실행 권한이 있는지
- `npx`가 `/opt/homebrew/bin/npx`가 아니면 `env.HWP2020_MCP_NPX`로 실제 경로를 지정했는지
