# #6919 정식 회귀 입력

[이슈 #6919](https://github.com/edwardkim/rhwp/issues/6919)의 재현물이다.

- 파일: [148726703-chart-preview.wmf](148726703-chart-preview.wmf) (4,286 B)
- 출처: `148726703_sjcu1112.hwp`(산업활동동향 브리핑) **6쪽** OLE 차트 개체의
  `\x02OlePres000` 미리보기 metafile 을 원본 바이트로 잘라 냈다
- SHA-256: `f8a7084f35f4a8d1fba9b0901c3cd3df0b2af44994a2333fb2030bd3f1263408`

## 문서가 아니라 metafile 을 등록한 이유

이 축은 **WMF 재생기**의 계약이고, 원본 문서는 87쪽 3.2 MB 라 렌더 래칫 전수 스윕에
그만큼의 비용을 얹는다. 잘라 낸 metafile 은 같은 결함을 그대로 담으면서 잠금 시험이
직접 읽는다.

```text
  표준 WMF(placeable 아님) · 4,286 B
  META_SETTEXTALIGN(0x012E)   0개     ← 결함의 조건
  글자 레코드(TEXTOUT/EXTTEXTOUT)  9개
```

## 정답지는 GDI+ 다

이 축의 정답지는 한컴 PDF 가 아니라 **Windows GDI+**(WMF 표준 재생기)와
[MS-WMF] 2.1.2.18 이다. 같은 WMF 를 `System.Drawing.Image.FromFile` 로 재생하면
제목이 테두리 아래에 온전히 들어간다(이슈 본문 실측: 잉크 y = 9.5 .. 20.0, 테두리 y=5).

⚠ 이 문서는 **한컴 변환 자체가 실패**한다(`HwpEditorFrame.SaveAsDocument returned
false`). 기준 PDF 를 만들 수 없어 담지 않았다.

## 회귀 계약

[MS-WMF] 2.1.2.18 의 재생 DC 기본 정렬은 `TA_LEFT | TA_TOP | TA_NOUPDATECP`(=`0x0000`)
이고 `TA_BASELINE`(0x0018)이 아니다. 곧 `META_EXTTEXTOUT` 의 `y` 는 **글자 셀의 위끝**
이며 baseline 은 `y + ascent` 다.

```text
  제목 레코드  META_EXTTEXTOUT x=110 y=13 · font height = -12
  수정 전      <text y="13">    테두리를 글자가 가로지른다
  수정 후      <text y="22">    = 13 + 0.8 × 12
```

잠금 시험은 세 갈래를 잰다.

1. `SetTextAlign` 없음 ≡ 명시 `TA_TOP(0x0000)` — 합성 metafile
2. **음성 대조** — 명시 `TA_BASELINE(0x0018)` 은 레코드 `y` 를 그대로 쓴다
3. 이 재현물에 `SetTextAlign(TA_BASELINE)` 을 한 줄 끼운 변형과 견줘 **모든 글자가
   내려가는지** — 끼운 변형이 곧 수정 전 출력이다

## 코퍼스 실측 (10,000건)

```text
  WMF 를 담은 문서                                     128건 (metafile 2,471개)
  그중 SetTextAlign 없이 글자를 쓰는 metafile 보유       42건 (metafile 1,022개)
```

`SetTextAlign` 을 부르는 metafile 은 그 핸들러(`v_bits == 0 → VTA_TOP`)가 비트를 그대로
읽으므로 **불변**이다. 바뀌는 것은 위 42문서의 metafile 뿐이고, 그 전부가 지금까지
글자를 한 줄 위로 올려 그리고 있었다.
