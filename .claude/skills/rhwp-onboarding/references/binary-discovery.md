# 바이너리 발견 — PATH에서 cargo bin까지

닥터는 rhwp 를 **실행 가능한 파일**로만 찾는다. 설치 프로그램이나
네트워크 fetch 를 하지 않는다. 못 찾으면 빌드 명령만 찍고 종료 코드 3.

## 탐색 순서

1. `--rhwp <경로>` — 있으면 이것만 본다. 없으면 `--rhwp(미발견)`.
2. 환경 변수 `RHWP_BIN` 또는 `RHWP` (PATH/release 가 없을 때).
3. `PATH` 의 `rhwp` / `rhwp.exe` (`shutil.which`).
4. `<repo>/target/release/rhwp[.exe]`.
5. `<repo>/target/debug/rhwp[.exe]` (release 가 없을 때만).
6. `$CARGO_HOME/bin` 또는 `~/.cargo/bin`.

호환: 예전 닥터는 override → PATH → release 만 봤다. 그 세 자리의
우선순위는 그대로다. 확장 자리는 앞자리가 비었을 때만 쓴다.

## 손으로 같은 순서

```bash
rhwp --version
echo "$RHWP_BIN"
ls target/release/rhwp target/release/rhwp.exe
ls target/debug/rhwp target/debug/rhwp.exe
ls "${CARGO_HOME:-$HOME/.cargo}/bin/rhwp"
```

PowerShell:

```powershell
Get-Command rhwp -ErrorAction SilentlyContinue
$env:RHWP_BIN
Get-Item target\release\rhwp.exe, target\debug\rhwp.exe -ErrorAction SilentlyContinue
Get-Item "$env:USERPROFILE\.cargo\bin\rhwp.exe" -ErrorAction SilentlyContinue
```

## 빌드 (닥터가 대신 하지 않음)

```bash
cargo build --release --bin rhwp
# 산출: target/release/rhwp   (Windows: target\release\rhwp.exe)
```

- 네이티브 빌드는 로컬 cargo. Docker 는 WASM 전용.
- 오프라인에서 의존성 캐시가 있으면 빌드는 된다.
- 캐시가 없으면 [exception-no-network.md](exception-no-network.md).

## 닥터 리포트 칸

| 필드 | 뜻 |
|---|---|
| `binary.found` | 실행 파일을 골랐는가 |
| `binary.path` | 고른 절대 경로 |
| `binary.source` | `--rhwp` / `PATH` / `target/release` / `target/debug` / `RHWP_BIN` / `cargo-bin` |
| `binary.onPath` | PATH 히트면 true. 스니펫 command 가 `rhwp` 가 된다 |
| `binary.version` | `--version` 첫 줄 |
| `binaryInventory[]` | 모든 자리의 hit/miss |

`onPath==false` 이면 `.mcp.json` 의 `command` 는 절대 경로다.

## 플랫폼

| OS | 파일명 | 기본 산출 | 확인 |
|---|---|---|---|
| Windows | `rhwp.exe` | `target\release\rhwp.exe` | Get-Command, $env:Path |
| Linux | `rhwp` | `target/release/rhwp` | which rhwp, ~/.cargo/bin |
| macOS | `rhwp` | `target/release/rhwp` | which rhwp, ~/.cargo/bin |

## 실패 모드

### 아무 자리에도 없음

exit 3, `missing_binary`. 빌드 안내.

### `--rhwp` 가 잘못된 경로

다른 자리를 보지 않는다. 경로를 고친다.

### `--rhwp` 가 디렉터리

파일이 아니므로 미발견.

### PATH 의 낡은 설치본

PATH 가 release 보다 앞선다. `--rhwp` 로 저장소 산출을 짚는다.

### debug 만 있음

확장 탐색이 debug 를 고른다. 느릴 수 있다. release 를 권장.

### 실행 권한 없음 (Unix)

`OSError` 로 version 검사 FAIL. chmod +x.

### MZ/ELF 가 아닌 텍스트

실행 시 OSError. 잘못 내려받은 파일.

### WSL vs Windows 경로

한 쪽에서 빌드한 exe 를 다른 쪽에서 실행하지 않는다.

### msvc 런타임

Windows 에서 실행 불가 메시지면 빌드 도구를 본다. 닥터가 설치하지 않는다.

### 여러 워크트리

`--repo-root` 로 이 워크트리의 `target/` 을 가리킨다.

## 환경 변수

| 이름 | 역할 |
|---|---|
| `RHWP_BIN` | 바이너리 절대 경로. PATH/release 미스 후 사용 |
| `RHWP` | `RHWP_BIN` 별칭 |
| `CARGO_HOME` | cargo bin 자리 |

설치 스크립트용 변수를 새로 만들지 않는다.

## 성공 판정

```text
rhwp --version     → exit 0, 비어 있지 않은 한 줄
doctor --json      → binary.found==true, checks.version==PASS
```

다음 실패는 [exception-missing-binary.md](exception-missing-binary.md).
