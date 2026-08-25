---
kind: guide
status: active
canonical: mydocs/manual/mcp_hwp2024Convert_usage.md
last_verified: 2026-08-24
---

# HWP 2024 변환 MCP client 사용법

이 문서는 macOS, Linux, Windows 작업 PC의 HWP/HWPX 파일을 원격 Windows Hancom HTTP MCP
service로 보내고 변환 결과를 다시 같은 client PC에 저장하는 방법을 설명한다. 변환 service는
Windows에서 HOffice120과 HOffice130 profile을 선택해 실행하지만 client OS에는 종속되지 않는다.

한컴오피스 2022 이하 저장본은 `engine: 2020`, 한컴오피스 2024 저장본은 `engine: 2024`를 사용한다.
`2020`은 이 service에 등록된 Windows HOffice120 호환 profile의 공개 식별자이며 폐기된 Linux 한컴
2020 beta service를 호출한다는 뜻이 아니다. 확장자만으로 engine을 선택하지 않는다.
별도 “HWP 2020 MCP” server나 client artifact는 제공하지 않으며, 이 client의 `engine: 2020`만 사용한다.
`rhwp info --json <파일>`의 `lastSavedWith.product`가 `hancom-office-2024`이면 `2024`,
`hancom-office-2022`이면 `2020`을 사용하고, 필드가 `null`이면 저장 환경을 별도로 확인한다.

## 개요

- MCP server 이름: `hwp2024Convert`
- 권장 실행 방식: `hwp2024-mcp-convert` CLI의 비동기 `start → status → download` 흐름.
  다쪽 문서, 변환 시간이 긴 문서와 PR review 기준 PDF는 항상 이 흐름을 사용한다.
- 동기 `hwp2024-mcp-convert convert` 호출은 소형 문서의 즉시 변환 확인에만 사용한다.
- VS Code 연동 방식: stdio MCP bridge `hwp2024-mcp-bridge`. 실제 변환은 비동기 tool을 우선한다.
- 지원 입력: `.hwp`, `.hwpx`
- 지원 출력: `pdf`, `hwp`, `hwpx`
- 지원 방향: `.hwp → pdf|hwp|hwpx`, `.hwpx → pdf|hwp`
- 지원 engine: `2020`(HOffice120 호환 profile), `2024`(HOffice130), 기본값 `2024`
- 선택적 암호 문서: MCP tool의 `password` 또는 CLI의 비공개 `--password-file`
- 동기 tool: `convert_local_document`
- 비동기 tool: `start_local_document_conversion`, `get_local_conversion_status`,
  `save_local_conversion_result`
- 지원 client 환경: Node.js 22 이상의 macOS, Linux, Windows PowerShell. Windows `cmd.exe`를 쓸 때는
  PowerShell 변수 대신 실제 `npx.cmd`, archive, `.env.local` 경로를 직접 넣고 같은 인자를 사용한다.
- client npm runtime dependency: 없음

이 client는 변환 엔진이 아니다. 모든 변환은 지정된 원격 HWP 2024 MCP server에서 수행되며 client는
서버 연결 없이 단독으로 변환할 수 없다. client가 local input을 Base64 blob으로 업로드하고 결과
resource blob의 byte 수와 SHA-256을 검증한 뒤 local output directory에 저장한다. client와 server가
같은 파일 경로나 공유 폴더를 사용할 필요는 없다.

server URL/IP, bearer token과 `.env.local` 내용은 Git, issue, PR, 공개 문서와 로그에 기록하지 않는다.
이 서비스는 rhwp maintainer, collaborator 또는 MCP 관리자가 별도로 인증한 사용자만 사용할 수 있다.
접근 정보는 MCP 관리자에게 비공개 경로로 전달받는다.

## 최신 client artifact

rhwp에는 HWP 2024용 최신 client artifact 하나를 유지한다.

```text
tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz
```

| 항목 | 값 |
| --- | --- |
| package version | `0.9.0` |
| archive bytes | 8,558 |
| archive SHA-256 | `830b1f7ff3696a9b499d0043e48e3dccfba7817797a698003bd909226f13cf72` |
| 외부 npm runtime dependency | 없음 |
| 포함 명령 | `hwp2024-mcp-bridge`, `hwp2024-mcp-convert` |

archive에 한컴 실행 파일이나 변환 DLL은 포함하지 않는다. 한컴 runtime은 MCP server에 설치되어 있어야
하며 client archive는 HTTP MCP 호출과 blob 처리만 담당한다.

## 환경 파일과 client 준비

client archive는 rhwp 저장소의 `tools/` 아래에 두고, endpoint와 token은 Git 밖의 local client
directory의 `$HOME/hwp-convert-2024/.env.local`에만 둔다. macOS와 Linux를 포함한 POSIX host는
같은 기본 경로를 사용하며, 저장소 안에 `.env.local`을 만들지 않는다.

| client OS | rhwp archive | 비공개 환경 파일 |
| --- | --- | --- |
| macOS | `$HOME/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz` | `$HOME/hwp-convert-2024/.env.local` |
| Linux | `$HOME/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz` | `$HOME/hwp-convert-2024/.env.local` |
| Windows | `C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260824-011002.tar.gz` | `C:\Users\<사용자>\hwp-convert-2024\.env.local` |

```env
HWP2024_MCP_SERVER_URL=http://<관리자가_제공한_MCP_endpoint>
HWP2024_MCP_AUTH_TOKEN=<관리자가_제공한_32_byte_이상_token>
```

POSIX host에서 새 환경 파일을 만들 때는 먼저 권한을 제한한다.

```bash
export HWP2024_MCP_ENV_DIR="$HOME/hwp-convert-2024"
mkdir -p "$HWP2024_MCP_ENV_DIR"
umask 077
${EDITOR:-vi} "$HWP2024_MCP_ENV_DIR/.env.local"
chmod 600 "$HWP2024_MCP_ENV_DIR/.env.local"
```

Windows에서는 `.env.local`을 현재 사용자만 읽을 수 있는 `hwp-convert-2024` directory에 두고,
사용자 범위의 Git 무시 규칙으로도 노출되지 않도록 한다. endpoint는 `/mcp`까지 포함한다. HTTP bearer
token은 전송 구간 암호화를 제공하지 않으므로 신뢰할 수 있는 내부망에서만 사용하고, 신뢰할 수 없는 망에서는
HTTPS reverse proxy 뒤에 둔다.

암호 문서를 CLI로 변환할 때는 문서 암호를 command line 인자에 직접 넣지 않는다. Git 밖의 현재 사용자만
읽을 수 있는 단일 행 파일을 만들고 `convert` 또는 `start`에 `--password-file <경로>`를 지정한다.
MCP tool에서는 요청의 선택적 `password` 필드로만 전달한다. client는 암호를 결과 JSON, job 상태,
파일명 또는 로그에 기록하지 않는다.

## 터미널 CLI

macOS·Linux에서는 `npx`의 절대 경로를 확인한다. macOS GUI VS Code는 shell `PATH`를 상속하지 않을 수
있으므로 VS Code 등록 시에도 이 경로를 사용한다.

```bash
command -v npx
# macOS Homebrew 예: /opt/homebrew/bin/npx
# Ubuntu 예: /usr/bin/npx

export HWP2024_MCP_PACKAGE="$HOME/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz"
export HWP2024_MCP_ENV_FILE="$HOME/hwp-convert-2024/.env.local"
```

Windows PowerShell에서는 다음 변수로 경로를 한 번만 정의한다.

```powershell
$Npx = 'C:\Program Files\nodejs\npx.cmd'
$Package = 'C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260824-011002.tar.gz'
$EnvFile = 'C:\Users\<사용자>\hwp-convert-2024\.env.local'
& $Npx --version
```

실제 문서 변환은 비동기 `start → status → download`를 기본으로 사용한다. 아래 동기 예제는 설치 확인이나
수 초 안에 끝나는 소형 문서 smoke test에만 사용한다. `--input`과 `--output-dir`은 모두 client PC의
local 경로다.

도움말 (macOS/Linux):

```bash
npx -y --package="file:$HWP2024_MCP_PACKAGE" -- hwp2024-mcp-convert --help
```

도움말 (Windows PowerShell):

```powershell
& $Npx -y `
  "--package=file:$Package" `
  -- hwp2024-mcp-convert --help
```

### 소형 문서 확인 전용: 동기 변환

작은 문서는 `convert`가 upload, 원격 변환, download, SHA-256 검증과 local 저장을 한 번에 수행한다.

macOS/Linux:

```bash
npx -y --package="file:$HWP2024_MCP_PACKAGE" -- hwp2024-mcp-convert convert \
  --env-file "$HWP2024_MCP_ENV_FILE" \
  --input "$HOME/rhwp/samples/example.hwp" \
  --target pdf \
  --engine 2024 \
  --output-dir "$HOME/rhwp/pdf" \
  --output-filename example-2024.pdf \
  --timeout-seconds 900
```

Windows PowerShell:

```powershell
& $Npx -y `
  "--package=file:$Package" `
  -- hwp2024-mcp-convert convert `
  --env-file $EnvFile `
  --input 'C:\Users\<사용자>\rhwp\samples\example.hwp' `
  --target pdf `
  --engine 2024 `
  --output-dir 'C:\Users\<사용자>\rhwp\pdf' `
  --output-filename 'example-2024.pdf' `
  --timeout-seconds 900
```

성공하면 `status`, `output_path`, `target`, `size`, `sha256`와 server 변환 metadata가 JSON으로
출력된다. 기존 output file은 덮어쓰지 않는다.

### 권장: 비동기 변환

큰 문서, 변환 시간이 긴 문서와 PR review 기준 PDF는 `start → status → download` 순서로 처리한다.

macOS/Linux:

```bash
# 1. local input upload와 remote job 시작
npx -y --package="file:$HWP2024_MCP_PACKAGE" -- hwp2024-mcp-convert start \
  --env-file "$HWP2024_MCP_ENV_FILE" \
  --input "$HOME/rhwp/samples/large-manual.hwpx" \
  --target pdf \
  --engine 2024 \
  --output-filename large-manual-2024.pdf \
  --timeout-seconds 1800

# 2. terminal=true가 될 때까지 상태 확인
npx -y --package="file:$HWP2024_MCP_PACKAGE" -- hwp2024-mcp-convert status \
  --env-file "$HWP2024_MCP_ENV_FILE" \
  --job-id <start가_반환한_UUID>

# 3. succeeded 뒤 result blob 검증과 local 저장
npx -y --package="file:$HWP2024_MCP_PACKAGE" -- hwp2024-mcp-convert download \
  --env-file "$HWP2024_MCP_ENV_FILE" \
  --job-id <start가_반환한_UUID> \
  --output-dir "$HOME/rhwp/pdf"
```

Windows PowerShell:

```powershell
# 1. local input upload와 remote job 시작
& $Npx -y `
  "--package=file:$Package" `
  -- hwp2024-mcp-convert start `
  --env-file $EnvFile `
  --input 'C:\Users\<사용자>\rhwp\samples\large-manual.hwpx' `
  --target pdf `
  --engine 2024 `
  --output-filename 'large-manual-2024.pdf' `
  --timeout-seconds 1800

# 2. terminal=true가 될 때까지 상태 확인
& $Npx -y `
  "--package=file:$Package" `
  -- hwp2024-mcp-convert status `
  --env-file $EnvFile `
  --job-id <start가_반환한_UUID>

# 3. succeeded 뒤 result blob 검증과 local 저장
& $Npx -y `
  "--package=file:$Package" `
  -- hwp2024-mcp-convert download `
  --env-file $EnvFile `
  --job-id <start가_반환한_UUID> `
  --output-dir 'C:\Users\<사용자>\rhwp\pdf'
```

status의 terminal 상태는 `succeeded`, `failed`, `expired`다. `succeeded`일 때만 download한다.
비동기 start에는 local `output_dir`을 전달하지 않으며 download 단계에서 지정한다.

## VS Code MCP 등록

VS Code Chat에서 tool을 사용할 때만 stdio bridge를 등록한다. terminal에서 bridge를 미리 실행해 두지
않는다. workspace의 `.vscode/mcp.json` 또는 VS Code user MCP 설정에서 client OS에 맞는 하나만 사용한다.

### macOS

macOS GUI VS Code에서는 `command -v npx`의 절대 경로를 `command`에 쓰고, Homebrew 경로를 `PATH`에
포함한다.

```json
{
  "servers": {
    "hwp2024Convert": {
      "type": "stdio",
      "command": "/opt/homebrew/bin/npx",
      "args": [
        "-y",
        "--package=file:${userHome}/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz",
        "--",
        "hwp2024-mcp-bridge"
      ],
      "envFile": "${userHome}/hwp-convert-2024/.env.local",
      "env": {
        "PATH": "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
      }
    }
  }
}
```

Intel Mac에서 Node.js가 `/usr/local/bin/npx`에 설치됐거나 다른 경로가 나오면 `command`와 `PATH`를
`command -v npx`의 결과로 바꾼다.

### Linux

Linux에서 Node.js가 system package로 설치됐다면 보통 `/usr/bin/npx`를 사용한다. nvm 등으로 설치했다면
`command -v npx`의 절대 경로로 교체한다.

```json
{
  "servers": {
    "hwp2024Convert": {
      "type": "stdio",
      "command": "/usr/bin/npx",
      "args": [
        "-y",
        "--package=file:${userHome}/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz",
        "--",
        "hwp2024-mcp-bridge"
      ],
      "envFile": "${userHome}/hwp-convert-2024/.env.local"
    }
  }
}
```

### Windows PowerShell

```json
{
  "servers": {
    "hwp2024Convert": {
      "type": "stdio",
      "command": "C:\\Program Files\\nodejs\\npx.cmd",
      "args": [
        "-y",
        "--package=file:${userHome}/rhwp/tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz",
        "--",
        "hwp2024-mcp-bridge"
      ],
      "envFile": "${userHome}/hwp-convert-2024/.env.local"
    }
  }
}
```

VS Code MCP 접근이 차단되어 있으면 user settings에 다음을 추가한다.

```json
{
  "chat.mcp.access": "all"
}
```

설정 후 `Developer: Reload Window` 또는 VS Code 재시작을 수행한다. `MCP: List Servers`에서
`hwp2024Convert`와 4개 tool이 보여야 한다.

## MCP tool 사용

### 동기

macOS/Linux 경로 예시:

```json
{
  "input_path": "/home/<사용자>/rhwp/samples/example.hwp",
  "target": "pdf",
  "engine": "2024",
  "output_dir": "/home/<사용자>/rhwp/pdf",
  "output_filename": "example-2024.pdf",
  "timeout_seconds": 900
}
```

Windows 경로 예시:

```json
{
  "input_path": "C:\\Users\\<사용자>\\rhwp\\samples\\example.hwp",
  "target": "pdf",
  "engine": "2024",
  "output_dir": "C:\\Users\\<사용자>\\rhwp\\pdf",
  "output_filename": "example-2024.pdf",
  "timeout_seconds": 900
}
```

### 비동기 시작

macOS/Linux 경로 예시:

```json
{
  "input_path": "/home/<사용자>/rhwp/samples/large-manual.hwpx",
  "target": "pdf",
  "engine": "2024",
  "output_filename": "large-manual-2024.pdf",
  "timeout_seconds": 1800
}
```

Windows 경로 예시:

```json
{
  "input_path": "C:\\Users\\<사용자>\\rhwp\\samples\\large-manual.hwpx",
  "target": "pdf",
  "engine": "2024",
  "output_filename": "large-manual-2024.pdf",
  "timeout_seconds": 1800
}
```

상태 확인:

```json
{
  "job_id": "<start가_반환한_UUID>"
}
```

결과 저장 (macOS/Linux 경로 예시):

```json
{
  "job_id": "<start가_반환한_UUID>",
  "output_dir": "/home/<사용자>/rhwp/pdf",
  "output_filename": "large-manual-2024.pdf"
}
```

결과 저장 (Windows 경로 예시):

```json
{
  "job_id": "<start가_반환한_UUID>",
  "output_dir": "C:\\Users\\<사용자>\\rhwp\\pdf",
  "output_filename": "large-manual-2024.pdf"
}
```

`input_path`와 `output_dir`은 모두 client PC의 경로다. POSIX host는 `/home/<사용자>/...` 또는
`/Users/<사용자>/...`, Windows는 `C:\Users\<사용자>\...`를 쓰며 server 내부 경로를 입력하지 않는다.
`output_filename`은 경로가 없는 파일명이어야 하며 target 확장자와 일치해야 한다.

암호 문서는 동기 또는 비동기 시작 요청에만 `password`를 추가한다. 상태 확인과 결과 저장 요청에는
암호를 다시 넣지 않는다.

```json
{
  "input_path": "/home/<사용자>/rhwp/samples/protected.hwp",
  "target": "pdf",
  "engine": "2020",
  "password": "<비공개_문서_암호>",
  "output_filename": "protected-hwp2020.pdf",
  "timeout_seconds": 900
}
```

## 동시 호출과 timeout

HTTP MCP server는 여러 client session을 받을 수 있지만 한컴 runtime 안정성을 위해 실제 변환은
server process 전체에서 하나씩 직렬 실행한다. 비동기 요청은 기본 8개까지 대기열에 들어간다.

`timeout_seconds` 허용 범위는 10~1800초이고 기본값은 600초다. 일반 문서는 600~900초, 큰 문서·이미지가
많은 문서·거대 표·중첩 표는 1800초를 권장한다. client의 동기 HTTP request는 변환 timeout에 120초
여유를 더해 기다린다.

## 성공 확인

변환 결과에서 다음을 확인한다.

- `status: success`
- `engine`: `start`와 `status` 응답에서 요청한 engine과 일치해야 한다. 2022 이하 저장본은
  `2020`, 2024 저장본은 `2024`를 명시한다.
- `server.engine`: 서버가 반환할 수 있는 concrete backend 식별자다. 요청한 engine과의 일치 여부를
  판정하는 값으로 사용하지 않는다. 현재 배포의 2024 backend는
  `hancom-2024-direct-host`를 반환한다.
- `server.engine_profile`, `server.hancom_version`: 서버가 제공하면 선택한 runtime의 추가 증적으로
  기록한다. 이 필드는 현재 배포 응답에 없을 수 있으므로 부재만으로 변환을 실패로 판단하지 않는다.
- `server.backend: hwp-managed-direct-dll-host`
- `server.worker_bits: 32`
- client가 보고한 `size`, `sha256`과 local file이 일치
- PDF는 `%PDF-` header와 `%%EOF`, HWPX는 ZIP signature, HWP는 CFB signature가 정상

macOS/Linux:

```bash
ls -lh "$HOME/rhwp/pdf/example-2024.pdf"
if command -v sha256sum >/dev/null; then
  sha256sum "$HOME/rhwp/pdf/example-2024.pdf"
else
  shasum -a 256 "$HOME/rhwp/pdf/example-2024.pdf"
fi
file "$HOME/rhwp/pdf/example-2024.pdf"
```

Windows PowerShell:

```powershell
Get-Item -LiteralPath 'C:\Users\<사용자>\rhwp\pdf\example-2024.pdf'
Get-FileHash -Algorithm SHA256 -LiteralPath 'C:\Users\<사용자>\rhwp\pdf\example-2024.pdf'
```

## artifact 검증 기준선

2026-08-24 생성 artifact는 현재 source 계약을 사용하는 local mock HTTP MCP에서 다음을 확인했다.

- package `0.9.0`, archive 8,558바이트, SHA-256 `830b1f7ff3696a9b499d0043e48e3dccfba7817797a698003bd909226f13cf72`
- tarball에서 `hwp2024-mcp-convert --help` 실행 성공
- stdio initialize와 tool discovery 성공, tool 4개
- archive 내 `node_modules` 0개, runtime dependency 0개, 외부 runtime import 0개
- tool schema에 `engine: 2020|2024`와 선택적 `password` 존재
- 실제 archive CLI의 비동기 `start`가 engine `2020`, 엔진별 기본 파일명, 비공개 암호 파일을 remote argument로 전달
- 결과 JSON에는 `status`, `job_id`, `engine`만 있고 암호 값은 없음

2026-08-24 실제 배포 MCP server에는 최신 artifact로 engine `2024`의 작은 HWPX를 동기 `convert`와
비동기 `start → status → download`로 각각 변환했다. 두 경로 모두 `success`, client/server SHA-256 일치와
PDF 서명을 확인했고, 비동기 `start`와 `status`의 `engine`은 요청값 `2024`와 일치했다. 또한 Hancom Office
2020 저장본 `kps-ai.hwp`는 `--engine 2020`을 명시한 비동기 흐름에서 `queued → succeeded → success`,
`start`·`status`의 `engine: "2020"`, PDF 서명 및 SHA-256 일치를 확인했다. `server.engine`은
`hancom-2024-direct-host` backend 식별자를 반환했고 `server.engine_profile`과 `server.hancom_version`은
반환하지 않았다.

2026-08-22의 이전 `0.8.0` artifact는 당시 실제 배포 MCP server에 연결해 다음을 확인했다.

- tarball에서 `hwp2024-mcp-convert --help` 실행 성공
- stdio initialize와 tool discovery 성공, tool 4개
- archive 내 `node_modules` 0개, runtime dependency import 0개
- 동기 HWP→HWPX: `success`, output 67,709 bytes
- 비동기 HWP→PDF: `queued → succeeded → success`, output 106,341 bytes
- 두 경로 모두 client/server output byte 수와 SHA-256 일치
- server engine `hancom-2024-direct-host`, backend `hwp-managed-direct-dll-host`, worker 32-bit

실제 server 주소와 token은 검증 기록에 포함하지 않았다.

## 문제 해결

- `HWP2024_MCP_AUTH_TOKEN or --auth-token is required`
  - `.env.local`에 `HWP2024_MCP_AUTH_TOKEN`이 있고 `envFile` 또는 `--env-file` 경로가 맞는지 확인한다.
- `HTTP MCP request failed with status 401`
  - token이 service config의 현재 token과 일치하는지 관리자에게 확인한다. token 자체를 로그에 출력하지 않는다.
- `HTTP MCP request failed with status 4xx/5xx`
  - endpoint가 `/mcp`까지 포함하는지, service health와 방화벽을 관리자가 확인한다.
- `input_path does not exist`
  - server 경로가 아니라 MCP client가 실행되는 PC의 local path를 사용한다.
- `unsupported target 'hwpx' for .hwpx input`
  - `.hwpx` 입력은 `pdf` 또는 `hwp`로 변환한다.
- 기존 파일 오류
  - client는 output을 덮어쓰지 않는다. 다른 `output_filename`을 사용하거나 기존 파일을 별도 보관한 뒤 재시도한다.
- 큰 문서 timeout
  - `timeout_seconds: 1800`과 비동기 start/status/download 흐름을 사용한다.
