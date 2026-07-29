# fidelity_compare — 정답지(한컴 공식 PDF) 페이지별 대규모 비교 하네스 (#3389)

`render-diff`(자기 일관성 기하 게이트)를 보완하는 **한컴 정답지와의 절대 정합** 도구다.
한컴이 출력한 공식 PDF 와 rhwp `export-svg` 렌더를 페이지별로 나란히 시트로 만들고,
픽셀 diff% 랭킹으로 **최악 페이지부터 사람이 감사**하게 한다 — 이 루프가 실제로
#3385(PUA 원문자 CharOverlap tofu)를 찾아냈다.

## 요구사항

```bash
pip install pypdfium2 pillow          # PDF 렌더 + 픽셀 diff
# Chrome 설치 필요 (SVG → PNG 캡처). 경로가 다르면 CHROME_BIN 환경변수.
# rhwp 바이너리: target/release-test/rhwp.exe (다르면 RHWP_BIN).
```

## 사용

```bash
python tools/fidelity_compare/fidelity_compare.py <키> <시작쪽> <끝쪽>   # 0 기준, 끝쪽 포함
# 예: 업무계획 전체 35쪽
python tools/fidelity_compare/fidelity_compare.py plan 0 34
```

산출(`output/fidelity/<키>/`): 페이지별 비교 시트 `cmp-pNNN.png`, diff 랭킹 `report.tsv`.
단계별 확장(10쪽 → 전수 → 고난도 문서)으로 돌리고, **랭킹 상위 페이지의 시트를 눈으로
감사**해 실질 결함만 이슈로 승격한다.

등록된 정답지 쌍(`REG`, ASCII 글롭 — 한글 경로 인코딩·NFC/NFD 함정 회피):

| 키 | 문서 | 난도 특성 |
|---|---|---|
| plan | 2022 국립국어원 업무계획 (35쪽) | 보고서 — 표·도해·강조 혼합 |
| bunjang | 보건소 분장사무 별표 | 표 중심 |
| korexam | 법학적성시험 언어이해 기출 (15쪽, **B4**) | 2단 조판·지문 박스·원문자 선지 |
| math | 수학 시험지 (20쪽) | **수식** |
| eng | 영어 시험지 (8쪽) | 라틴 혼합 |

## 실측 기록 (2026-07-26)

- 업무계획 35쪽 전수: 구조·내용·줄바꿈 위치까지 대체로 동일. 최악 페이지 감사에서
  **#3385 발견** (PUA 원문자 U+F02B1~F02C4 가 CharOverlap 문맥에서 tofu).
- math 20쪽: diff 6~11% — 수식 렌더 정합이 강함을 실측.
- korexam 15쪽(B4): 2단·헤더·지문 박스·30문항 구조 재현. 잔여 = 본문 자간/글자폭
  미세 확대로 단 내 줄바꿈이 밀리는 부류 — 폰트 폴백 메트릭 의심.
  `RHWP_FONT_PATH_DIR=<폰트 폴더>` 로 폰트를 고정해 재측정하는 것이 다음 실험.

## 함정 노트 (재현 시 시간 절약)

- SVG 캡처 창은 **SVG 판형을 읽어 자동 맞춤**한다 — 고정 창은 B4 문서를 크롭해
  가짜 diff 를 만든다 (초기 버전의 실수).
- diff% 는 랭킹용이다. 자간 미세 차가 픽셀로 누적되므로 절대값이 아니라
  **순위 + 사람 감사**로 쓴다.
- 배경 셸에서 한글 argv/경로는 cp949 로 깨질 수 있어 키·글롭만 쓴다.
