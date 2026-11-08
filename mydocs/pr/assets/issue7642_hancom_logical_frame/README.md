# Issue #7642: HancomGlyphs 논리 프레임의 Native 보정 대조

- 생산 코드: `a9732e2425` (공통 판별 세 곳에 새 일반 글리프 런 연결).
- 작은 괄호: `pdf/issue7642-small-delimiter-diagnostic/small-delimiter-hancom-2020.hwp` 및 같은 이름 PDF. Native 1쪽 48.48540% → 98.59805%.
- 수식 문맥: `pdf/issue7642-term-context-diagnostic/term-context-hancom-2020.hwp` 및 같은 이름 PDF. Native 전 3쪽 최저 82.95174% → 97.18524%.
- 2022년 9월: `samples/3-09월_교육_통합_2022.hwp`, `mydocs/pr/assets/pr_7600_control_print_20261006/input-01-3-09월_교육_통합_2022-2020.pdf`. Native 전 23쪽 최저 49.15194% → 91.76071%, 쪽수 일치.
- 한컴 2020 Print에 대응하는 `--compat 2022`, 인쇄 프로필, 전체 글꼴 공급, 96 DPI.
- 작은 괄호의 9/11/16/22pt 혼합 본체·첨자·분수와 후속 줄의 누적 세로 이동 해소를 직접 판독했다. 수식 문맥과 시험지의 glyph·미세 간격 확대 판독은 계속한다.
- 수식 진단 9개·12쪽의 fresh WASM도 모두 통과했다(최저 93.09138%). 나머지 원본 전쪽 검증과 확대 판독·필수 회귀/lint는 진행 중이며 전체 완료·승인을 판정하지 않는다.
- TSV·입력/PDF/바이너리 해시·전송 증거: ignored stage475 및 stage479. 이 PNG는 그 측정의 원본 PNG로 canonical 비교 함수를 통해 합성했다.
