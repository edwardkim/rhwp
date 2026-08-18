# 08 — 로컬 face 별칭

`--font-style` 이 남기는 `@font-face src: local("…")` 가 이 장의 대상이다.
에이전트는 별칭 표를 새로 만들지 않고, 하네스가 내보낸 SVG 를 읽는다.

## 왜 별칭이 필요한가

HWP 레코드의 face 문자열은 1990년대 이름이다. 현재 파일은 다른
영문 family / Full Name / PostScript 이름을 가진다. Chrome 의
`local()` 매칭은 그 이름 중 일부만 본다. 별칭이 여러 이름을 나열해야
설치되어 있는데도 두부가 나오는 오염을 막는다.

예 (개념, 실제 문자열은 바이너리·SVG 가 권위):

```css
@font-face {
  font-family: "한양중고딕";
  src: local("한양중고딕"), local("HY중고딕"), local("HYgtr");
}
```

에이전트가 이 CSS 를 손 편집해 비교하지 않는다. 편집한 SVG 는 하네스
캐시와 어긋나고, 재현이 사라진다.

## 설치되어 있는데 □

점검 순서:

1. 시트가 온통 □ 인가 → F14, 17장 (하네스 오염)
2. 특정 런만 □ 인가 → `svg-glyph-risk-report.tsv` 의 PUA / U+FFFD
3. `local()` 목록에 실제 설치 이름이 있는가
4. Windows 면 `C:\Windows\Fonts` 와 한컴 Shared\Fonts
5. Linux 면 `RHWP_FONT_PATH_DIR` + fontconfig (10장)
6. HMKMM/HMKMG 인가 → outline 우선 규칙 (07장)

4~5 에서 글꼴이 없으면 **설치를 사용자에게 요청** 한다. 글꼴 파일을
저장소에 넣지 않는다.

## 별칭과 embed

`local()` 은 머신에 파일이 있을 때만 동작한다. CI 러너에 한컴 글꼴이
없으면 시트는 두부가 된다. 그래서:

- CI 계약 시험은 **파일을 읽기만** 하고 시트를 필수로 두지 않는다
- 사람 감사 환경은 글꼴이 있는 머신에서 돌린다
- 증거용으로만 `RHWP_SVG_FONT_MODE=full` 을 검토한다 (라이선스)

"CI 가 한컴과 같다"는 문장을 이 스킬이 주장하지 않는다.

## 레시피 — 별칭 목록 추출

```bash
# 산출 SVG 에서 local() 이름만
python3 - <<'PY'
import re, sys
from pathlib import Path
root = Path("/tmp/rhwp-fidelity-plan/svg")
names = set()
for p in root.glob("*.svg"):
    text = p.read_text(encoding="utf-8", errors="replace")
    names.update(re.findall(r'local\("([^"]+)"\)', text))
    names.update(re.findall(r"local\('([^']+)'\)", text))
print("\n".join(sorted(names)))
PY
```

이 스크립트는 진단이다. 새 도구로 커밋하지 않아도 된다. 이름이 비면
`--font-style` 이 안 붙은 export 다.

## Chrome 이 고른 얼굴

Chrome 이 실제로 무슨 파일을 썼는지는 시트만으로 단정하기 어렵다.
단서:

- 본문 한글이 전부 □ → local 매칭 실패 또는 EBDT `.notdef`
- 본문은 살아 있고 원문자만 □ → PUA (#3385 유형), 별칭 문제가 아님
- 영문만 다른 폭 → 폴백 메트릭, 랭킹 상승, 구조 결함 아님

유지자에게 이 세 분류를 그대로 넘긴다. 에이전트가 글꼴 파일을
교체하는 PR 을 이 스킬 범위에서 열지 않는다.

## 에이전트 금지

- SVG 의 `@font-face` 를 손으로 고친 뒤 시트를 "개선" 으로 제출
- 별칭 사전을 DocumentCore 에 추가하는 구현을 이 이슈에 섞기
- 미설치 글꼴을 웹 폰트 URL 로 몰래 바꾸기 (비교가 한컴과 더 멀어짐)
