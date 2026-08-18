# 17 — 예외: 두부 가득 하네스 (F14)

비교 PNG 가 **본문까지** □ 로 채워져 있으면 문서 회귀보다 하네스
글꼴 오염을 먼저 의심한다. 09장은 코드포인트 분류, 이 장은
**실행을 멈추고 다시 돌리는** 절차다.

## 오염으로 보는 신호

- 모든 쪽, 모든 본문 한글이 네모
- `svg-glyph-risk-report` 가 0 이거나 U+25A1 만
- Linux CI 에 `RHWP_FONT_PATH_DIR` 없음
- Windows 에 한컴 미설치 + `C:\Windows\Fonts` 에 한글 계열 없음
- HMKMM/HMKMG 만 고른 로그/별칭
- `--font-style` 없는 옛 SVG 캐시를 재사용

한 쪽의 원문자만 네모인 것은 오염이 아니라 #3385 유형 후보다.
그 경우 재실행하지 않고 09장 템플릿으로 유지자에게 넘긴다.

## 처방

1. 랭킹과 시트를 **폐기** 한다. 숫자를 이슈에 쓰지 않는다.
2. 글꼴 디렉터리를 확인한다 (10장).
3. `--out-dir` 을 새로 잡거나 `cmp-*.png` 와 `svg/` 를 지운다.
   옛 PNG 캐시가 오염 시트를 재사용한다.
4. `--font-style` 기본이 살아 있는지 확인한다 (`RHWP_SVG_FONT_MODE`).
5. 같은 범위를 다시 돈다.
6. 본문이 살아나면 그때의 `report.tsv` 만 읽는다.
7. 그래도 본문이 네모면 환경을 유지자에게 넘긴다. 렌더러 패치를
   이 이슈에서 열지 않는다.

## 레시피

```bash
# 오염 산출은 남기지 말고 새 디렉터리
export RHWP_FONT_PATH_DIR="/opt/hancom/fonts:$HOME/Library/Fonts"
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 \
  --out-dir /tmp/rhwp-fidelity-plan-fonts
# cmp-p001.png 본문이 한글인지 눈으로 확인한 뒤에만 범위를 넓힌다
```

Windows:

```powershell
$env:RHWP_FONT_PATH_DIR = "C:\Windows\Fonts;C:\Program Files (x86)\Hnc\Shared\Fonts"
Remove-Item -Recurse -Force $env:TEMP\rhwp-fidelity-plan -ErrorAction SilentlyContinue
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 2 `
  --out-dir $env:TEMP\rhwp-fidelity-plan
```

## 캐시 함정

같은 `--out-dir` 에서 글꼴만 바꾸고 다시 돌리면 `capture_with_chrome`
이 기존 PNG 를 크기 > 0 이라 재사용한다. **시트가 그대로 두부** 다.
디렉터리를 바꾸거나 PNG 를 지우는 것이 F14 의 일부다.

SVG 캐시도 별칭이 없는 옛 export 일 수 있다. 의심되면 `svg/` 도 지운다.

## 에이전트 한 줄

"시트 전면 두부는 하네스 오염으로 보고 랭킹을 폐기했습니다.
`RHWP_FONT_PATH_DIR` 재실행 뒤에만 후보를 읽겠습니다."
