---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-10-22
---

# Kitty graphics 터미널·페이지별 파일 출력

`native-skia` 빌드의 `rhwp export-kitty`는 문서의 Native 래스터를 PNG로 인코딩해
Kitty graphics protocol로 표시한다. 기본 너비는 360px이며 배경은 흰색이다.
Sixel처럼 색을 256색으로 양자화하지 않고 PNG의 색을 유지한다.
지원 범위는 Native CLI다. rhwp-studio의 화면이나 WASM API에 명령을 추가하지 않는다.

## 시작하기

Kitty graphics를 지원하는 터미널에서 실행한다. 소스에서 빌드한다면 저장소 루트에서
`native-skia`를 켠다. 기본 빌드만 사용하면 실제 내보내기는 종료 코드 2로 끝난다.
Native 빌드의 준비 사항은 [개발 환경 가이드](dev_environment_guide.md#네이티브-빌드와-테스트)를 따른다.

```bash
cargo build --locked --release --features native-skia --bin rhwp
./target/release/rhwp export-kitty --help
./target/release/rhwp export-kitty samples/hwp_table_test.hwp -p 0 --width 360 --profile print
```

설치한 바이너리가 PATH에 있으면 아래 예시처럼 `rhwp`로 실행한다.
Windows의 직접 실행 파일은 `rhwp.exe`다. SSH에서는 원격 호스트에 맞게 빌드한 바이너리를 사용한다.

## 직접 표시와 파일 저장

```bash
rhwp export-kitty document.hwp --page 0 --width 360
rhwp export-kitty document.hwpx --page 1 --width 720 --profile print
rhwp export-kitty document.hwpx --width 360 > page.kitty
cat page.kitty
rhwp export-kitty document.hwp -o output/kitty --width 720
rhwp export-kitty document.hwp -p 2 -o output/kitty-page --width 720
```

## 옵션과 입출력

- `-p, --page <번호>`는 0부터 시작한다. stdout 모드는 생략하면 첫 쪽만 표시한다.
- `-o, --output <폴더>`를 지정하면 전체 쪽을 개별 파일로 저장한다. `-p`를 함께 지정하면
  해당 쪽만 저장한다. 한 쪽 또는 선택 쪽은 `<문서명>.kitty`, 여러 쪽은
  `<문서명>_001.kitty`부터 시작한다. 같은 이름의 파일은 덮어쓰며, 뒤 쪽 저장이 실패해도
  이미 저장한 앞 쪽은 남는다.
- `--width <픽셀>`은 1..16384 범위다. 가로·세로에 같은 배율을 적용하며 세로는 올림한다.
  한 변 16,384px·전체 67,108,864픽셀 제한을 적용한다. 터미널 창이 작으면 360px부터 확인한다.
- `--font-path <경로>`는 반복 가능하다. 글꼴 선택과 `--profile screen|print|high-quality|fast-preview`,
  `--compat 2022|2024`의 의미는 기존 Native 출력과 같다. 기본 프로필은 `screen`, 호환 옵션은 `2022`다.
- 일반 실행의 stdout은 그래픽 APC 청크와 마지막 CRLF다. 파일 모드 stdout은 비어 있으며
  저장 안내·오류는 stderr다. `--json`은 지원하지 않는다. `--help`는 텍스트를 출력한다.
- 성공은 종료 코드 0, 읽기·파싱·렌더·인코딩·쓰기 실패는 1, 잘못된 옵션·쪽 범위·feature 누락은 2다.
  렌더·PNG 생성은 쓰기 전에 끝낸다. 쓰기 도중 실패하면 이미 보낸 바이트를 되돌리지는 못한다.
- 명령은 stdin을 읽거나 터미널 응답을 기다리지 않으며 지원 탐지·설정 변경·기존 이미지 삭제를
  수행하지 않는다. 종료 코드 0은 전송 성공이다. 터미널의 실제 표시 여부는 별도로 확인한다.

## SSH와 실제 터미널 확인

원격 호스트에 Native CLI·문서·필요한 글꼴을 준비한다. 로컬에서 Kitty graphics를 해석하는
터미널을 사용하고, 터미널 이름·버전·접속 경로를 함께 기록한다.

```bash
ssh -T host 'rhwp export-kitty document.hwp -p 0 --width 360 --profile print'
ssh -T host 'cat page.kitty'
```

직접 표시와 저장 파일 재표시에서 한글·표·각주·그림·종횡비·프롬프트 위치를 확인한다.
원격 셸의 배너나 stdout 로그가 그래픽 사이에 섞이지 않는지도 확인한다.
CLI와 저장 파일 모두 직접 전송을 사용하므로 SSH 양 끝에서 파일 경로를 공유할 필요는 없다.
tmux/screen 중계와 자동 Sixel 전환은 이번 지원 범위에 포함하지 않는다.

2026-10-22 kitty 0.49.2의 Linux WSLg 실행에서 로컬 직접 표시·파일 재표시와
원격 Linux의 SSH 직접 표시·파일 재표시를 확인했다.
같은 날 Windows WezTerm 20240203의 WSL 경유 SSH에서는 이미지가 표시되지 않았다.
Windows 11 호스트의 WezTerm 자체 SSH에서는 사용자가 그래픽과 한글 표시를 확인했다.
Windows의 중계 계층을 우회하는
[WezTerm 자체 SSH](https://wezterm.org/ssh.html)는 다음처럼 연결한다.

```powershell
wezterm ssh user@host
```

접속 후 원격 셸에서 `rhwp export-kitty document.hwp -p 0 --width 360`을 실행한다.
WSL 경유 실패만으로 WezTerm 자체 SSH의 성공·실패를 판정하지 않는다.
환경별 실제 관측은 [#8150 2단계 기록](../working/task_m100_8150_stage2.md)에 연결한다.

이미지의 표·괘선은 보이는데 한글만 비어 있다면 **렌더링하는 원격 호스트의 글꼴**을 확인한다.
터미널에는 PNG가 전송되므로 로컬 터미널의 한글 폰트 설정으로 원격의 누락 글자를 복원할 수 없다.
원격에 필요한 글꼴을 공급하고, 필요하면 `--font-path /path/to/fonts`를 지정해 다시 생성한다.
글꼴 디렉터리 또는 개별 파일을 지정할 수 있으며, 여러 경로는 `--font-path`를 반복한다.
지정한 경로가 실제로 존재하고 필요한 한글·기호를 포함한 글꼴인지 확인한다.
저장한 `.kitty` 파일은 이전 이미지이므로 폰트 공급 뒤 다시 내보내야 한다.
저장소의 `ttfs/opensource`를 배포할 때는 포함된 OFL 라이선스도 함께 둔다.
상대 경로의 번들 공급에 의존한다면 작업 디렉터리를 확인하고, 다른 디렉터리에서 실행할 때는
`--font-path`에 절대 경로를 전달한다.

예를 들어 원격 호스트의 `/opt/rhwp/fonts`에 필요한 글꼴과 라이선스를 준비했다면 다음처럼 실행한다.

```bash
rhwp export-kitty document.hwp -p 0 --width 360 --profile print --font-path /opt/rhwp/fonts
rhwp export-kitty document.hwp -p 0 --font-path /opt/rhwp/fonts/korean.ttf --font-path /opt/rhwp/fonts/symbols.otf
ssh -T host 'rhwp export-kitty document.hwp -p 0 --width 360 --font-path /opt/rhwp/fonts'
rhwp export-kitty document.hwp -p 0 --width 360 --font-path /opt/rhwp/fonts > page.kitty
cat page.kitty
```

예시의 경로는 렌더링 호스트의 실제 경로로 바꾼다.
[Sixel의 글자 누락 해결 방법](export_sixel_command.md#한글이나-일부-글자가-보이지-않을-때)도 같은 원칙을 따른다.

## 문제가 있을 때

| 증상 | 확인할 항목 |
| --- | --- |
| `native-skia feature` 오류 | 위의 Native 빌드 방법으로 만든 실행 파일인지 확인한다. |
| 종료 코드 0인데 이미지가 없음 | Kitty graphics 지원 터미널과 실제 접속 경로를 확인한다. 자동 탐지·Sixel 전환은 없다. |
| 표·그림은 보이고 한글만 없음 | 렌더링 호스트의 글꼴을 공급하고 `--font-path`를 지정해 다시 출력한다. |
| 글꼴을 공급해도 저장 파일은 그대로임 | `.kitty`에는 이미 렌더링한 PNG가 들어 있으므로 다시 생성한다. |
| 이미지가 너무 크거나 잘림 | `--width 360`처럼 너비를 줄인다. 터미널 창에 맞춘 자동 축소는 하지 않는다. |

이 명령의 stdout을 로그나 JSON으로 처리하지 않는다. 파일로 저장할 때는 `>`로 그래픽만 받고,
진단까지 같은 파일에 합치는 `2>&1`은 사용하지 않는다.

## 독립 전송 검사

[공식 Kitty 규격](https://sw.kovidgoyal.net/kitty/graphics-protocol/)의 PNG 직접 전송을 사용한다.
최초 APC는 `a=T,f=100,t=d,q=2`, 계속 청크는 `m`·`q`만 가지며 마지막 청크는 `m=0`이다.
Base64 payload는 청크당 최대 4,096바이트다.

`scripts/verify-kitty-output.py`는 제품 writer를 호출하지 않고 Python 표준 Base64와 Pillow로
복원한다. 문서 전쪽의 stdout/파일 일치, PNG 픽셀 무손실, 기존 PNG 픽셀·Sixel 바이트의
변경 전후 보존을 검사한다. 이전과 현재의 Native CLI를 별도 경로로 전달한다.

```bash
venv/bin/python scripts/verify-kitty-output.py \
  --rhwp-bin output/issue8150/candidate/rhwp \
  --baseline-bin output/issue8150/base/rhwp \
  --input samples/hwp_table_test.hwp \
  --input samples/footnote-01.hwp \
  --input samples/pic-crop-01.hwp \
  --input samples/issue7688/arrow-controls/control-compound.hwpx \
  --out output/issue8150/independent
venv/bin/python -m unittest discover -s scripts/tests -p test_verify_kitty_output.py
```

기존 결과가 있는 출력 폴더는 거부한다. 실행 명령·stderr·입력/producer SHA·페이지별 결과는
출력 폴더의 `commands.jsonl`과 `results.json`에 기록한다. 비교 프로필은 모두 `print`다.
종이 크기도 같은 Print layer SVG에서 읽으므로 기본 screen SVG의 논리 크기와 섞지 않는다.
독립 복원 성공은 한컴과의 조판 일치 증거를 대신하지 않는다. 해당 비교는
[Visual Sweep](verification/visual_sweep_guide.md)의 별도 절차를 따른다.
