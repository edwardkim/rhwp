---
kind: guide
status: active
canonical: mydocs/manual/mcp_hwp2020Convert_usage.md
last_verified: 2026-07-28
---

# HWP 2020 변환 client 사용법

이 문서는 rhwp 작업에서 로컬 HWP/HWPX 파일을 원격 Hancom Office 2020 MCP 변환 서비스로 보내고,
변환 결과를 다시 로컬에 저장하는 방법을 설명한다.

## 개요

- 권장 실행 방식: `hwp2020-mcp-convert` CLI
- 선택 실행 방식: VS Code MCP 서버 `hwp2020Convert` / tool `convert_local_document`
- 지원 입력: `.hwp`, `.hwpx`
- 지원 출력: `pdf`, `hwpx`, `hwp`
- 변환 방향: `.hwp -> pdf|hwpx|hwp`, `.hwpx -> pdf|hwp`이며 `.hwpx -> hwpx`는 지원하지 않는다.
- PDF는 Hancom Office 2020의 non-GUI `PrintToPDFEx`와 `PrintMethod=0`(1-up) 인쇄 경로로 생성한다.
- 암호가 설정된 `.hwp`와 `.hwpx`는 `password`(VS Code) 또는 `--password-stdin`(CLI)으로 변환할 수 있다.
  암호 HWPX는 ODF AES-256-CBC/PBKDF2 형식만 지원하며 DRM 등 그 밖의 암호화 문서는 지원하지 않는다.
- CLI의 `--input`, `--output-dir`과 VS Code tool의 `input_path`, `output_dir`은 모두 client가 실행되는
  로컬 PC 경로다. 원격 서버의 경로를 입력하지 않는다.
- 변환본은 MCP response의 inline binary로 client에 전달되며, server는 응답을 만든 직후 input, intermediate,
  output, log가 있는 작업 directory를 삭제한다. server에서 나중에 다시 내려받는 persistent `resource_uri`는 제공하지 않는다.

서버 URL/IP, 인증 토큰, `.env.local` 내용은 Git, issue, PR, 공개 문서, 로그에 기록하지 않는다.
인증된 collaborator만 비공개 채널로 공유받고, 필요하면 `@jangster77`에게 요청한다.

## 최신 client artifact

rhwp에는 최신 client artifact 하나만 유지한다.

```text
tools/hwp-convert-mcp-client-20260728-215936.tar.gz
```

이전 client archive는 사용하거나 보관하지 않는다. 작업 PC에서는 새 archive만 유지한다.

## 로컬 준비

MCP client tarball은 rhwp 저장소의 `tools/` 아래에 둔다. 서버 URL/token을 담은 `.env.local`은
사용자 PC의 로컬 client 디렉터리에 둔다.

```text
/Users/me/rhwp/
  tools/hwp-convert-mcp-client-20260728-215936.tar.gz

/Users/me/Devel/hwp-convert/
  .env.local
```

`.env.local` 예:

```env
HWP2020_MCP_SERVER_URL=http://<관리자가_제공한_MCP_endpoint>
HWP2020_MCP_AUTH_TOKEN=<관리자가_제공한_토큰>
```

`.env.local`은 Git에 커밋하지 않는다.

## 권장: CLI 변환

`hwp2020-mcp-convert` CLI가 로컬 파일 바이트를 읽어 원격 HTTP MCP에 전송하고, 응답 변환본을 로컬
`output_dir`에 저장한다. 일반 변환, PR review 기준 PDF 생성, 자동화에는 이 방식을 사용한다.

도움말:

```bash
/opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260728-215936.tar.gz \
  -- \
  hwp2020-mcp-convert --help
```

PDF 변환 예:

```bash
/opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260728-215936.tar.gz \
  -- \
  hwp2020-mcp-convert \
  --env-file /Users/me/Devel/hwp-convert/.env.local \
  --input /Users/me/rhwp/samples/example.hwp \
  --target pdf \
  --output-dir /Users/me/rhwp/pdf \
  --output-filename example-2020.pdf \
  --paper-size a4 \
  --orientation landscape \
  --timeout-seconds 900
```

성공하면 JSON으로 `status`, `output_path`, `size`, `sha256`, validation, conversion metadata가
출력된다.

### 암호 HWP/HWPX CLI 변환

암호는 command argument나 `.env.local`에 저장하지 않는다. `--password-stdin`은 stdin 첫 줄을 읽어 MCP
request의 `password`로 전달하며, 대화형 prompt를 열지 않는다.

```bash
printf '%s\n' "$HWP_DOCUMENT_PASSWORD" | \
  /opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260728-215936.tar.gz \
  -- \
  hwp2020-mcp-convert \
  --env-file /Users/me/Devel/hwp-convert/.env.local \
  --input /Users/me/rhwp/samples/locked.hwpx \
  --target pdf \
  --output-dir /Users/me/rhwp/pdf \
  --output-filename locked-2020.pdf \
  --timeout-seconds 900 \
  --password-stdin
```

`HWP_DOCUMENT_PASSWORD`는 현재 shell에서만 설정하고 shell history, Git, `.env.local`에는 저장하지 않는다.
암호 처리 중 생성되는 server-side HWPX intermediate와 PDF는 response 생성 후 즉시 삭제된다.

## PDF 용지와 방향

`--paper-size`와 `--orientation`은 `--target pdf`일 때만 지정한다. `hwp` 또는 `hwpx` target에
지정하면 client가 요청 전에 거부한다. 둘 다 생략하면 원본 문서의 용지와 방향을 유지한다.

| `--paper-size` | 규격 | portrait PDF page size | landscape PDF page size |
|---|---|---:|---:|
| `a4` | 210 x 297 mm | 약 595 x 842 pt | 약 842 x 595 pt |
| `a3` | 297 x 420 mm | 약 842 x 1191 pt | 약 1191 x 842 pt |
| `b4` | Korean/JIS B4, 257 x 364 mm | 약 729 x 1032 pt | 약 1032 x 729 pt |

추가로 `a5`, `letter`, `legal`을 지원한다. `--orientation`의 허용값은 `portrait`, `landscape`다.

용지 또는 방향을 명시하면 한글 2020 `PageSetup`이 문서 전체에 적용된다. 내용이 재조판되어 페이지
수가 달라질 수 있으므로, 시각 검증에서는 반드시 출력 PDF의 page size와 page count를 확인한다.

긴 문서·이미지가 많은 문서·거대 표·중첩 표는 `--timeout-seconds 900` 또는 `1800`을 사용한다.
허용 범위는 10~1800초이고, client는 이 값에 120초 여유를 더해 MCP 요청 대기 시간에도 적용한다.
따라서 SDK 기본 60초 timeout으로 먼저 종료되지 않는다.

## 선택: VS Code MCP 등록

VS Code Chat에서 자연어로 `hwp2020Convert` tool을 호출할 때만 stdio bridge를 등록한다. 사용자가
터미널에서 bridge를 미리 실행해 둘 필요는 없다. VS Code가 `mcp.json`의 `command`를 실행한다.

macOS GUI에서 실행되는 VS Code는 shell `PATH`를 그대로 받지 않을 수 있다. `npx` 절대경로를
확인한다.

```bash
which npx
```

Apple Silicon Homebrew 환경에서는 보통 `/opt/homebrew/bin/npx`다. 워크스페이스의
`.vscode/mcp.json` 예시는 다음과 같다.

```json
{
  "servers": {
    "hwp2020Convert": {
      "type": "stdio",
      "command": "/opt/homebrew/bin/npx",
      "args": [
        "-y",
        "--package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260728-215936.tar.gz",
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

MCP 접근이 막혀 있으면 VS Code 사용자 설정에 다음을 추가한다.

```json
{
  "chat.mcp.access": "all"
}
```

설정 후 `Developer: Reload Window`를 실행하거나 VS Code를 재시작한다. `MCP: List Servers`에서
`hwp2020Convert`가 보이고 tool discovery가 성공해야 한다.

## VS Code 변환 요청 예

```text
hwp2020Convert를 사용해서 /Users/me/rhwp/samples/example.hwp 파일을 A3 세로 PDF로 변환하고
/Users/me/rhwp/pdf 에 example-a3-2020.pdf 이름으로 저장해줘.
```

직접 tool 인자로 표현하면 다음과 같다.

```json
{
  "input_path": "/Users/me/rhwp/samples/example.hwp",
  "target": "pdf",
  "output_dir": "/Users/me/rhwp/pdf",
  "output_filename": "example-a3-2020.pdf",
  "timeout_seconds": 900,
  "paper_size": "a3",
  "orientation": "portrait"
}
```

암호 `.hwp` 또는 `.hwpx`는 `password`를 추가한다. 암호는 result JSON, output filename, server persistent storage에 남지 않는다.

```json
{
  "input_path": "/Users/me/rhwp/samples/locked.hwpx",
  "target": "pdf",
  "output_dir": "/Users/me/rhwp/pdf",
  "output_filename": "locked-2020.pdf",
  "password": "<문서_암호>",
  "timeout_seconds": 900
}
```

기타 선택 인자:

```json
{
  "clear_distribution": false,
  "table_patch_last_row_count": false,
  "disable_pdf_hwpx_fallback": true,
  "allow_sibling_fallbacks": false
}
```

- `clear_distribution`: 문서 정책상 허용된 경우에만 distribution flag를 제거한 임시 입력으로 재시도한다.
- `table_patch_last_row_count`: 특정 표 메타데이터 오류가 재현되는 입력에서만 사용한다.
- `disable_pdf_hwpx_fallback`: 기본값은 `true`이며 direct PDF 변환 경로를 유지한다.
- `allow_sibling_fallbacks`: 기본값은 `false`이며 사용자가 지정한 입력 바이트만 변환한다.

## rhwp PR 리뷰 출력 저장 규칙

PR 리뷰나 시각 검증 기준 PDF는 `output/`에만 두지 않는다. 50MB 미만이면 저장소의 `pdf/` 아래에
입력 샘플의 하위 구조를 유지해 저장한다.

```text
samples/task2097/1730000_selection_report.hwp
pdf/task2097/1730000_selection_report-2020.pdf
```

## 성공 확인

변환 결과에서 다음을 확인한다.

- `status: success`
- `server.run_status: 0`
- `server.validation: ok`
- 출력 PDF가 로컬 `output_dir`에 존재
- 용지/방향을 지정했다면 PageSetup metadata가 요청한 `paper_size`, `orientation`, `landscape` 값과 일치
- `pdfinfo`의 `Page size`와 `Pages`가 기대값과 일치

```bash
pdfinfo /Users/me/rhwp/pdf/example-a3-2020.pdf | rg '^(Pages|Page size):'
file /Users/me/rhwp/pdf/example-a3-2020.pdf
shasum -a 256 /Users/me/rhwp/pdf/example-a3-2020.pdf
```

## 문제 해결

- client archive가 `tools/hwp-convert-mcp-client-20260728-215936.tar.gz` 하나만 있는지 확인한다.
- `--package=file:/.../tools/hwp-convert-mcp-client-20260728-215936.tar.gz`처럼 `file:` 스킴과
  절대경로를 사용한다.
- `/opt/homebrew/bin/npx`가 실제 `npx` 경로와 다르면 `mcp.json`의 `command`를 `which npx` 결과로
  바꾼다.
- `--env-file`과 `envFile`은 절대경로를 사용한다.
- `.env.local`에 `HWP2020_MCP_SERVER_URL`, `HWP2020_MCP_AUTH_TOKEN`이 있는지 확인한다.
- 서버 URL은 `/mcp` endpoint까지 포함해야 한다.
- `--env-file`을 썼는데 `--server-url is required`가 나오면 `.env.local`에
  `HWP2020_MCP_SERVER_URL`을 추가하거나 `--server-url`을 직접 지정한다.
- `--input`은 client 로컬 파일이어야 하고, `--output-dir` 상위 경로에는 쓰기 권한이 있어야 한다.
- 큰 문서는 `--timeout-seconds`를 900~1800초로 늘린다.
- VS Code의 `MCP: Reset Cached Tools` 명령이 없으면 `Developer: Reload Window` 또는 재시작을 사용한다.
