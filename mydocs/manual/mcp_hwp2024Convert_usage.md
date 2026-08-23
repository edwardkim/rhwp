---
kind: guide
status: active
canonical: mydocs/manual/mcp_hwp2024Convert_usage.md
last_verified: 2026-08-23
---

# HWP 2024 변환 MCP client 사용법

이 문서는 rhwp 작업 PC의 HWP/HWPX 파일을 원격 Windows Hancom Office 2024 HTTP MCP server로
보내고 변환 결과를 다시 client PC에 저장하는 방법을 설명한다.

한컴오피스 2024에서 저장한 `.hwp`/`.hwpx`는 이 `hwp-convert-2024` 서비스를 사용한다.
저장한 한컴오피스 버전이 2022 이하인 `.hwp`/`.hwpx`는 이 서비스가 아니라
[HWP 2020 MCP 사용법](mcp_hwp2020Convert_usage.md)의 `hwp-convert-2020` 서비스를 사용한다.
확장자만으로 서비스를 선택하지 않는다. `rhwp info --json <파일>`의 `lastSavedWith.product`가
`hancom-office-2024`이면 이 서비스를 사용하고, 필드가 `null`이면 저장 환경을 별도로 확인한다.

## 개요

- MCP server 이름: `hwp2024Convert`
- 권장 실행 방식: `hwp2024-mcp-convert` CLI의 비동기 `start → status → download` 흐름.
  다쪽 문서, 변환 시간이 긴 문서와 PR review 기준 PDF는 항상 이 흐름을 사용한다.
- 동기 `hwp2024-mcp-convert convert` 호출은 소형 문서의 즉시 변환 확인에만 사용한다.
- VS Code 연동 방식: stdio MCP bridge `hwp2024-mcp-bridge`. 실제 변환은 비동기 tool을 우선한다.
- 지원 입력: `.hwp`, `.hwpx`
- 지원 출력: `pdf`, `hwp`, `hwpx`
- 지원 방향: `.hwp → pdf|hwp|hwpx`, `.hwpx → pdf|hwp`
- 동기 tool: `convert_local_document`
- 비동기 tool: `start_local_document_conversion`, `get_local_conversion_status`,
  `save_local_conversion_result`
- 요구 Node.js: 22 이상
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
tools/hwp-convert-mcp-2024-client-20260822-225818.tar.gz
```

| 항목 | 값 |
| --- | --- |
| package version | `0.8.0` |
| archive bytes | 7,673 |
| archive SHA-256 | `55b75bb002818a42a18f0f289f3f8d669ac63af7185d2f6eece062ab326b120c` |
| 외부 npm runtime dependency | 없음 |
| 포함 명령 | `hwp2024-mcp-bridge`, `hwp2024-mcp-convert` |

archive에 한컴 실행 파일이나 변환 DLL은 포함하지 않는다. 한컴 runtime은 MCP server에 설치되어 있어야
하며 client archive는 HTTP MCP 호출과 blob 처리만 담당한다.

## 환경 파일 준비

Git 밖의 비공개 client directory에 `.env.local`을 만든다.

```env
HWP2024_MCP_SERVER_URL=http://<관리자가_제공한_MCP_endpoint>
HWP2024_MCP_AUTH_TOKEN=<관리자가_제공한_32_byte_이상_token>
```

예시 배치:

```text
C:\Users\<사용자>\rhwp\
  tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz

C:\Users\<사용자>\hwp-convert-2024-client\
  .env.local
```

endpoint는 `/mcp`까지 포함한다. HTTP bearer token은 전송 구간 암호화를 제공하지 않으므로 신뢰할 수
있는 내부망에서만 사용하고, 신뢰할 수 없는 망에서는 HTTPS reverse proxy 뒤에 둔다.

## 터미널 CLI

실제 문서 변환은 비동기 `start → status → download`를 기본으로 사용한다. 아래 동기 예제는 설치 확인이나
수 초 안에 끝나는 소형 문서 smoke test에만 사용한다.

도움말:

```powershell
& 'C:\Program Files\nodejs\npx.cmd' -y `
  --package='file:C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz' `
  -- hwp2024-mcp-convert --help
```

### 소형 문서 확인 전용: 동기 변환

작은 문서는 `convert`가 upload, 원격 변환, download, SHA-256 검증과 local 저장을 한 번에 수행한다.

```powershell
& 'C:\Program Files\nodejs\npx.cmd' -y `
  --package='file:C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz' `
  -- hwp2024-mcp-convert convert `
  --env-file 'C:\Users\<사용자>\hwp-convert-2024-client\.env.local' `
  --input 'C:\Users\<사용자>\rhwp\samples\example.hwp' `
  --target pdf `
  --output-dir 'C:\Users\<사용자>\rhwp\pdf' `
  --output-filename 'example-2024.pdf' `
  --timeout-seconds 900
```

성공하면 `status`, `output_path`, `target`, `size`, `sha256`와 server 변환 metadata가 JSON으로
출력된다. 기존 output file은 덮어쓰지 않는다.

### 권장: 비동기 변환

큰 문서, 변환 시간이 긴 문서와 PR review 기준 PDF는 `start → status → download` 순서로 처리한다.

```powershell
# 1. local input upload와 remote job 시작
& 'C:\Program Files\nodejs\npx.cmd' -y `
  --package='file:C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz' `
  -- hwp2024-mcp-convert start `
  --env-file 'C:\Users\<사용자>\hwp-convert-2024-client\.env.local' `
  --input 'C:\Users\<사용자>\rhwp\samples\large-manual.hwpx' `
  --target pdf `
  --output-filename 'large-manual-2024.pdf' `
  --timeout-seconds 1800

# 2. terminal=true가 될 때까지 상태 확인
& 'C:\Program Files\nodejs\npx.cmd' -y `
  --package='file:C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz' `
  -- hwp2024-mcp-convert status `
  --env-file 'C:\Users\<사용자>\hwp-convert-2024-client\.env.local' `
  --job-id <start가_반환한_UUID>

# 3. succeeded 뒤 result blob 검증과 local 저장
& 'C:\Program Files\nodejs\npx.cmd' -y `
  --package='file:C:\Users\<사용자>\rhwp\tools\hwp-convert-mcp-2024-client-20260822-225818.tar.gz' `
  -- hwp2024-mcp-convert download `
  --env-file 'C:\Users\<사용자>\hwp-convert-2024-client\.env.local' `
  --job-id <start가_반환한_UUID> `
  --output-dir 'C:\Users\<사용자>\rhwp\pdf'
```

status의 terminal 상태는 `succeeded`, `failed`, `expired`다. `succeeded`일 때만 download한다.
비동기 start에는 local `output_dir`을 전달하지 않으며 download 단계에서 지정한다.

## VS Code MCP 등록

워크스페이스의 `.vscode/mcp.json` 또는 VS Code user MCP 설정에 다음과 같이 등록한다.

```json
{
  "servers": {
    "hwp2024Convert": {
      "type": "stdio",
      "command": "C:\\Program Files\\nodejs\\npx.cmd",
      "args": [
        "-y",
        "--package=file:${userHome}/rhwp/tools/hwp-convert-mcp-2024-client-20260822-225818.tar.gz",
        "--",
        "hwp2024-mcp-bridge"
      ],
      "envFile": "${userHome}/hwp-convert-2024-client/.env.local"
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

```json
{
  "input_path": "C:\\Users\\<사용자>\\rhwp\\samples\\example.hwp",
  "target": "pdf",
  "output_dir": "C:\\Users\\<사용자>\\rhwp\\pdf",
  "output_filename": "example-2024.pdf",
  "timeout_seconds": 900
}
```

### 비동기 시작

```json
{
  "input_path": "C:\\Users\\<사용자>\\rhwp\\samples\\large-manual.hwpx",
  "target": "pdf",
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

결과 저장:

```json
{
  "job_id": "<start가_반환한_UUID>",
  "output_dir": "C:\\Users\\<사용자>\\rhwp\\pdf",
  "output_filename": "large-manual-2024.pdf"
}
```

`input_path`와 `output_dir`은 모두 client PC의 경로다. server 내부 경로를 입력하지 않는다.
`output_filename`은 경로가 없는 파일명이어야 하며 target 확장자와 일치해야 한다.

## 동시 호출과 timeout

HTTP MCP server는 여러 client session을 받을 수 있지만 한컴 runtime 안정성을 위해 실제 변환은
server process 전체에서 하나씩 직렬 실행한다. 비동기 요청은 기본 8개까지 대기열에 들어간다.

`timeout_seconds` 허용 범위는 10~1800초이고 기본값은 600초다. 일반 문서는 600~900초, 큰 문서·이미지가
많은 문서·거대 표·중첩 표는 1800초를 권장한다. client의 동기 HTTP request는 변환 timeout에 120초
여유를 더해 기다린다.

## 성공 확인

변환 결과에서 다음을 확인한다.

- `status: success`
- `server.engine: hancom-2024-direct-host`
- `server.backend: hwp-managed-direct-dll-host`
- `server.worker_bits: 32`
- client가 보고한 `size`, `sha256`과 local file이 일치
- PDF는 `%PDF-` header와 `%%EOF`, HWPX는 ZIP signature, HWP는 CFB signature가 정상

```powershell
Get-Item -LiteralPath 'C:\Users\<사용자>\rhwp\pdf\example-2024.pdf'
Get-FileHash -Algorithm SHA256 -LiteralPath 'C:\Users\<사용자>\rhwp\pdf\example-2024.pdf'
```

## artifact 검증 기준선

2026-08-22 생성 artifact를 실제 배포 MCP server에 연결해 다음을 확인했다.

- tarball에서 `hwp2024-mcp-convert --help` 실행 성공
- stdio initialize와 tool discovery 성공, tool 4개
- archive 내 `node_modules` 0개, runtime dependency import 0개
- 동기 HWP→HWPX: `success`, output 67,709 bytes
- 비동기 HWP→PDF: `queued → succeeded → success`, output 106,341 bytes
- 두 경로 모두 client/server output byte 수와 SHA-256 일치
- engine `hancom-2024-direct-host`, backend `hwp-managed-direct-dll-host`, worker 32-bit

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
