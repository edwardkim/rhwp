# 10 — `RHWP_FONT_PATH_DIR`

라이선스가 있는 로컬 글꼴 디렉터리를 하네스와 rhwp 에 알리는 계약이다.
기본 `--font-style` 은 embed 하지 않으므로, **파일이 머신 어디에 있는지**
가 Chrome/rhwp 양쪽의 전제다.

## 형식

역사적으로 한 디렉터리였다. 지금은 플랫폼 pathsep 으로 여러 개를 받는다.

| 플랫폼 | 구분자 | 예 |
| --- | --- | --- |
| POSIX | `:` | `/Library/Fonts:/opt/hancom/fonts` |
| Windows | `;` | `C:\Windows\Fonts;C:\Program Files (x86)\Hnc\Shared\Fonts` |

존재하지 않는 조각은 `configured_font_paths` 가 버린다. 빈 값이면
추가 디렉터리가 없다.

## 누가 읽는가

- **rhwp** export-svg: `--font-path` 계열로 로더가 파일을 찾는다
- **Linux Chrome**: 하네스가 `work_dir/_fontconfig/fonts.conf` 를 만들고
  `FONTCONFIG_PATH` / `FONTCONFIG_FILE` 을 캡처 프로세스에만 넣는다
- **Windows / macOS Chrome**: fontconfig 를 만들지 않는다. 설치 글꼴
  동작을 유지한다. 디렉터리를 넘겨도 캡처 env 는 `None`

Linux CI 에서 한컴 글꼴을 쓰려면 이 변수가 거의 필수다.
Windows 개발 머신은 한컴 설치만으로 충분한 경우가 많다.

## 레시피 — Linux 증거 런

```bash
export RHWP_FONT_PATH_DIR="/usr/local/share/fonts/hancom:/usr/share/fonts/truetype"
export RHWP_BIN=target/pr-review/release-test/rhwp
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 4 \
  --out-dir /tmp/rhwp-fidelity-plan-fonts
# work_dir/_fontconfig/fonts.conf 에 <dir> 두 줄이 있는지 확인
```

`fonts.conf` 는 산출이다. 저장소에 커밋하지 않는다.

## 레시피 — Windows

```powershell
$env:RHWP_FONT_PATH_DIR = "C:\Windows\Fonts;C:\Program Files (x86)\Hnc\Shared\Fonts"
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 4 `
  --out-dir $env:TEMP\rhwp-fidelity-plan-fonts
```

`_fontconfig` 폴더가 생기지 않는 것이 정상이다.

## 라이선스

한컴 글꼴은 재배포가 막혀 있는 경우가 많다.

- 저장소 `ttfs/opensource/` 만 커밋되어 있다
- `RHWP_FONT_PATH_DIR` 은 **로컬 절대 경로** 다
- 글꼴 파일을 `--out-dir` 에 복사하지 않는다
- `RHWP_SVG_FONT_MODE=full` 은 SVG 안에 바이너리가 들어간다. 산출물을
  공개 gist 에 올리지 않는다

이 스킬은 글꼴 설치 프로그램을  bundles 하지 않는다. 사용자에게
"한글이 쓰는 글꼴이 설치된 머신에서 돌리세요" 라고 한다.

## korexam 후속 실험

README 실측: A3 2단의 잔여 줄바꿈은 폰트 폴백 메트릭 의심.
다음 실험이 바로 이 변수다.

```bash
# 폴백(오픈소스만) vs 한컴 디렉터리
RHWP_FONT_PATH_DIR="$PWD/ttfs/opensource" \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py korexam 0 2 \
  --out-dir /tmp/korexam-oss
RHWP_FONT_PATH_DIR="/opt/hancom/fonts" \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py korexam 0 2 \
  --out-dir /tmp/korexam-hnc
# report.tsv 순위를 비교. 절대값 비교는 하지 않는다.
```

두 런의 provenance 에 글꼴 경로를 남긴다.

## 에이전트 금지

- 글꼴 TTF 를 이 PR 에 추가
- Linux fontconfig 를 Windows 에 이식하는 패치
- 변수 이름을 `FONT_DIR` 등으로 새로 만들기 (계약은 `RHWP_FONT_PATH_DIR`)
- 디렉터리가 비었는데도 시트를 문서 회귀로 승격
