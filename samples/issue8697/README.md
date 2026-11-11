# #8697 HFT 프로그램 폭 탐침

일곱 언어 슬롯을 모두 같은 HFT 글꼴로 둔 합성 입력이다. 문단마다 `가<글자>가` 를 적어,
독립 한컴 Print PDF 의 Type3 `/Widths`·ToUnicode 와 글자 원점으로 그 글꼴 프로그램이 직접
그리는 글자와 전진폭을 읽는다. 저장 줄 정보는 없다.

## 생성

```sh
CHARS='0021,0022,0023,0024,0025,0026,0027,0028,0029,002A,002B,002C,002D,002E,002F,0030,0031,0032,0033,0034,0035,0036,0037,0038,0039,003A,003B,003C,003D,003E,003F,0040,0041,0042,0043,0044,0045,0046,0047,0048,0049,004A,004B,004C,004D,004E,004F,0050,0051,0052,0053,0054,0055,0056,0057,0058,0059,005A,005B,005C,005D,005E,005F,0060,0061,0062,0063,0064,0065,0066,0067,0068,0069,006A,006B,006C,006D,006E,006F,0070,0071,0072,0073,0074,0075,0076,0077,0078,0079,007A,007B,007C,007D,007E,3000,3001,3002,3003,3004,3005,3006,3007,3008,3009,300A,300B,300C,300D,300E,300F,3010,3011,3012,3013,3014,3015,3016,3017,3018,3019,301A,301B,301C,301D,301E,301F,3020,3021,3022,3023,3024,3025,3026,3027,3028,3029,302A,302B,302C,302D,302E,302F,3030,3031,3032,3033,3034,3035,3036,3037,3038,3039,303A,303B,303C,303D,303E,303F,00B7,2027,2015,2018,2019,201C,201D,2026,203B,25CB,25A1,2192,339C,338F,33A1'
for face in '한양중고딕:hanyang_junggothic' '신명 중고딕:sinmyeong_junggothic'; do
  name=${face%%:*}; out=${face##*:}
  FTYPE=HFT FACES="$name,$name,$name,$name,$name,$name,$name" CHARS="$CHARS" PATTERN='{c}{x}{c}' \
    python3 samples/lang_slot/gen_lang_slot_probe.py samples/issue2527_empty_linesegs.hwpx \
    samples/issue8697/hft-${out}-probe.hwpx
done
```

## 독립 Print

한컴 2020(11.0.0.9136) Print method 0, 각 4쪽. 2026-11-07 KST.

| 입력 | SHA-256 | Print PDF | SHA-256 |
|---|---|---|---|
| hft-hanyang_junggothic-probe.hwpx | `e05e1580ed8310d82c2bc0a48d4666111788e849fabf247f2de2d73c083508d2` | ../../pdf/issue8697/hft-hanyang_junggothic-probe-2020.pdf | `5682714db337de62ef0b0d2a4b67ea7a896e2b48ac65868e68290e9a0b0faa7b` |
| hft-sinmyeong_junggothic-probe.hwpx | `2efe204f233ceeb19416f5af6abb3fe629bfaf6805038d13352fd6f34c7f3f13` | ../../pdf/issue8697/hft-sinmyeong_junggothic-probe-2020.pdf | `221c16c03b6ca3e91ee6e68e4d224dc6c19188d55ef72df3cebbb7789dd82d85` |

한/글이 그 HFT 프로그램 대신 TrueType(바탕·돋움·한컴바탕)으로 대체해 그린 글자는
`src/renderer/font_metric_legacy_hft.rs` 의 폭 표 근거가 아니다.
