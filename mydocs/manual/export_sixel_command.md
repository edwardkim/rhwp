---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-10-11
---

# Sixel 터미널·페이지별 파일 출력

`native-skia`로 빌드한 CLI에서 문서 한 쪽을 terminal 이미지로 표시한다.
문서를 저장·편집하지 않는다. PNG와 같은 `PageLayerTree`와 Skia replay를 사용한다.

```bash
rhwp export-sixel samples/hwp_table_test.hwp --page 0 --width 360
rhwp export-sixel samples/footnote-01.hwp --page 1 --width 720 --profile print
rhwp export-sixel document.hwpx --width 360 > page.six
cat page.six
rhwp export-sixel document.hwpx -o output/sixel --width 720
rhwp export-sixel document.hwpx -o output/sixel-page -p 2 --width 720
```

- 쪽 번호는 기존 export 명령처럼 **0부터** 시작한다. stdout에서는 생략하면 첫 쪽만 출력한다.
- `-o, --output <폴더>`로 파일 저장을 선택한다. 쪽을 생략하면 전체 쪽을 각각 저장하며,
  `-p`를 지정하면 해당 쪽만 저장한다. 파일은 한 쪽일 때 `<문서명>.six`, 여러 쪽일 때
  `<문서명>_001.six`부터 시작한다. 기존 `export-png`와 같은 이름 규칙이다.
  파일 모드 stdout은 비어 있고 저장 안내·오류는 stderr다. 실패 시 종료 코드 1이며
  그 전에 저장한 파일은 남는다. 기존 같은 이름의 파일은 덮어쓴다.
- 너비는 픽셀 단위, 기본 360이다. 가로 배율을 세로에도 적용하고 높이는 정수 픽셀로 올림한다.
- 너비·높이는 각각 16,384픽셀 이하, 총 67,108,864픽셀 이하이다.
  Sixel의 마지막 6줄 밴드까지 포함하여 총 픽셀 한도를 확인한다.
  terminal 자체 제한은 더 작을 수 있으므로 360~720픽셀부터 확인한다.
- `--font-path <경로>`는 반복 가능하다. `--profile`과 `--compat 2022|2024`는 PNG와 같은 의미다.
- stdout 모드 성공 출력은 **Sixel DCS 스트림 하나**다. 로그·오류는 stderr이며, `--json`은 지원하지 않는다.
  명령 도움말만 요청하면 일반 텍스트를 출력한다. 스트림에 문서 파일명이나 본문 제어문자를 삽입하지 않는다.
- 종료 코드는 성공 0, 입력 읽기·파싱·렌더·인코딩·stdout 쓰기 오류 1, 사용법·쪽 범위·feature 누락 2다.
- 흰 배경에 alpha 합성한 뒤 Wu 방식으로 최대 256색으로 양자화한다. 디더링을 꺼서 새 점을 흩뿌리지 않는다.
  Sixel은 원본 PNG와 색상이 완전히 동일한 무손실 포맷이 아니다.

## 지원 환경과 직접 판정

[WezTerm](https://wezterm.org/escape-sequences.html)의 Sixel 지원과
[Foot](https://codeberg.org/dnkl/foot)의 그래픽 지원을 사용하는 것을 목표로 한다.
실제로 설치한 버전·설정·terminal 화면에서 별도로 확인한다. `TERM` 값만으로 지원을 판정하지 않으며
CLI가 terminal 설정을 자동으로 변경하거나 지원 탐지용 응답을 기다리지 않는다.
Sixel을 해석하지 않는 terminal에서는 그래픽이 표시되지 않을 수 있다.

1. `rhwp --version`, terminal 이름·버전, Sixel 설정과 출력 너비를 기록한다.
2. 한글 본문·표·각주·그림을 포함한 문서에서 PNG와 Sixel을 같은 쪽·너비로 대조한다.
3. 누락·색 채널 뒤바뀜·표 외곽·각주 위치·종횡비·마지막 밴드를 확인한다.
4. Foot와 WezTerm을 각각 판정한다. PNG를 `img2sixel`로 변환한 결과는 이 CLI 검증으로 대체하지 않는다.
5. SSH는 별도 기록한다. 원격에 이 CLI와 글꼴이 필요하며 `ssh -T host 'rhwp export-sixel ...'`처럼
   PTY 변환 없이 바이트를 전송한다. 배너·stdout 로그·tmux/screen 통과 여부도 확인한다.

현재 검증 결과와 미검증 환경은 #7820 작업 기록에 남긴다. 지원 목표를 실행 완료로 해석하지 않는다.

## 독립 인코딩 검증

`libsixel`의 `sixel2png`로 stdout을 복원한 뒤 동일 너비의 기준 래스터와 크기·픽셀을 대조한다.
단순 인코더 자체 decode 성공은 독립 검증이 아니다. 팔레트의 RGB는 Sixel 규격상 0~100%로
표현되므로 색 값의 작은 반올림 차이를 허용하며, 256색 축소로 발생한 오차는 문서별로 따로 기록한다.
레이아웃·채널 순서·내용 누락을 양자화 오차로 분류하지 않는다. 한컴과의 조판 비교는 별도로
[Visual Sweep](verification/visual_sweep_guide.md)의 Native/fresh Docker WASM 절차를 따른다.
