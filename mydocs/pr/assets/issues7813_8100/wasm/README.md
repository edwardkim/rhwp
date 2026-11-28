# 11건의 fresh WASM Canvas 전쪽 검증

원본 19건·512쪽을 한컴 Print PDF와 대조했다. 전쪽 최저는 **98.04%**다.
빌드 소스 커밋은 `592e625dda60d88181777f892878fe4d8d6f6364`이며 리베이스 기준은 `274ae9d4c83154e06da049c90eed6904a8b2ec89`다.
실행물 SHA-256은 `0948dfb6e5a9efdae96ad4ddcd211fb39875cf28e9d6e5b1ad78589c48fd4411`다. 입력·기준 PDF·TSV·대표 PNG SHA와 글꼴 공급 순서는 [검증 매니페스트](manifest.json)에 기록했다.

지표는 96dpi, 색상 경계 보정 전 2px 내용 실루엣 일치율이다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp <원본> --pdf <한컴 Print PDF> --rhwp-bin <고정 Native 실행 파일> --wasm-pkg <고정 WASM 패키지> --embed-fonts full --font-path <공급 파일> --compat <문서별 값> --dpi 96
```

| 이슈·입력 | 쪽수 | 최저 | 전쪽 TSV | 대표 비교 PNG |
| --- | ---: | ---: | --- | --- |
| #7813 · `7813-hwp` | 115 | 98.29% | [7813-hwp.tsv](7813-hwp.tsv) | [6쪽](7813-p006.png) |
| #7813 · `7813-hwpx` | 115 | 98.04% | [7813-hwpx.tsv](7813-hwpx.tsv) | — |
| #7821 · `7821` | 3 | 98.12% | [7821.tsv](7821.tsv) | [1쪽](7821-p001.png) |
| #7822 · `7822` | 4 | 100.00% | [7822.tsv](7822.tsv) | [1쪽](7822-p001.png) |
| #7830 · `7830` | 29 | 98.95% | [7830.tsv](7830.tsv) | [8쪽](7830-p008.png) |
| #7831 · `7831` | 39 | 98.12% | [7831.tsv](7831.tsv) | [19쪽](7831-p019.png) |
| #7834 · `7834` | 15 | 99.98% | [7834.tsv](7834.tsv) | [6쪽](7834-p006.png) |
| #7863 · `7863` | 1 | 100.00% | [7863.tsv](7863.tsv) | [1쪽](7863-p001.png) |
| #7864 · `7864` | 1 | 100.00% | [7864.tsv](7864.tsv) | [1쪽](7864-p001.png) |
| #7922 · `7922` | 1 | 99.16% | [7922.tsv](7922.tsv) | [1쪽](7922-p001.png) |
| #7955 · `7955` | 3 | 99.85% | [7955.tsv](7955.tsv) | [1쪽](7955-p001.png) |
| #8100 · `8100-added-issue7677-float_push_spacing_before_continuation-2024-hwp` | 6 | 99.76% | [8100-added-issue7677-float_push_spacing_before_continuation-2024-hwp.tsv](8100-added-issue7677-float_push_spacing_before_continuation-2024-hwp.tsv) | — |
| #8100 · `8100-added-issue7677-float_push_spacing_before_continuation-hwpx` | 6 | 100.00% | [8100-added-issue7677-float_push_spacing_before_continuation-hwpx.tsv](8100-added-issue7677-float_push_spacing_before_continuation-hwpx.tsv) | — |
| #8100 · `8100-added-issue7678-float_push_spacing_before_continuation-2024-hwp` | 8 | 99.76% | [8100-added-issue7678-float_push_spacing_before_continuation-2024-hwp.tsv](8100-added-issue7678-float_push_spacing_before_continuation-2024-hwp.tsv) | — |
| #8100 · `8100-added-issue7678-float_push_spacing_before_continuation-hwpx` | 8 | 100.00% | [8100-added-issue7678-float_push_spacing_before_continuation-hwpx.tsv](8100-added-issue7678-float_push_spacing_before_continuation-hwpx.tsv) | — |
| #8100 · `8100-added-issue7679-tac-after-text-hwpx` | 3 | 100.00% | [8100-added-issue7679-tac-after-text-hwpx.tsv](8100-added-issue7679-tac-after-text-hwpx.tsv) | — |
| #8100 · `8100-added-issue7695-footnote-page-tail-hwpx` | 32 | 100.00% | [8100-added-issue7695-footnote-page-tail-hwpx.tsv](8100-added-issue7695-footnote-page-tail-hwpx.tsv) | — |
| #8100 · `8100-added-issue7874-omitted-spacing-after-hwpx` | 2 | 100.00% | [8100-added-issue7874-omitted-spacing-after-hwpx.tsv](8100-added-issue7874-omitted-spacing-after-hwpx.tsv) | — |
| #8100 · `8100-counter-22081993` | 121 | 98.41% | [8100-counter-22081993.tsv](8100-counter-22081993.tsv) | [92쪽](8100-p092.png) |
