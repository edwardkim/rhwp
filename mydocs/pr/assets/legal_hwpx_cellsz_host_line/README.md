# 법제처 HWPX — cellSz 행 높이와 글 뒤로 그림 host 줄 (PR #7905 증적)

[PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 의 시각 검증 증적이다. 판정과 해석은 PR 본문에 있고,
이 폴더는 본문이 인용하는 입력·기준 PDF·TSV 원본·대표 이미지를 보존한다.

## 입력과 기준 PDF

| 파일 | 설명 | 바이트 | SHA-256 |
| --- | --- | ---: | --- |
| `input/jongbu_law.hwpx` | 국가법령정보센터(법제처) 종합부동산세법 시행규칙 HWPX 6쪽. `linesegarray` 없음, `content.hpf` 메타데이터는 자리표시자 | 140149 | `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1` |
| `input/jongbu_law_hancom2018_print.pdf` | 한컴오피스 2018(`Hwp 2018 10.0.0.14727`) Print 경로 출력(`Hancom PDF 1.3.0.550`, A4 6쪽) | 287952 | `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511` |

기준 PDF 는 한컴이 출력한 원본(SHA-256 `d7b9c88e32cead0f146d4951257dddc13dd7084866349a7d7815167116bc196a`)에서
**작성자 메타데이터(Info `/Author`, XMP `dc:creator`)만** 같은 길이의 `n/a` 로 바꾼 사본이다(20바이트 차이).
쪽 내용 스트림은 건드리지 않았다. pdfium 96dpi 로 6쪽을 모두 그려 원본과 픽셀이 같음을 확인했고,
Native 후보 TSV 를 이 사본으로 다시 산출해 원본 PDF 로 낸 TSV 와 바이트가 같음(`0703380b…`)을 확인했다.

## TSV 원본 (`--silhouette-only`, print profile, `--compat 2022`, 96dpi, 6쪽 전체)

CRLF 인 Windows 생성 원본이며 바이트를 변환하지 않는다(`.gitattributes` 의 `-text`). 각 263바이트.

| 파일 | 기준/후보 | 경로 | SHA-256 |
| --- | --- | --- | --- |
| `base/native/jongbu_law_silhouette.tsv` | 기준 `devel` `efc52afb7` | Native Skia | `33ffb17bcb54c3b1eb81ccb35b4458e545e5e54216c732664768b3c09ded3333` |
| `native/jongbu_law_silhouette.tsv` | 후보 `cd88822c8` | Native Skia | `0703380bbffeccd10ed611675954259563c5aa4b8e40b550e9d33d7b1531c9ef` |
| `base/wasm/jongbu_law_silhouette.tsv` | 기준 `devel` `efc52afb7` | fresh WASM(Canvas2D) | `b3b4c6de18d4cac961139b33026b0541cf0623fbcfc0f1aea9a955fabccd97ee` |
| `wasm/jongbu_law_silhouette.tsv` | 후보 `cd88822c8` | fresh WASM(Canvas2D) | `de061f8030cf13fb9ee831992e7be9a97b3775bfcc9c9ba2af79e35fdf26c504` |

producer 는 후보 `cd88822c8` 이다. 이후 커밋은 `tests/`·`mydocs/` 만 바꾸므로 `git diff cd88822c8 HEAD -- src` 는 비어 있다.

| 빌드 산출물 | SHA-256 |
| --- | --- |
| 기준 Native `rhwp_base_skia.exe` (`--features native-skia`, `efc52afb7`) | `ab090adfc38504cc2e1e98f8a803f0decfe04879c064c239382944767a24bca8` |
| 후보 Native `rhwp_headfinal_skia.exe` (`--features native-skia`, `cd88822c8`) | `dcb0e0534b64500388ff430cbf622ed8eb01b251ab3ce799de44ed8583327f45` |
| 기준 `pkg/rhwp_bg.wasm` (`efc52afb7`, `--dev`) | `d8c8f6dc3a7a8e37d5518c54531090b29d868c22253c69e773826e9bcd0e8bcd` |
| 후보 `pkg/rhwp_bg.wasm` (`cd88822c8`, `--dev`) | `3826772e6d15c590e59fca625d9043e6c05b903244f333896199d4619e81d546` |
| `pkg/rhwp.js` (기준·후보 동일) | `d9c9011fe3d0e4b0b364b9da00e7d812d2962c3f1f96164d3a4f0f41a2634fcc` |

## 이미지 (같은 입력·쪽)

- `jongbu_law-pNNN-{native,wasm}-{review,overlay}.png` — 후보의 Native Skia·fresh WASM review(rhwp | 한컴 PDF | 겹침)와 겹침 이미지
- `jongbu_law-pNNN-base-native-{review,overlay}.png` — 기준(`devel`)의 Native Skia 같은 쪽
- `jongbu_law-p001-skia.png` — 후보의 실제 Native Skia print PNG, `jongbu_law-p001-canvas2d.png` — 후보 WASM 으로 띄운 Studio Canvas2D(legacy) 화면
- `jongbu_law-pNNN-zoom-*-{native,wasm}.png` — 한컴 Print PDF(위)와 rhwp(아래)를 같은 영역·같은 2배로 자른 확대 대조

## 재현 형태

```bash
python scripts/visual_sweep.py --silhouette-only \
  --hwp mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx \
  --pdf mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf \
  --key jongbu_law --rhwp-bin <native-skia rhwp> [--wasm-pkg pkg] --compat 2022 --dpi 96 --out <출력 폴더>
```

---
최종 수정일: 2026-10-19
