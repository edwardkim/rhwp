# Task #1700 검증용 한글 문서 (표 직후 빈 문단 페이지 배치 보존)

한컴 한글 2022 정답지 대비 페이지·PI 매칭 검증(`tools/verify_pi_page_vs_hangul.py`)에서
도출한 회귀 방지용 표본. 출처: 공공 공개 문서(법령 별표서식 / 국가유산청 행정규칙 붙임).

| 파일 | 원본 ID | 구조 | 기대 동작(수정 후) |
|------|---------|------|------------------|
| `byeolpyo1_uujeong_wrap_singlepage.hwp` | 17978249 | 캡션 + 어울림(Square) 표 11×2 + 표 직후 빈 문단 (1쪽) | `dump-pages` 에 `WrapAroundPara pi=2` 가 **page 1** 에 표면화. 한글과 문단수·페이지 정합(MATCH). |
| `myeonjeok_wrap_10page.hwp` | 2067603 | 캡션 + 258행 어울림 표(10쪽 분할) + 표 직후 빈 문단 | `WrapAroundPara pi=2` 가 표 끝 **page 10** 에 귀속. 한글과 정합(MATCH). |

## 재현

```bash
cargo build --release
rhwp dump-pages samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp | grep -E "페이지|pi="
# Windows + 한컴오피스 + pyhwpx 환경:
python tools/verify_pi_page_vs_hangul.py --files \
  samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp \
  samples/task1700/myeonjeok_wrap_10page.hwp -o out.tsv
```

수정 전: 표 직후 빈 문단(`pi=2`)이 `dump-pages` 에 누락 → 한글 대비 문단수 off-by-one(PARA_COUNT).
수정 후: 어울림 흡수 문단이 앵커 표의 페이지에 표면화되어 문단→페이지 매핑이 한글과 일치.

> 비고: 한글 편집기 비접근 환경에서는 `dump-pages` 의 `WrapAroundPara` 라인 표면화만으로
> 회귀 확인 가능(한글 대조는 선택).
