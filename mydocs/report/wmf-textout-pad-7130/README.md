# #7130 — `META_TEXTOUT` 채움 바이트가 글자로 그려진다

한/글 2020 이 만든 `실험4 Transistor-MOSFET.hwp`(#7105 첨부)의 OLE 회로도 WMF 에서,
rhwp 가 **길이가 홀수인 라벨** 끝에 글자를 하나씩 더 그렸다.

MS-WMF §2.3.5.6 에서 `StringLength` 는 글자 수이고 `String` 필드가 차지하는 **자리만**
짝수로 올림된다. 홀수 길이일 때 뒤에 붙는 1바이트는 WORD 정렬용 채움이며 **0 으로 정해져
있지 않다**. 이 문서는 버퍼에 남아 있던 값을 그대로 쓴다.

## 원본 바이트 (문서의 `BinData/*.OLE` 전수)

`META_TEXTOUT` 홀수 길이 406개 · 짝수 287개. `META_EXTTEXTOUT` 은 0개.

| 라벨 | 길이 | 채움 바이트 | 종전 출력 |
|---|---:|---|---|
| `Emitter` | 7 | `0x6F` = `o` | `Emittero` |
| `VCC`·`10V`·`20K`·`500`·`VBB` | 3 | `0x31` = `1` | `VCC1`·`10V1`·`20K1`·`5001`·`VBB1` |
| `COM`·`GND` | 3 | `0x5E` = `^` / `0x4B` = `K` | `COM^`·`GND^` / `COMK`·`GNDK` |
| `Collector` | 9 | `0x00` | (눈에 안 보임) |
| `Base`·`IB`·`2N2222`·`PNP Transistor` | 짝수 | 없음 | 정상 |

## 문서 전수 대조 — 라벨 222개 중 24개가 고쳐졌다

| 쪽 | 수정 전 → 수정 후 |
|---:|---|
| 1 | `Emittero` → `Emitter` |
| 2 | `20KV` → `20K` · `500V` → `500` · `VCCV` → `VCC` · `10VV` → `10V` |
| 3 | `20K^` → `20K` · `VBB^` → `VBB` · `5009` → `500` |
| 4 | `DrainF`·`Draine` → `Drain` · `VGSe`·`VGS9` → `VGS` |
| 5 | `CLK2` → `CLK` |
| 6 | `COM^`·`COMK` → `COM` · `GND^`·`GNDK` → `GND` |
| 7 | `VBB1` → `VBB` · `20K1` → `20K` · `5001` → `500` · `VCC1` → `VCC` · `10V1` → `10V` · `LED2` → `LED` |
| 8 | `RD21` → `RD2` |

나머지 **198개는 그대로**다.

한/글 2020 정본 PDF 에 고친 20종이 전부 있고, 고치기 전 형태는 하나도 없다. 유일한 예외인
`VCC1` 은 7쪽에 **진짜로 있는 라벨**이며(전원 둘), 정본 1개 · 수정 후 1개로 개수가 맞는다 —
가짜 `VCC1` 만 `VCC` 로 돌아갔다.

## 시각 증적

`export-png` 는 이 OLE 도해를 회색 상자로만 그린다(별개 축). 아래 이미지는 `export-svg` 가
내보낸 중첩 SVG 를 그대로 래스터화한 실제 렌더다.

- `npn-pnp-symbol-before-after-oracle.png` — 1쪽 NPN/PNP 기호. `Emittero` → `Emitter`.
- `npn-bias-circuit-before-after-oracle.png` — 2쪽 bias 회로. 한 도해에서 네 곳
  (`20KV`·`500V`·`VCCV`·`10VV`)이 동시에 고쳐진다.
