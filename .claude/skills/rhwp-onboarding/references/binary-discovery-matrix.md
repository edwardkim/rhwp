# 바이너리 발견 행렬 — OS × 자리 × 스니펫

닥터 `binary_search_plan` / `find_binary` / `choose_mcp_command` 가 실제로 보는 자리.
추측 경로를 추가하지 않는다.

## Windows

| 순위 | source | 경로 | onPath | mcp command |
|---:|---|---|---|---|
| 0 | `--rhwp` | 사용자가 준 파일 | false | 그 절대 경로 |
| 1 | `PATH` | `Get-Command rhwp` | true | `rhwp` |
| 2 | `target/release` | `<repo>\target\release\rhwp.exe` | false | 절대 경로 |
| 3 | `RHWP_BIN` / `RHWP` | 환경 변수 파일 | false | 절대 경로 |
| 4 | `target/debug` | `<repo>\target\debug\rhwp.exe` | false | 절대 경로 |
| 5 | `cargo-bin` | `%USERPROFILE%\.cargo\bin\rhwp.exe` 또는 `%CARGO_HOME%\bin\rhwp.exe` | false | 절대 경로 |

확인 명령:

```powershell
Get-Command rhwp -ErrorAction SilentlyContinue | Format-List
$env:RHWP_BIN
Get-Item .\target\release\rhwp.exe -ErrorAction SilentlyContinue
Get-Item .\target\debug\rhwp.exe -ErrorAction SilentlyContinue
Get-Item "$env:USERPROFILE\.cargo\bin\rhwp.exe" -ErrorAction SilentlyContinue
```

빌드:

```powershell
cargo build --release --bin rhwp
.\target\release\rhwp.exe --version
```

MCP JSON (PATH 없음):

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "C:\\src\\rhwp\\target\\release\\rhwp.exe",
      "args": ["mcp-serve"]
    }
  }
}
```

주의:

- `rhwp` 와 `rhwp.exe` 를 다른 도구로 취급하지 않는다. 닥터 `_exe_name()` 이 OS 를 본다.
- PowerShell 의 `rhwp` 별칭이 배치 파일을 가리키면 PATH 히트다. 저장소 산출을 쓰려면 `--rhwp`.
- WSL 에서 빌드한 ELF 를 Windows 호스트 command 에 넣지 않는다.

## Linux

| 순위 | source | 경로 |
|---:|---|---|
| 0 | `--rhwp` | 사용자 파일 |
| 1 | `PATH` | `which rhwp` |
| 2 | `target/release` | `<repo>/target/release/rhwp` |
| 3 | env | `$RHWP_BIN` |
| 4 | `target/debug` | `<repo>/target/debug/rhwp` |
| 5 | cargo-bin | `$CARGO_HOME/bin/rhwp` 또는 `~/.cargo/bin/rhwp` |

```bash
command -v rhwp
ls -l target/release/rhwp
cargo build --release --bin rhwp
chmod +x target/release/rhwp   # 빌드가 이미 켜 둠
```

실행 권한이 없으면 `check_version` 이 `OSError` 로 FAIL.

## macOS

Linux 와 같은 자리. Mach-O 시그니처는 닥터가 고르지 않는다. 실행은 OS 가 거절한다.

Apple Silicon 과 Intel 산출을 섞지 않는다. 워크트리에서 다시 빌드한다.

## 우선순위 시나리오

### A. PATH 에 낡은 0.7, 워크트리에 새 release

`find_binary` 는 PATH 를 고른다. 온보딩이 낡은 표면을 본다.
저장소 산출을 쓰려면:

```bash
python tools/agent_onboarding/rhwp_doctor.py --rhwp target/release/rhwp --json
```

### B. release 없음, debug 있음

PATH 가 없으면 debug 로 fallback. 느리다. 리포트 `source` 가 `target/debug`.
가능하면 release 를 빌드한다.

### C. `--rhwp` 가 존재하지 않음

다른 자리를 보지 않는다. `source==--rhwp(미발견)`, exit 3.

### D. `RHWP_BIN` 만 있음

PATH/release 가 없을 때 이긴다. 리포트 `source==RHWP_BIN`.

### E. 아무 것도 없음

exit 3, `missing_binary`. [exception-missing-binary.md](exception-missing-binary.md).

## 스니펫 command 결정

```text
if binary is not None and not onPath:
    command = str(binary)   # 절대 경로
else:
    command = "rhwp"
```

바이너리가 없어도 스니펫은 `rhwp` 로 방출된다. 붙이기 전에 빌드한다.

## 닥터가 보지 않는 자리

일부러 빠졌다. 추측 설치 경로를 늘리면 거짓 hit 가 생긴다.

- `C:\Program Files\rhwp\`
- `/usr/bin/rhwp` 를 특별 취급 (PATH 로만)
- npm / pip / cargo install 전역 이름 강제
- gym 러너, `rhwp-agent` 실험 바이너리
- Docker 이미지 안의 바이너리

`rhwp-agent` (`src/bin/rhwp-agent`) 는 이 온보딩의 대상이 아니다.
본 CLI `rhwp` 만 찾는다.

## 인벤토리 읽기

`--json` 의 `binaryInventory[]` 예:

```json
[
  {"source": "PATH", "kind": "which", "exists": false, "resolved": null},
  {"source": "target/release", "kind": "file", "exists": true, "resolved": "C:\\repo\\target\\release\\rhwp.exe"},
  {"source": "target/debug", "kind": "file", "exists": false, "resolved": null}
]
```

`exists:true` 인 첫 자리가 항상 선택되지는 않는다. `find_binary` 의 호환 순서
(PATH 가 release 보다 앞)를 따른다. 인벤토리는 디버그용이다.

## 빌드가 네트워크를 만나는 경우

`cargo build --release --bin rhwp` 가 crate 를 받으려 하면 오프라인에서 실패한다.
그건 닥터 exit 3 과 별개다. [exception-no-network.md](exception-no-network.md).

캐시가 있으면 오프라인 빌드가 된다. 닥터는 빌드를 시도하지 않으므로
이 실패를 `checks[]` 에 넣지 않는다.

## 버전 문자열

`rhwp --version` 이 exit 0 이고 한 줄 이상이면 PASS.
내용을 파싱해 semver 를 비교하지 않는다. 구버전은 비임계 명령이 SKIP 될 수 있다.

## 관련 테스트

- `TestBinaryDiscovery.test_find_binary_override_missing`
- `TestBinaryDiscovery.test_find_binary_release_when_no_path`
- `TestBinaryDiscovery.test_find_binary_debug_fallback`
- `TestBinaryDiscovery.test_find_binary_env_after_path_and_release_miss`
- `TestBinaryDiscovery.test_choose_mcp_command`
