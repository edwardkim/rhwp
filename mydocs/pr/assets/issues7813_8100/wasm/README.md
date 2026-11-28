# 5건의 fresh WASM Canvas 전쪽 검증

원본 5건·10쪽을 한컴 Print PDF와 대조했다. 전쪽 최저는 **98.12%**다.
빌드 소스 커밋은 `592e625dda60d88181777f892878fe4d8d6f6364`이며 리베이스 기준은 `274ae9d4c83154e06da049c90eed6904a8b2ec89`다.
실행물 SHA-256은 `0948dfb6e5a9efdae96ad4ddcd211fb39875cf28e9d6e5b1ad78589c48fd4411`다. 입력·기준 PDF·TSV·대표 PNG SHA와 글꼴 공급 순서는 [검증 매니페스트](manifest.json)에 기록했다.

지표는 96dpi, 색상 경계 보정 전 2px 내용 실루엣 일치율이다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp <원본> --pdf <한컴 Print PDF> --rhwp-bin <고정 Native 실행 파일> --wasm-pkg <고정 WASM 패키지> --embed-fonts full --font-path <공급 파일> --compat <문서별 값> --dpi 96
```

| 이슈·입력 | 쪽수 | 최저 | 전쪽 TSV | 대표 비교 PNG |
| --- | ---: | ---: | --- | --- |
| #7821 · `7821` | 3 | 98.12% | [7821.tsv](7821.tsv) | [1쪽](7821-p001.png) |
| #7822 · `7822` | 4 | 100.00% | [7822.tsv](7822.tsv) | [1쪽](7822-p001.png) |
| #7863 · `7863` | 1 | 100.00% | [7863.tsv](7863.tsv) | [1쪽](7863-p001.png) |
| #7864 · `7864` | 1 | 100.00% | [7864.tsv](7864.tsv) | [1쪽](7864-p001.png) |
| #7922 · `7922` | 1 | 99.16% | [7922.tsv](7922.tsv) | [1쪽](7922-p001.png) |
