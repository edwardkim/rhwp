# #8020 바탕쪽 편집 시각 근거

## 입력

`samples/issue8020/`의 두 HWP는 이 브랜치의 WASM API로 만들었다. 세 쪽 문서(쪽마다 `본문`)에
바탕쪽을 만들고, 바탕쪽 글은 본문 첫 줄과 겹치지 않게 빈 문단 뒤에 둔다.

| 입력 | 바탕쪽 | SHA-256 |
| --- | --- | --- |
| `master_even_only.hwp` | 짝수 쪽 `짝수바탕` 하나 | `7b9b3e6352b2983b90ed23ece3eced0edb3eccd280c2febd3c6f458d64a9a344` |
| `master_last_optional_overlap.hwp` | 양 쪽 `가양쪽`, 마지막 쪽 `나끝쪽`, 2쪽 겹치게 하기 `다둘째`, 3쪽 맨 위로 보내기 `라셋째` | `2b7b948e67eef305e67eb65cafa089885a9669433c869cc841352aadfc7ed201` |

```js
doc.createMasterPage(0, '{"applyTo":1}');                                   // 짝수 쪽
doc.createMasterPage(0, '{"kind":"page","pageNumber":2,"overlap":true}');  // 임의 2쪽, 겹치게 하기
doc.createMasterPage(0, '{"applyTo":0}');                                   // 양 쪽
doc.createMasterPage(0, '{"kind":"last"}');                                 // 마지막 쪽
doc.createMasterPage(0, '{"kind":"page","pageNumber":3,"front":true}');    // 임의 3쪽, 맨 위로 보내기
doc.insertTextInHeaderFooter(0, false, 16 + n, 0, 0, text);                 // n번째 바탕쪽 글
```

## 기준 PDF

한컴오피스 한글 12.30.0(빌드 6544), macOS 27.2에서 입력을 그대로 열어 파일 > 인쇄 > PDF > PDF로 저장했다.
A4, 100%, 한 쪽씩, 전체 3쪽.

| PDF | SHA-256 | 한컴이 그린 바탕쪽 |
| --- | --- | --- |
| `pdf/issue8020/master_even_only-hancom-12.30-mac.pdf` | `635fc0d34d09efdd5972dc37f51462628f8c283bd111bfe2d7c968006e7aa771` | 1쪽 없음, 2쪽 `짝수바탕`, 3쪽 없음 |
| `pdf/issue8020/master_last_optional_overlap-hancom-12.30-mac.pdf` | `974d40c17f65372f49c2b6822325716444ad7f3f95d363722ae3e307dc9dfa97` | 1쪽 `가양쪽`, 2쪽 `가양쪽`+`다둘째`, 3쪽 `나끝쪽` |

## 실루엣 TSV

`scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 전체 3쪽.

| 입력 | base devel `e00f9b7e2` Native | 후보 Native | 후보 fresh WASM |
| --- | --- | --- | --- |
| `master_even_only` | 35.25 / 21.15 / 35.25 | 100.00 ×3 | 100.00 ×3 |
| `master_last_optional_overlap` | 27.02 / 19.24 / 25.83 | 100.00 ×3 | 100.00 ×3 |

devel은 짝수 쪽 바탕쪽을 세 쪽 모두에 그리고, 2쪽에는 겹치게 하기 바탕쪽만, 3쪽에는 임의 3쪽 바탕쪽을
그린다. 빈 바탕쪽 문단에 줄을 두지 않아 바탕쪽 글도 한 줄 위(164px, 한컴 186px)에 놓인다.

원본: `base/native/`, `head/native/`, `head/wasm/`. 대표 쪽의 review·overlay PNG는 이 폴더에 있다.
후보 producer는 코드 head `eb65e8946`이다(Native `rhwp` release-test + native-skia,
fresh WASM `pkg/rhwp_bg.wasm` SHA-256 `fc7d6be9359fd66b6937c0bfe8473e98f98825470db8529445b0d6b8f0abebbe`).

## 기존 문서 영향

- `samples/`·`tests/fixtures/`의 HWP·HWPX 2,118개를 devel `e00f9b7e2`와 후보(렌더 코드가 같은 앞 커밋
  `1c2f43d4f`)로 전쪽 SVG 출력해 비교했다.
  27개 문서의 SVG가 달랐지만 모두 clip·pattern id 번호만 달랐고, id 번호를 지우면 같다.
  나머지는 바이트까지 같다. 출력 실패 10개(암호·HWP3 일부)는 두 쪽이 같다.
- 편람 HWPX `--handbook-gate`: 383쪽, 최저 99.96%(130쪽), 100.00% 370쪽, #8389 기준 대비 하락 0쪽(`handbook-gate.json`,
  `handbook-hwpx-native.tsv`).
