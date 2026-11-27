# RHWP WinUI 3 문서 보기

Windows x64에서 **파일 열기** 또는 Ctrl+O로 HWP/HWPX 파일을 선택합니다. 첫 쪽부터 이전·다음
쪽 버튼으로 이동하고 50%에서 400%까지 확대하거나 축소할 수 있습니다. 창보다 큰 쪽은
스크롤해서 봅니다. **문서 닫기**는 앱을 종료하지 않고 현재 문서를 닫습니다.
[WinUI 3 로드맵](https://github.com/edwardkim/rhwp/issues/8701)

## Windows에서 빌드하고 실행하기

Windows x64에 .NET 10 SDK, Visual Studio의 WinUI 및 C++ 빌드 도구,
Windows App SDK 2.7 x64 런타임을 설치합니다. Rust는 MSVC x64 도구 체인을 사용합니다.
[WinUI 도구 설치](https://learn.microsoft.com/en-us/windows/apps/get-started/start-here) ·
[Windows App SDK 다운로드](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads)

저장소 최상위에서 다음 PowerShell 명령을 실행합니다.

```powershell
rustup target add x86_64-pc-windows-msvc
$nativeTarget = Join-Path $PWD "target\winui-x64"
$env:CARGO_TARGET_DIR = $nativeTarget
cargo build --locked --release -p rhwp-native-ffi --features native-skia --target x86_64-pc-windows-msvc

$dll = Join-Path $nativeTarget "x86_64-pc-windows-msvc\release\rhwp_native_ffi.dll"
$out = Join-Path $PWD "apps\WinUI\bin\rhwp-x64"
dotnet build apps/WinUI/Rhwp.WinUI.csproj -c Release -p:Platform=x64 "-p:RhwpNativeDll=$dll" -o $out
& (Join-Path $out "Rhwp.WinUI.exe")
```

`RhwpNativeDll`은 빌드 결과에 DLL을 포함합니다. 기존 x64 Skia 빌드가 있으면 그 DLL의 절대
경로를 지정할 수 있습니다. 앱은 unpackaged이고 설치된 Windows App SDK 런타임을 사용합니다.
[Unpackaged 배포](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/unpackage-winui-app)

앱 창은 파일 선택과 프레임 표시를 맡습니다. 기존 C# 바인딩으로 worker를 만들고 요청, 폴링,
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

```powershell
& (Join-Path $out "Rhwp.WinUI.exe") (Resolve-Path ".\문서.hwpx").Path
```

## 문단 편집

본문이나 표 셀의 글자를 클릭한 뒤 편집 대화상자에서 한 문단의 내용을 바꾸고 **적용**을
누릅니다. 대화상자의 표준 WinUI TextBox가 Windows 입력기를 사용합니다. 적용할 때 클릭한
문단의 문서 ID·revision·글꼴 세대·선택과 UTF-16 범위를 Native가 함께 검증합니다.
도구 모음에서 변경을 **실행 취소**하거나 **다시 실행**할 수 있습니다.

아직 저장하지 않은 변경이 있으면 파일 교체, 문서 닫기, 창 닫기에서 폐기 또는 취소를 선택합니다.
파일 저장은 후속 단계에 연결합니다.

## 검증

기존 Skia 공유 라이브러리 경로를 지정해 관리 디스패처를 검사할 수 있습니다. 예시는 Linux입니다.

```sh
LD_LIBRARY_PATH=/path/to/skia-library-dir dotnet run --project apps/WinUI.Smoke -- \
  samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  samples/issue5599_oracle/3191107_leave_request_form.hwpx
```

Windows 창에서 다음 동작을 확인합니다.

- 한글 파일명의 여러 쪽 HWP와 HWPX를 교차로 엽니다. 1·2쪽 이동, 확대·축소 및 스크롤 중
  이전 문서나 다른 쪽 픽셀이 나타나지 않는지 확인합니다.
- 파일 선택기를 취소하면 기존 쪽을 유지합니다. 읽을 수 없는 파일에서 오류가 표시된 뒤
  다른 파일을 열 수 있습니다.
- Windows 배율 100%와 150%에서 Native CPU 타일의 내용·색상·알파를 표시 결과와 대조합니다.
- **문서 닫기**를 누르면 화면이 비워지고 새 파일을 열 수 있습니다. 렌더 요청 중 앱도 닫을 수 있습니다.
- 본문과 공개 신청서 이름 칸을 클릭해 Windows 한국어 입력기로 한글을 입력합니다. 대화상자
  취소, 적용, 실행 취소·다시 실행에서 클릭한 문단과 이웃 셀을 확인합니다. 파일 교체를 취소하면
  문서와 변경 내용을 유지합니다.

관리 디스패처와 UTF-16 변환은 Linux의 실제 Skia 라이브러리로 검증했습니다. 이 환경의
.NET 9 SDK는 Windows 프로젝트의 .NET 10 대상을 빌드하지 못했습니다(`NETSDK1045`).
WinUI 창, 한글 IME 대화상자, Windows 배율별 캡처는 Windows에서 확인합니다.
