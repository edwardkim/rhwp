# #9087 HFT 「신명 순명조」 폭 탐침

일곱 언어 슬롯을 모두 HFT 「신명 순명조」로 둔 합성 입력이다. 문단마다 `가<글자>가` 를 적어,
독립 한컴 Print PDF 의 Type3 `/Widths`·ToUnicode 와 글자 원점으로 그 글꼴 프로그램이 직접
그리는 글자와 전진폭을 읽는다. 저장 줄 정보는 없다. 굵게 탐침은 글자 모양에 `<hh:bold/>` 만 더했다.

## 생성

`samples/issue8697/README.md` 의 `CHARS` 를 그대로 쓴다.

```sh
name='신명 순명조'
FTYPE=HFT FACES="$name,$name,$name,$name,$name,$name,$name" CHARS="$CHARS" PATTERN='{c}{x}{c}' \
  python3 samples/lang_slot/gen_lang_slot_probe.py samples/issue2527_empty_linesegs.hwpx \
  samples/issue9087/hft-sinmyeong_sunmyeongjo-probe.hwpx
```

## 독립 Print

한컴오피스 2022(12.0.0.4547) Print → Hancom PDF, 각 4쪽. 2026-11-15 KST.

| 입력 | SHA-256 | Print PDF | SHA-256 |
|---|---|---|---|
| hft-sinmyeong_sunmyeongjo-probe.hwpx | `94c4832df0fcf621b47d25cbafa49820d066fea94070fa8525063f2809934cf2` | ../../pdf/issue9087/hft-sinmyeong_sunmyeongjo-probe-2022.pdf | `452361f51b1a62832f9cf7175dcfcfd773b9799117ea015e6ddb113566083fd0` |
| hft-sinmyeong_sunmyeongjo-bold-probe.hwpx | `fa1aa40d06b8617f628a18b6e2263e072725586f90252d939b8ce89f8584a245` | ../../pdf/issue9087/hft-sinmyeong_sunmyeongjo-bold-probe-2022.pdf | `cc5f80351a7ebdaf66bb3c25074cb72c8cc1be39dec577e598f7fe7049d090ff` |

한/글이 이 HFT 프로그램 대신 TrueType(한컴바탕)으로 대체해 그린 글자는
`src/renderer/font_metric_legacy_hft.rs` 의 폭 표 근거가 아니다.
