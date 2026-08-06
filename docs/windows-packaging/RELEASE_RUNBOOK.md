# RHWP Studio Windows 설치 파일 릴리스 절차

사내 배포용 Windows 데스크톱 앱(`RHWP Studio`) 설치 파일을 만드는 절차다. 설계 배경은
[Windows 패키징 설계 스펙](../superpowers/specs/2026-08-06-rhwp-windows-packaging-design.md)을 따른다.

이 절차는 **전부 Windows 머신에서 실행한다** — Tauri의 NSIS 번들러가 Windows에서만 최종 `.exe`를
만든다. CI 자동화는 범위 밖이므로 릴리스 담당자가 아래를 순서대로 수동 실행한다.

## 0. 사전 준비 (최초 1회, 이후 재사용)

- Rust, Node.js, `wasm-pack` 설치 (저장소 `mydocs/manual/onboarding_guide.md` 참고)
- Windows SDK 설치 (`signtool.exe` 포함) — Visual Studio Installer의 "Windows 10/11 SDK" 구성요소로
  설치 가능
- 코드사이닝 인증서가 없다면:
  ```powershell
  scripts/windows-packaging/generate-self-signed-cert.ps1
  ```
  개인키(`.pfx`)는 사내 비밀 저장소에 보관하고, 공개 인증서(`.cer`)는 아래 위치에 복사한다.
  ```
  rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer
  ```
  인증서가 만료되기 전까지는 매 릴리스마다 새로 만들 필요 없다.

## 1. WASM 빌드

저장소 루트에서:

```powershell
wasm-pack build --target web --out-dir pkg
```

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

산출물: `rhwp-studio/src-tauri/target/release/bundle/nsis/RHWP Studio_<version>_x64-setup.exe`

## 4. 서명

```powershell
scripts/windows-packaging/sign-installer.ps1 `
  -InstallerPath "rhwp-studio\src-tauri\target\release\bundle\nsis\RHWP Studio_<version>_x64-setup.exe" `
  -PfxPath "<인증서 .pfx 경로>"
```

서명 후 같은 폴더에 `<설치파일명>.sha256.txt` 체크섬이 함께 생성된다.

## 5. 배포용 파일명 확정

기존 CLI 릴리스(`rhwp-v<version>-windows-x86_64.zip`, GitHub Releases)와 혼동하지 않도록 파일명을
바꾼다.

```
rhwp-studio-setup-v<version>.exe
rhwp-studio-setup-v<version>.exe.sha256.txt
```

## 6. 배포

수동 배포(공유폴더/메일/사내 포털) — 스펙에서 정한 대로 자동 업데이트 인프라는 두지 않는다. 배포 시
[직원용 설치 안내](EMPLOYEE_INSTALL_GUIDE.md)를 함께 공유한다.

새 버전이 나오면 이 절차를 반복해서 새 설치 파일을 다시 공유한다 — 기존 설치는 새 설치 파일을 그대로
실행하면 같은 경로에 덮어써진다(별도 제거 필요 없음).
