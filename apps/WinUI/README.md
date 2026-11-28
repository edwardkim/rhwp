# RHWP WinUI 3 편집기

WinUI 창에서 로컬로 빌드한 rhwp Studio 편집기를 엽니다. 메뉴·도구 모음과 문서 편집은
Studio가 제공하고, WinUI가 Windows 창과 파일 열기·저장을 연결합니다.
[WinUI 3 로드맵](https://github.com/edwardkim/rhwp/issues/8701)

## Studio 번들과 앱 빌드

Windows x64에 .NET 10 SDK, WinUI 도구, Windows App SDK 2.7 x64 런타임,
WebView2 Evergreen Runtime을 설치합니다. Studio 번들을 같은 소스에서 만들려면 저장소의 Rust
도구 체인과 wasm-pack, Node.js와 npm이 필요합니다. 프로젝트에서 사용하는 Node.js와 wasm-pack
버전은 GitHub CI 설정을 기준으로 합니다.
[WinUI 도구 설치](https://learn.microsoft.com/en-us/windows/apps/get-started/start-here) ·
[WebView2 런타임](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/webview2)

저장소 최상위의 PowerShell에서 같은 소스의 WASM과 Studio를 빌드합니다.

```powershell
.\scripts\wasm-pack-locked.cmd --target web --release --out-dir pkg
npm ci --prefix rhwp-studio
node rhwp-studio/build-desktop.mjs

$out = Join-Path $PWD "apps\WinUI\bin\rhwp-x64"
dotnet build apps/WinUI/Rhwp.WinUI.csproj -c Release -p:Platform=x64 -p:RequireRhwpStudio=true -o $out
& (Join-Path $out "Rhwp.WinUI.exe")
```

POSIX 빌드 환경에서는 `./scripts/wasm-pack-locked.sh`로 같은 WASM을 만들 수 있습니다.
`node rhwp-studio/build-desktop.mjs <새 출력 경로>`로 외부에서 만든 번들은
`-p:RhwpStudioDist=<번들 디렉터리의 절대 경로>`로 지정합니다. 기존 번들을 보관하려면 다시
빌드할 때 새 출력 경로를 지정합니다. 디렉터리의 `studio/index.html`이 진입점이고 `bundle.json`은
실제로 포함된 WASM 바이트의 SHA-256을 기록합니다.

번들은 Studio의 WASM·글꼴·인쇄 자산을 함께 넣습니다. 앱은 묶인 콘텐츠를 WebView2의
가상 HTTPS 호스트에서 엽니다. 일반 빌드는 번들을 준비하는 동안 C#을 컴파일할 수 있습니다.
게시하거나 `-p:RequireRhwpStudio=true`로 빌드할 때는 번들이 필요합니다. 배포할 때 실행 파일과
같은 디렉터리에 `Studio` 폴더를 둡니다.
[WebView2 로컬 콘텐츠](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/working-with-local-content)

## Windows 편집기 인수

파일 메뉴와 Ctrl+O는 Windows 파일 선택기를 엽니다. 파일 경로를 EXE 인수로 넘겨도 열 수
있습니다. 일반 HWP·HWPX는 Ctrl+S로 저장하고, 다른 이름으로 저장으로 새 파일을 만듭니다.
저장할 때 원본의 내용 지문과 변환 손실을 검사합니다. 원본이 외부에서 바뀌었거나 새 경로에
파일이 이미 있으면 편집 상태를 유지합니다. 암호와 변환이 필요한 문서는 Studio의 기존
파일 메뉴를 사용합니다.

편집 중 강제 종료하면 같은 Windows 프로필에서 Studio의 복구 창이 열립니다. 복구한 문서의
첫 저장은 새 경로를 선택합니다. 복구본을 연 상태에서는 실행 인수에 지정된 문서가 복구본을
덮어쓰지 않습니다. Windows 테마와 고대비 설정은 Studio 메뉴·도구 모음·초점 표시에 반영됩니다.

```powershell
powershell.exe -NoProfile -File apps/WinUI.Smoke/Verify-Windows.ps1 `
    -AppPath (Join-Path $out "Rhwp.WinUI.exe") `
    -Hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp `
    -Hwpx samples/issue5599_oracle/3191107_leave_request_form.hwpx
```

이 검사는 두 포맷의 파일 인수로 앱을 실행해 창 제목과 Studio의 첫 쪽 표시를 UI Automation으로
확인합니다. 이어 실제 창에서 한글 IME, 본문·표 수정, 실행 취소, 서식, 미저장 편집의 인쇄·PDF,
HWP/HWPX 저장·재열기, 외부 변경 충돌과 강제 종료 후 복구를 확인합니다. 밝은 테마·어두운 테마·
고대비에서 100%와 150% 배율을 적용해 메뉴와 초점 표시를 검사합니다. Narrator로 메뉴와
본문·표를 각각 확인합니다.

## AppKit에서 재사용할 Studio

공통 계약은 `rhwp-studio/src/desktop/desktop-host.ts`에 있습니다. 네이티브 호스트는 편집기 문서가
실행되기 전에 `window.rhwpDesktopHost` v1의 `postMessage`와 `subscribe`를 제공합니다. WinUI는
`appearance: 'fluent'`로 Windows 스타일을 선택합니다. AppKit은 같은 편집기와 번들을 불러오고
macOS 창·메뉴와 시스템 테마를 적용합니다.

Studio의 `ready` 이벤트는 복구 선택과 초기 문서 열기가 끝난 뒤 도착합니다. 호스트가 선택한
파일은 최대 128 MiB이며, 768 KiB 이하의 바이트 조각으로 전달합니다. OS 경로는 호스트에
보관합니다. Studio의 문서 세대와 변경 번호를 기준으로 열기·저장·폐기를 검증하고, 디스크
교체와 쓰기 확인이 끝난 뒤에만 해당 편집 상태의 저장 완료를 알립니다. 복구 데이터는 같은
로컬 오리진의 영속 저장소에 둡니다.

| AppKit 순서 | 구현과 인수 기준 |
| --- | --- |
| 1. 번들 실행 | 기존 Studio 번들을 앱 리소스에 넣고 `WKWebView`로 엽니다. 오프라인에서 상대 경로 자산, 보안 컨텍스트, WASM, WebCrypto, 재실행 후 IndexedDB를 확인해 로컬 오리진 방식을 결정합니다. |
| 2. 창과 브리지 | `WKUserScript`와 `WKScriptMessageHandler`로 v1을 연결하고 메인 프레임과 번들 오리진을 확인합니다. AppKit 창·메뉴·키보드 명령과 시스템 테마를 연결합니다. |
| 3. 문서와 복구 | `NSOpenPanel`·`NSSavePanel`, Finder 파일 열기를 연결합니다. HWP/HWPX 저장·재열기, 외부 변경, 실패 시 편집 유지, 강제 종료와 앱 업데이트 후 복구를 확인합니다. |
| 4. 실제 편집 인수 | 한글 IME와 Retina 배율, VoiceOver, 인쇄·PDF를 확인합니다. Windows와 macOS에서 같은 픽스처를 수정하고 두 포맷의 재열기 결과를 비교합니다. |

[WKUserScript](https://developer.apple.com/documentation/webkit/wkuserscript) ·
[WKScriptMessageHandler](https://developer.apple.com/documentation/webkit/wkscriptmessagehandler)

## Native 편집기 개발 기록

기존 `MainWindow`의 Native Skia 코드와 관리 디스패처 검증은 참고용으로 남아 있습니다.
다음 동작과 Windows 절차는 기존 Native 편집기 빌드를 대상으로 합니다.

Native 편집기 창은 파일 선택과 프레임 표시를 맡습니다. 기존 C# 바인딩으로 worker를 만들고 요청, 폴링,
버퍼 복사·해제를 전용 C# 스레드에서 수행합니다. Rust worker가 문서를 열고 Skia CPU 타일을
그립니다. 표시 영역만 요청하며 확대 배율과 창의 배율을 적용한 한 프레임은 최대 2048×2048
픽셀입니다. 이를 초과하면 해당 프레임의 래스터 배율을 낮춥니다. 디스패처는 문서
ID·revision·글꼴 세대와 픽셀 원점을 확인한 후 RGBA를 BGRA premultiplied 비트맵으로 전달합니다.
파일 교체·쪽 이동·확대·스크롤 중 새 요청이 생기면 이전 요청의 결과를 적용하지 않습니다.
창을 닫을 때는 native worker 종료를 확인하고 handle을 해제합니다.
[WinUI 비트맵 형식](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/imaging)

앱 창에 HWP/HWPX 파일을 놓거나 실행 파일에 경로를 인수로 넘겨도 같은 열기 경로를 사용합니다.
파일 연결을 이미 설정한 Windows에서는 전달받은 파일 활성화도 처리합니다.
[Windows 파일 활성화](https://learn.microsoft.com/en-us/windows/apps/develop/launch/app-lifecycle)

### 문단 편집

본문이나 표 셀의 글자를 클릭하면 그 자리에 편집 상자가 열립니다. 키보드로는 문서에 초점을
놓고 **F2**를 누릅니다. 표준 WinUI TextBox에서 Windows 입력기로 한 문단을 편집합니다.
**적용** 또는 **Ctrl+Enter**로 반영하고, **취소** 또는 **Esc**로 닫습니다. 한글 조합이 끝나면
변경을 적용할 수 있습니다. 적용할 때 문단의 문서 ID·revision·글꼴 세대·선택과 UTF-16 범위를
Native가 함께 검증합니다. 적용한 변경은 도구 모음에서 **실행 취소**하거나 **다시 실행**할 수
있습니다. 편집 상자를 적용하거나 닫은 뒤 쪽 이동·확대·저장·문서 닫기를 사용할 수 있습니다.

### 화면 읽기

UI Automation은 현재 쪽에 배치된 본문과 표 셀을 읽습니다. 쪽을 넘기면 그 쪽의 트리로 바뀝니다.
중첩 표와 셀은 그룹으로, 문단은 텍스트로 제공하며 유효한 셀의 행·열과 병합 범위를 포함합니다.
문단과 그룹의 화면 좌표는 현재 확대율과 스크롤 위치를 반영합니다. 편집 중인 입력 상자는
WinUI의 기본 접근성 기능을 사용합니다. 각 쪽은 본문·표 텍스트 최대 512개 문단과 512 KiB,
중첩 여덟 단계까지 읽으며, 일부만 제공되는 경우 문서의 도움말에서 알립니다.

### 새 파일에 저장

아직 저장하지 않은 변경이 있으면 파일 교체, 문서 닫기, 창 닫기에서 새 파일에 저장하거나
변경을 폐기하거나 전환을 취소할 수 있습니다.

**새 파일에 저장**은 HWP 또는 HWPX 사본을 만듭니다. 변환 손실이 보고되면 항목을 읽고
계속할지 선택합니다. 선택한 이름이 이미 사용 중이면 다른 경로를 선택해야 합니다.
앱은 준비된 바이트를 작성하고 파일 내용을 확인한 뒤 같은 Native 저장 토큰을 승인합니다.
손실 승인 취소나 게시 실패 시 토큰을 취소하고 편집본을 유지합니다.

### 현재 쪽 PDF 미리보기와 저장

PDF 출력을 지원하는 Native 라이브러리에서는 **현재 쪽 PDF 미리보기**로 미저장 편집을
포함한 한 쪽을 확인합니다. 앱은 Native Print 출력의 문서 ID·revision·글꼴 세대와 쪽
번호를 확인한 뒤, Windows의 PDF 렌더러로 한 쪽을 미리보기 이미지로 만듭니다. 미리보기
이미지는 축당 최대 1536픽셀입니다.
[Windows PDF 렌더러](https://learn.microsoft.com/en-us/uwp/api/windows.data.pdf.pdfpage.rendertostreamasync)

미리보기에서 **새 PDF 저장**을 선택하면 미리보기에 사용한 PDF 바이트를 선택한 새 경로에
씁니다. 미저장 편집과 실행 취소는 이후에도 문서에서 계속 사용할 수 있습니다. 선택한 경로에
기존 PDF가 있으면 새 경로가 필요합니다.

**PDF 작업 취소**는 대기 작업을 취소하거나 파일 게시가 시작되기 전에 작업을 중단합니다.
Native 또는 Windows 렌더러가 이미 실행 중이라면 작업이 끝난 뒤 취소를 처리합니다. Windows
인쇄 대화상자와 프린터 출력을 연결하려면 별도 Windows 검증이 필요합니다.

### 검증

기존 Skia 공유 라이브러리 경로를 지정해 관리 디스패처를 검사할 수 있습니다. 예시는 Linux입니다.

```sh
LD_LIBRARY_PATH=/path/to/skia-library-dir dotnet run --project apps/WinUI.Smoke -- \
  samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  samples/issue5599_oracle/3191107_leave_request_form.hwpx
```

PDF 기능을 제공하지 않는 라이브러리의 capability 및 검증기만 검사할 수 있습니다.

```sh
LD_LIBRARY_PATH=/path/to/library-dir dotnet run --project apps/WinUI.Smoke -- --pdf-contract-only
```

`livePdfExport`를 제공하는 같은 리비전의 Native Skia 라이브러리를 사용한다면 두 문서 경로
뒤에 `--live-pdf`를 추가합니다. 현재 둘째 쪽의 출력, 잘못된 revision·글꼴 세대·출력 한도 거부,
PDF 사본 게시, 미저장 편집과 기존 문서 저장 토큰의 보존을 함께 검사합니다.

이미 빌드한 Native 편집기 EXE의 파일 활성화와 첫 쪽 렌더링, UI Automation 본문 노출을
검사할 때는 `-LegacyNative`를 추가합니다. 입력 문서는 각 형식의 본문 텍스트를 포함해야 합니다.

```powershell
powershell.exe -NoProfile -File apps/WinUI.Smoke/Verify-Windows.ps1 `
    -AppPath .\Rhwp.WinUI.exe `
    -Hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp `
    -Hwpx samples/issue5599_oracle/3191107_leave_request_form.hwpx -LegacyNative
```

Windows 창에서 다음 동작도 확인합니다.

- 한글 파일명의 여러 쪽 HWP와 HWPX를 교차로 엽니다. 1·2쪽 이동, 확대·축소 및 스크롤 중
  이전 문서나 다른 쪽 픽셀이 나타나지 않는지 확인합니다.
- 파일 선택기를 취소하면 기존 쪽을 유지합니다. 읽을 수 없는 파일에서 오류가 표시된 뒤
  다른 파일을 열 수 있습니다.
- Windows 배율 100%와 150%에서 Native CPU 타일의 내용·색상·알파를 표시 결과와 대조합니다.
- **문서 닫기**를 누르면 화면이 비워지고 새 파일을 열 수 있습니다. 렌더 요청 중 앱도 닫을 수 있습니다.
- 본문과 공개 신청서 이름 칸을 클릭해 같은 위치에서 Windows 한국어 입력기로 편집합니다.
  Tab과 F2로도 편집을 시작합니다. 조합 중 적용이 막히는지, 조합 뒤 Ctrl+Enter와 Esc, 실행
  취소·다시 실행에서 해당 문단과 이웃 셀이 예상대로 남는지 확인합니다.
- Narrator 또는 Accessibility Insights로 본문과 신청인의 중첩 셀을 읽습니다. 스크롤·확대 후
  보이는 본문의 좌표를 확인하고, 긴 문서의 후반 쪽에서도 그 쪽 내용을 읽습니다. 편집 중 입력
  상자의 초점과 텍스트가 노출되고, 적용·실행 취소·문서 교체 뒤 트리가 갱신되는지 확인합니다.
- 편집 후 새 이름으로 HWP와 HWPX를 저장하고 앱에서 다시 엽니다. 기존 이름에 저장을 시도하면
  기존 파일과 편집 상태가 보존됩니다.
- 편집한 상태에서 현재 쪽의 PDF 미리보기와 저장한 PDF의 내용을 비교합니다. 미리보기 닫기,
  저장 위치 선택 취소, 기존 경로 선택 후 새 파일이 생기지 않고 문서 상태가 유지되는지 확인합니다.

관리 디스패처, UTF-16 변환, 중첩 셀 위치와 접근성 데이터는 Linux의 실제 Skia 라이브러리로
검증할 수 있습니다. Windows에서는 XAML 빌드, WinUI 창, 한국어 입력기, 화면 배율별 렌더링,
UI Automation 탐색을 확인합니다.
