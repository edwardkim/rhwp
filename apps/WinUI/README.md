# RHWP WinUI 3 편집기

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

## 문서 위에서 편집하기

본문이나 편집할 수 있는 표 셀을 클릭하면 문서에 커서가 놓입니다. 키보드로는 문서에 초점을
놓고 **F2**를 누릅니다. Windows 입력기로 바로 입력하고, 본문 또는 한 셀 문단 안에서
드래그해 텍스트를 선택합니다. 본문에서는 방향키와 Shift+방향키, Home·End,
Backspace·Delete, Enter를 사용할 수 있습니다. 실행 취소와 다시 실행은 Rust 편집 기록을
사용합니다.

입력기의 조합 미리보기는 Skia에 표시됩니다. 조합을 취소하면 원래 텍스트가 복원되고,
확정하면 실행 취소할 수 있는 변경으로 기록됩니다. 저장·찾기·PDF는 대기 중인 편집과
조합이 끝난 문서에서 동작합니다.

**편집** 메뉴에서 실제 문서 선택을 복사하거나 잘라내고, Windows 클립보드에서 붙여넣습니다.
본문은 텍스트와 HTML을 사용할 수 있습니다. 표 셀의 붙여넣기는 한 문단의 일반 텍스트를 사용합니다.

## Studio 기능과 도구 모음

서식 도구 모음은 본문과 한 셀 안의 텍스트 선택에 굵게와 글자 크기를 적용합니다.
문단 정렬은 캐럿이나 한 문단 안의 선택에 적용하며, 지원하는 Native 라이브러리에서 표시됩니다.

목표는 Studio에서 제공하는 문서 편집·메뉴·도구 모음을 WinUI에 연결하는 것입니다.
파일·편집·보기, 입력·서식·쪽·표·도구 명령의 적용 조건과 결과를 공통 Rust 계약으로
관리합니다. 세부 기능 및 실제 HWP/HWPX 인수 항목은 로드맵에서 추적합니다.
[Studio 기능 및 Native 로드맵](https://github.com/edwardkim/rhwp/issues/8701)

## 문서 찾기와 바꾸기

**편집 → 찾기** 또는 **Ctrl+F**로 검색창을 엽니다. 검색어를 입력하고 Enter나 아래쪽 화살표를
누르면 본문과 표 셀을 검색합니다. 위쪽 및 아래쪽 화살표로 항목을 이동하면 해당 쪽과 위치를 표시합니다.
검색 범위가 제한되거나 읽을 수 없는 내용이 있으면 검색창에 안내가 표시됩니다.

**편집 → 바꾸기** 또는 **Ctrl+H**로 바꿀 내용을 입력합니다. **현재 항목 바꾸기**로 선택한 범위만
바꾸고, 문서나 검색어가 바뀌면 새로 검색합니다. 변경은 **실행 취소**하거나 **다시 실행**할 수 있습니다.
**Esc**로 검색창을 닫습니다.

## 화면 읽기

UI Automation은 현재 쪽에 배치된 본문과 표 셀을 읽습니다. 쪽을 넘기면 그 쪽의 트리로 바뀝니다.
중첩 표와 셀은 그룹으로, 문단은 텍스트로 제공하며 유효한 셀의 행·열과 병합 범위를 포함합니다.
문단과 그룹의 화면 좌표는 현재 확대율과 스크롤 위치를 반영합니다. 편집 중인 입력 상자는
WinUI의 기본 접근성 기능을 사용합니다. 각 쪽은 본문·표 텍스트 최대 512개 문단과 512 KiB,
중첩 여덟 단계까지 읽으며, 일부만 제공되는 경우 문서의 도움말에서 알립니다.

## 저장과 복구

**파일 → 저장** 또는 **Ctrl+S**는 열 때 읽은 HWP/HWPX 파일의 내용 지문을 확인하고 같은
형식으로 저장합니다. 외부에서 파일 내용이 바뀌거나 변환 손실이 보고되면 편집본을 유지합니다.
**다른 이름으로 저장** 또는 **Ctrl+Shift+S**로 새 파일을 만들 수 있습니다. 변환 손실은
목록을 보여 주고 확인을 받으며, 저장에 성공하면 새 파일이 이후 저장 대상이 됩니다.

앱은 임시 파일을 작성하고 같은 위치에서 교체한 뒤 저장 바이트를 확인합니다. 교체 직전
경합이나 교체 결과 확인 실패가 발생하면 확보한 교체 전 파일을 백업으로 남깁니다.
파일 게시를 확인한 뒤 Native 저장 토큰을 승인합니다. 게시 실패나 취소 시 편집본을 유지합니다.

확정된 편집이나 실행 취소·다시 실행 뒤 입력이 잠시 멈추면 미저장 상태를 사용자 로컬의 복구
사본에 기록합니다.
문서 형식의 손실 보고가 없는 사본만 저장하며, 실패하면 마지막으로 저장한 복구 사본을 유지합니다.
예기치 않게 종료한 다음 앱을 시작하면 복구할지 묻습니다. 복구본은 미저장 문서로 열리고
첫 저장에서 새 경로를 선택합니다. 정상 저장이나 편집 폐기를 확인하면 해당 복구 사본을 정리합니다.
입력기에서 조합을 확정하면 그 변경을 복구 사본에 기록할 수 있습니다.

## 현재 쪽 PDF 미리보기와 저장

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

## 검증

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

Windows 데스크톱에서는 EXE의 파일 활성화와 첫 쪽 렌더링, UI Automation 본문 노출을 검사할 수
있습니다. 이미 저장하지 않은 복구 사본이 있는 프로필에서는 복구 선택을 먼저 끝냅니다.
입력 문서는 각 형식의 본문 텍스트를 포함해야 합니다.

```powershell
powershell.exe -NoProfile -File apps/WinUI.Smoke/Verify-Windows.ps1 `
    -AppPath .\Rhwp.WinUI.exe `
    -Hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp `
    -Hwpx samples/issue5599_oracle/3191107_leave_request_form.hwpx
```

Windows 창에서 다음 동작도 확인합니다.

- 한글 파일명의 여러 쪽 HWP와 HWPX를 교차로 엽니다. 1·2쪽 이동, 확대·축소 및 스크롤 중
  이전 문서나 다른 쪽 픽셀이 나타나지 않는지 확인합니다.
- 파일 선택기를 취소하면 기존 쪽을 유지합니다. 읽을 수 없는 파일에서 오류가 표시된 뒤
  다른 파일을 열 수 있습니다.
- Windows 배율 100%와 150%에서 Native CPU 타일의 내용·색상·알파를 표시 결과와 대조합니다.
- **문서 닫기**를 누르면 화면이 비워지고 새 파일을 열 수 있습니다. 렌더 요청 중 앱도 닫을 수 있습니다.
- 본문과 공개 신청서 이름 칸에 커서를 놓고 Windows 한국어 입력기로 입력합니다.
  100%·150% 배율에서 조합 표시와 입력기 후보 위치를 비교하고, Esc로 조합을 취소했을 때
  원문이 복원되는지 확인합니다. 확정된 조합은 실행 취소 한 번으로 되돌립니다.
  본문에서 드래그·Shift+방향키로 선택하고 삭제·다시 실행한 뒤 저장·재열기합니다.
- 공개 신청서를 열고 Ctrl+F로 **생년월일**을 찾습니다. 이전/다음으로 신청인과 자녀의 중첩 표를
  이동하면서 위치 표시와 Narrator의 결과 안내를 확인합니다. Ctrl+H로 한 항목을 바꾸고
  실행 취소했을 때 원래 두 항목이 복원되는지 확인합니다.
- Narrator 또는 Accessibility Insights로 본문과 신청인의 중첩 셀을 읽습니다. 스크롤·확대 후
  보이는 본문의 좌표를 확인하고, 긴 문서의 후반 쪽에서도 그 쪽 내용을 읽습니다. 편집 중 입력
  컨트롤의 초점이 노출되고, 입력·실행 취소·문서 교체 뒤 트리가 갱신되는지 확인합니다.
- 테스트용 HWP와 HWPX 사본을 편집하고 **Ctrl+S**로 저장한 뒤 다시 엽니다. 열어 둔 동안 다른
  프로그램이 파일을 수정한 경우 저장이 거절되고, 다른 이름으로 편집본을 저장할 수 있는지 확인합니다.
- 편집을 확정하고 복구 사본이 기록된 상태에서 테스트용 앱 프로세스를 강제 종료합니다. 다음 실행에서 미저장
  복구본을 열고, 새 이름으로 저장한 파일을 다시 열어 내용과 서식을 확인합니다.
- 편집한 상태에서 현재 쪽의 PDF 미리보기와 저장한 PDF의 내용을 비교합니다. 미리보기 닫기,
  저장 위치 선택 취소, 기존 경로 선택 후 새 파일이 생기지 않고 문서 상태가 유지되는지 확인합니다.

관리 디스패처, UTF-16 변환, 중첩 셀 위치와 접근성 데이터는 Linux의 실제 Skia 라이브러리로
검증할 수 있습니다. Windows에서는 XAML 빌드, WinUI 창, 한국어 입력기, 화면 배율별 렌더링,
UI Automation 탐색을 확인합니다.
