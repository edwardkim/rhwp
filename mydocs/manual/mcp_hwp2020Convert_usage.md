# hwp2020Convert MCP 사용법

이 문서는 VS Code/Codex에서 `hwp2020Convert` MCP 서버를 사용해 로컬 HWP/HWPX 파일을 원격 한컴 변환 서버로 보내고, 변환 결과를 다시 로컬에 저장하는 절차를 정리한다.

## 개요

- VS Code MCP 서버 이름: `hwp2020Convert`
- MCP tool 이름: `convert_local_document`
- 지원 입력: `.hwp`, `.hwpx`
- 지원 출력: `pdf`, `hwpx`, `hwp`
- PDF 생성은 서버 구현의 현재 방식, 즉 인쇄 설정을 명시한 Print 방식 기준이다.
- `input_path`와 `output_dir`은 모두 MCP 클라이언트가 실행되는 로컬 Mac 경로다.

서버 URL/IP와 인증 토큰은 공개 문서, GitHub issue/PR/comment, 로그에 기록하지 않는다. 인증된 collaborator만 비공개 채널로 공유받고, 필요하면 `@jangster77`에게 요청한다.

## 로컬 준비

MCP 클라이언트 패키지와 환경 파일을 한 디렉터리에 둔다.

예시:

```text
/Users/me/Devel/hwp-convert/
  .env.local
  hwp-convert-mcp-client-20260707-308464b.tar.gz
```

`.env.local` 예:

```env
HWP2020_MCP_SERVER_URL=http://<server-ip>:3001/mcp
HWP2020_MCP_AUTH_TOKEN=<token>
```

`.env.local`은 Git에 커밋하지 않는다.

## help 확인

`npx --package`에는 tarball 경로 앞에 `file:` 스킴을 붙인다.

```bash
npx -y \
  --package=file:/Users/me/Devel/hwp-convert/hwp-convert-mcp-client-20260707-308464b.tar.gz \
  -- \
  hwp2020-mcp-bridge --help
```

정상 출력에는 다음 내용이 포함되어야 한다.

```text
MCP tool:
  convert_local_document
```

## VS Code MCP 등록

macOS GUI에서 실행되는 VS Code는 shell PATH를 그대로 받지 못할 수 있으므로 `npx` 절대경로를 확인한다.

```bash
which npx
```

Apple Silicon Homebrew 환경에서는 보통 `/opt/homebrew/bin/npx`다.

워크스페이스 설정 파일:

```text
/Users/me/rhwp/.vscode/mcp.json
```

예시:

```json
{
  "servers": {
    "hwp2020Convert": {
      "type": "stdio",
      "command": "/opt/homebrew/bin/npx",
      "args": [
        "-y",
        "--package=file:/Users/me/Devel/hwp-convert/hwp-convert-mcp-client-20260707-308464b.tar.gz",
        "--",
        "hwp2020-mcp-bridge"
      ],
      "envFile": "/Users/me/Devel/hwp-convert/.env.local",
      "env": {
        "PATH": "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
      }
    }
  }
}
```

설정 후 VS Code에서 `Developer: Reload Window`를 실행하거나 VS Code를 재시작한다. 이후 `MCP: List Servers`에서 `hwp2020Convert`가 보여야 한다.

## 변환 요청 예

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

MCP 변환 결과에서 다음을 확인한다.

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

페이지 수가 많거나 거대 표/중첩 표/성능 검증 샘플처럼 오래 걸릴 수 있는 문서는 `timeout_seconds`를 900~1800초 범위로 충분히 크게 잡는다.

## 문제 해결

`MCP: Reset Cached Tools` 명령이 없는 VS Code 버전에서는 `Developer: Reload Window` 또는 VS Code 재시작으로 대체한다.

서버는 보이지만 tool 호출이 실패하면 다음을 확인한다.

- `envFile`이 절대경로인지
- `.env.local`에 `HWP2020_MCP_SERVER_URL`과 `HWP2020_MCP_AUTH_TOKEN`이 있는지
- tarball 경로가 절대경로이고 `--package=file:/...tar.gz` 형태인지
- `command`가 GUI 환경에서 접근 가능한 `npx` 절대경로인지
- 서버 URL이 `/mcp` endpoint까지 포함하는지
