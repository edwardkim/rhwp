# HWPX→HWP 저장 계약 (oracle vs generated)

HWPX 로 연 문서를 HWP 로 저장(#178 어댑터)했을 때 한컴이 다르게 여는 경우.

## 이름

| 이름 | 누구의 손 | 파일 예 |
|---|---|---|
| oracle | 한컴이 저장한 HWP | `oracle.hwp`, `hancom-saved.hwp` |
| generated | rhwp 가 저장한 HWP | `generated.hwp`, `rhwp-saved.hwp` |
| source | 원본 HWPX | `source.hwpx` |

명령 인자는 항상 `oracle generated` 순이다.

## 언제 이 축인가

- "한컴에서 안 열려요" / "표가 한컴이랑 달라요" / "저장하니까 깨져요"
- convert 산출을 한컴이 거부
- IR 은 같은데 한컴만 실패

화면 겹침만 있고 한컴 저장본이 없으면 1–5단으로 남는다. 6단을 가짜 oracle 로 채우지 말 것.

## 최소 세트

```bash
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/
rhwp hwp5-anchor-trace generated.hwp --needle "문제문장" --section 0
```

원본 HWPX 가 있으면:

```bash
rhwp hwp5-contract-analyze source.hwpx oracle.hwp generated.hwp --out-dir output/poc/contract/
rhwp hwp5-char-shape-audit oracle.hwp generated.hwp --source-hwpx source.hwpx --out output/audit.md
```

## 읽기 규칙

- inventory 힌트는 축 후보다. serializer 패치를 여기서 쓰지 않는다.
- CHAR_SHAPE equivalent 는 비활성 underline/strike/shadow sentinel 제거 비교다.
- PARA_LINE_SEG 쪽수 표식이 0 일 수 있다. 한컴 PDF 쪽번호와 같지 않다.
- 같은 source charPr signature 가 서로 다른 raw 분류에 나타나면 선택 기준으로 쓰지 않는다.

## 자기 라운드트립과의 관계

`hwp5-roundtrip generated.hwp` 가 통과해도 oracle 과 다를 수 있다.
자기 닫힘과 한컴 계약은 다른 명제다. [19_roundtrip_vs_hangul.md](19_roundtrip_vs_hangul.md).
