# RHWP Studio Windows 설치 파일 릴리스 절차

사내 배포용 Windows 데스크톱 앱(`RHWP Studio`) 설치 파일을 만드는 절차다. 설계 배경은
[Windows 패키징 설계 스펙](../superpowers/specs/2026-08-06-rhwp-windows-packaging-design.md)을 따른다.

이 절차는 **전부 Windows 머신에서 실행한다** — Tauri의 NSIS 번들러가 Windows에서만 최종 `.exe`를
만든다. CI 자동화는 범위 밖이므로 릴리스 담당자가 아래를 순서대로 수동 실행한다.

빌드 머신이 **ARM64 Windows**(예: Apple Silicon Mac 위의 Parallels VM)라면 본문에 더해
[부록 A](#부록-a-arm64-빌드-머신에서-x64-설치-파일-만들기)를 함께 따른다 — 기본 명령만 실행하면
배포용 x64가 아니라 ARM64 설치 파일이 나오고, 일반 x64 PC에서는 실행되지 않는다.

## 0. 사전 준비 (최초 1회, 이후 재사용)

- Rust, Node.js, `wasm-pack` 설치 (저장소 `mydocs/manual/onboarding_guide.md` 참고)
- Windows SDK 설치 (`signtool.exe` 포함) — Visual Studio Installer의 "Windows 10/11 SDK" 구성요소로
  설치 가능
- LLVM/clang 설치 (`winget install LLVM.LLVM`) — `ring` 크레이트가 Windows ARM64에서 어셈블리를
  빌드할 때 clang을 요구한다. 없으면 `cargo install wasm-pack` 단계부터 실패한다. x64 빌드
  머신에서는 없어도 되지만 설치해 두어도 무방하다.
- 코드사이닝 인증서가 없다면:
  ```powershell
  scripts/windows-packaging/generate-self-signed-cert.ps1
  ```
  개인키(`.pfx`)는 사내 비밀 저장소에 보관하고, 공개 인증서(`.cer`)는 아래 위치에 복사한다.
  ```
  rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer
  ```
  인증서가 만료되기 전까지는 매 릴리스마다 새로 만들 필요 없다.

  스크립트 출력 폴더(`scripts/windows-packaging/out/`)는 `.gitignore`에 등록되어 있다 — 개인키가
  실수로 커밋되지 않도록 폴더째 제외된다. 인증서를 다른 경로에 만든다면 그 경로도 저장소 밖이거나
  ignore 대상인지 확인한다.

  암호는 반드시 기록해 둔다. 분실하면 `.pfx`를 복구할 수 없고, 인증서를 새로 만들면 `.cer`이 바뀌므로
  **3단계 빌드부터 다시 해야 한다**(`.cer`이 설치 파일 안에 리소스로 포함되기 때문).

## 1. WASM 빌드

저장소 루트에서:

```powershell
wasm-pack build --target web --out-dir pkg
```

> ARM64 빌드 머신에서는 이 명령이 마지막 최적화 단계에서 실패한다 —
> [부록 A](#부록-a-arm64-빌드-머신에서-x64-설치-파일-만들기) 참고.

## 2. 프론트엔드 데스크톱 빌드

```powershell
cd rhwp-studio
npm ci   # 최초 1회 또는 package-lock.json 변경 시
npm run build:desktop
```

`build:desktop`은 일반 웹 빌드(`npm run build`)와 달리 PWA/서비스워커 플러그인을 제외한다 — Tauri
웹뷰에는 불필요하고 캐시 충돌 위험이 있기 때문이다.

## 3. Tauri NSIS 설치 파일 빌드

`rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer`가 준비돼 있는지 확인한 뒤:

```powershell
npm run tauri:build
```

인증서가 그 자리에 없으면 컴파일도 시작하지 못하고 아래에서 멈춘다 — `tauri.conf.json`이 인증서를
번들 리소스로 선언하기 때문이다. `.cer`은 저장소에 커밋하지 않으므로(→ [0단계](#0-사전-준비-최초-1회-이후-재사용)),
새로 clone한 머신에서 특히 자주 만난다.

```
resource path `packaging\cert\rhwp-studio-cert.cer` doesn't exist
failed to build app: failed to build app
```

산출물: `rhwp-studio/src-tauri/target/release/bundle/nsis/RHWP Studio_<version>_x64-setup.exe`

`<version>`은 `rhwp-studio/src-tauri/tauri.conf.json`의 `version` 값이다 — `package.json`이 아니다.
두 값이 어긋나면 설치 파일명과 앱 정보 화면의 버전이 제품 버전과 달라지므로, 릴리스 전에 일치하는지
확인한다.

> ARM64 빌드 머신에서는 `--target`을 붙여야 하고 산출물 경로도 달라진다 —
> [부록 A](#부록-a-arm64-빌드-머신에서-x64-설치-파일-만들기) 참고.

## 4. 서명

```powershell
scripts/windows-packaging/sign-installer.ps1 `
  -InstallerPath "rhwp-studio\src-tauri\target\release\bundle\nsis\RHWP Studio_<version>_x64-setup.exe" `
  -PfxPath "<인증서 .pfx 경로>"
```

서명 후 같은 폴더에 `<설치파일명>.sha256.txt` 체크섬이 함께 생성된다.

주의할 점 두 가지:

- **암호를 빈 값으로 두지 않는다.** Windows PowerShell 5.1은 네이티브 명령에 빈 문자열 인자를 넘길 때
  그 인자를 통째로 누락시킨다. 그러면 `/p` 뒤의 `/fd`가 암호로 해석되어 `No file digest algorithm
  specified`라는, 원인과 무관해 보이는 에러가 난다.
- **`signtool verify`가 실패해도 서명은 정상이다.** 자체 서명 인증서의 루트가 빌드 머신의 신뢰
  저장소에 없기 때문이며, `A certificate chain processed, but terminated in a root certificate which
  is not trusted` 메시지가 나온다. 직원 PC에서는 `installer-hooks.nsh`가 설치 시 인증서를 등록한다.
  `Successfully signed`가 출력됐다면 서명 자체는 완료된 것이다.

## 5. 배포용 파일명 확정

기존 CLI 릴리스(`rhwp-v<version>-windows-x86_64.zip`, GitHub Releases)와 혼동하지 않도록 파일명을
바꾼다.

```
rhwp-studio-setup-v<version>.exe
rhwp-studio-setup-v<version>.exe.sha256.txt
```

체크섬 파일은 **내용에도 파일명이 적혀 있다.** 파일만 바꿔치기 하면 `sha256sum -c`가 파일명 불일치로
실패하므로, 개명한 뒤 체크섬을 다시 생성한다.

```powershell
$new = "rhwp-studio-setup-v0.8.2.exe"
Rename-Item "RHWP Studio_0.8.2_x64-setup.exe" $new
Remove-Item "RHWP Studio_0.8.2_x64-setup.exe.sha256.txt" -ErrorAction SilentlyContinue
"$((Get-FileHash $new -Algorithm SHA256).Hash.ToLower())  $new" |
  Out-File "$new.sha256.txt" -Encoding ascii
```

개명은 서명 이후에 해도 서명이 깨지지 않는다 — Authenticode 서명은 파일 내용에만 걸리고 파일명은
포함하지 않는다.

## 6. 배포

수동 배포(공유폴더/메일/사내 포털) — 스펙에서 정한 대로 자동 업데이트 인프라는 두지 않는다. 배포 시
[직원용 설치 안내](EMPLOYEE_INSTALL_GUIDE.md)를 함께 공유한다.

새 버전이 나오면 이 절차를 반복해서 새 설치 파일을 다시 공유한다 — 기존 설치는 새 설치 파일을 그대로
실행하면 같은 경로에 덮어써진다(별도 제거 필요 없음).

## 부록 A. ARM64 빌드 머신에서 x64 설치 파일 만들기

Apple Silicon Mac 위의 Windows VM처럼 빌드 머신이 ARM64인 경우에 해당한다. 사내 직원 PC는 대부분
x64이고, **ARM64용 `.exe`는 x64 Windows에서 아예 실행되지 않으므로**(x64 → ARM64 방향만 에뮬레이션이
된다) 반드시 x64로 크로스컴파일해야 한다.

### A-1. 추가 사전 준비

```powershell
rustup target add x86_64-pc-windows-msvc
```

Visual Studio Build Tools 설치 시 **x64 크로스 컴파일러**를 함께 넣는다. `VC\Tools\MSVC\<ver>\bin\`
아래에 `Hostarm64\x64`가 있으면 준비된 것이다.

빌드 명령은 MSVC 환경변수가 설정된 셸에서 실행해야 한다. 일반 PowerShell 창에서는
`VCINSTALLDIR`이 비어 있어 `cc-rs`가 컴파일러를 못 찾고 clang을 찾다 실패한다. 크로스컴파일용
환경은 다음으로 적재한다.

```powershell
& "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" arm64_x64
```

(`vcvarsall.bat`은 cmd용이므로, PowerShell에서는 `cmd /c "... && set"`으로 환경을 받아오는 래퍼를
쓰거나 x64 Native Tools 명령 프롬프트에서 실행한다.)

### A-2. 1단계 — WASM 빌드

`wasm-pack`은 마지막에 `wasm-opt`(binaryen)로 최적화하는데, **Windows ARM64용 프리빌트
`wasm-opt` 바이너리가 없다.** 다음 에러로 실패한다.

```
Error: no prebuilt wasm-opt binaries are available for this platform: Unrecognized target!
```

최적화를 건너뛰고 빌드한 뒤, 아키텍처 독립적인 npm `binaryen` 패키지로 직접 최적화한다.
(`--no-opt`만 쓰고 최적화를 생략하면 wasm이 약 15% 커진다.)

```powershell
wasm-pack build --target web --out-dir pkg --no-opt

npm install binaryen          # 임시 폴더에서 1회
npx wasm-opt pkg/rhwp_bg.wasm -O -o pkg/rhwp_bg.opt.wasm
Move-Item -Force pkg/rhwp_bg.opt.wasm pkg/rhwp_bg.wasm
```

### A-3. 3단계 — Tauri 빌드

```powershell
npx tauri build --target x86_64-pc-windows-msvc
```

산출물 경로에 타깃 트리플이 들어간다.

```
rhwp-studio/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/RHWP Studio_<version>_x64-setup.exe
```

4단계 `-InstallerPath`도 이 경로로 지정한다.

빌드가 `failed to bundle project: 액세스가 거부되었습니다. (os error 5)`로 실패하면, Tauri가
NSIS를 내려받아 푸는 도중 실시간 검사와 충돌한 것이다. 캐시를 지우고 다시 실행하면 통과한다.

```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\tauri\nsis-3.11"
```

### A-4. 검증

설치 파일 자체(NSIS 스텁)는 **x86으로 나오는 것이 정상**이다. NSIS는 32비트 스텁만 생성하며,
파일명의 `x64`와 실제 아키텍처는 안에 담긴 `app.exe`를 가리킨다. 다음으로 확인한다.

```powershell
# PE 헤더의 머신 타입을 읽는다 — app.exe가 0x8664(AMD64)여야 한다.
$p = "rhwp-studio\src-tauri\target\x86_64-pc-windows-msvc\release\app.exe"
$fs = [IO.File]::OpenRead($p); $br = [IO.BinaryReader]::new($fs)
$fs.Position = 0x3C; $fs.Position = $br.ReadInt32(); $null = $br.ReadUInt32()
'0x{0:X4}' -f $br.ReadUInt16(); $fs.Close()
```

서명 후에는 서명 주체와 타임스탬프도 확인한다.

```powershell
Get-AuthenticodeSignature $installer |
  Select-Object Status, @{n='Signer';e={$_.SignerCertificate.Subject}},
                @{n='Timestamp';e={$_.TimeStamperCertificate.Subject}}
```

`Status`가 `UnknownError`로 나오는 것은 자체 서명 인증서라 루트가 신뢰되지 않기 때문이며, `Signer`가
`CN=RHWP Studio (Internal Distribution)`으로 찍히고 `Timestamp`가 비어 있지 않으면 정상이다.
