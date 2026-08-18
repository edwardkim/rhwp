# 07 — `--font-style` (하네스 기본)

하네스의 SVG export 는 `--font-style` 을 기본으로 사용한다.
`rhwp export-svg --font-style` 은 글꼴 바이너리를 embed 하지 않고,
문서 face → 설치된 family/full name 별칭을 `@font-face src: local(...)`
로 남긴다. Chrome 이 그 local 이름을 고르면 비교 PNG 가 두부가 되지
않는다.

이 장은 그 기본값을 에이전트가 **끄거나 대체하지 않게** 고정한다.

## 왜 기본인가

한컴 문서는 `한양중고딕`, `휴먼명조` 같은 legacy 이름을 들고 있다.
OS 에 설치된 파일의 family 는 `HY중고딕`, `HMKMM` 일 수 있다.
별칭 없이 SVG 가 문서 이름만 쓰면 Chrome 은 폴백 글꼴로 가고, 한글이
없으면 □ 가 된다. 그건 문서 회귀가 아니라 **하네스 오염** 이다 (F14).

`--font-style` 은 그 별칭을 SVG 에 써서, 설치만 되어 있으면 Chrome 이
같은 파일을 고르게 한다. 라이선스 바이너리를 저장소나 SVG 에 넣지 않는다.

## 환경 변수 `RHWP_SVG_FONT_MODE`

하네스 `svg_font_export_option`:

| 값 | export 플래그 | 언제 |
| --- | --- | --- |
| `style` (기본) | `--font-style` | 일상 비교. embed 없음 |
| `subset` | `--embed-fonts` | 증거용. 라이선스 확인 후 |
| `full` | `--embed-fonts=full` | 샌드박스 Chrome 이 시스템 글꼴을 못 볼 때 |

기본을 바꾸라는 요청이 없으면 `style` 을 유지한다. `full` 은 산출 SVG 가
커지고, 라이선스 글꼴이 파일에 실릴 수 있다. 이 스킬은 embed 를 기본
레시피에 넣지 않는다.

잘못된 값이면 `RHWP_SVG_FONT_MODE은 style, subset, full 중 하나여야 합니다.`
에이전트가 네 번째 모드를 발명하지 않는다.

## 에이전트가 export-svg 를 직접 칠 때

단건 디버깅으로 `rhwp export-svg` 를 직접 호출할 때도 **같은 플래그** 를
쓴다. 하네스와 다른 플래그로 뽑은 SVG 를 시트에 섞지 않는다.

```bash
# 하네스와 동일한 기본
rhwp export-svg samples/doc.hwp -o /tmp/p.svg --font-style
# 금지: 플래그 없이 뽑고 "한컴과 글꼴이 다르다"고 결론
```

`--font-path` 는 rhwp 로더용이다. Chrome 은 별도다. Linux 에서는
하네스가 `RHWP_FONT_PATH_DIR` 로 fontconfig 를 맞춘다 (10장).
Windows/macOS 는 설치 글꼴을 그대로 쓴다.

## HMKMM / HMKMG / HY신명조

README 정본:

- 휴먼명조 → Chrome 이 `HMKMM` 을 고르지만 EBDT 라 표준 한글을
  `.notdef` 로 그림 → **outline 명조를 먼저** 선택, 좌표는 유지
- 휴먼고딕 → `HMKMG` 동일
- HY신명조 → 정상 outline. **원 face 우선순위를 보존**

에이전트가 "한글이 깨지니 전부 Noto 로 바꿔" 라고 export 를 바꾸지
않는다. 그건 비교를 오염시킨다. 별칭과 우선순위는 렌더러/도구가 이미
갖고 있고, 이 스킬은 그 기본을 호출만 한다.

## 레시피 — 기본이 살아 있는지

산출 SVG 를 열어 `@font-face` 와 `local(` 이 있는지 본다.

```bash
rg -n "@font-face|local\\(" /tmp/rhwp-fidelity-plan/svg/*.svg | head
```

없으면 `RHWP_SVG_FONT_MODE` 가 비었는지, 옛 바이너리인지 확인한다.
플래그를 이 스킬이 새로 추가하지 않는다.

## 에이전트 금지

- 기본을 `--embed-fonts=full` 로 조용히 올리기
- 라이선스 글꼴을 저장소에 커밋
- 비교용 SVG 에서 `@font-face` 를 지우고 시스템 기본만 쓰기
- `font-style` 을 새 CLI 하위명령처럼 문서화하기 (이미 있는 플래그다)
