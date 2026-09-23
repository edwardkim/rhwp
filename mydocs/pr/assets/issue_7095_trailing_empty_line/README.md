# #7095 끝 조각 상자의 꼬리 빈 문단 줄 상자 — Visual Sweep 증적

검증 head `5be01b064` (PR #7354 위). 정본은
`pdf/issue2430/1382000_domestic_violence_survey-2020.pdf`
(`Hancom PDF 1.3.0.550` / `Hwp 2020 0.0.0.0`, `hancom_version 11.0.0.9136`,
`sha256 59baa27df0aa73f776af7e3db6ab539824c50012ef17b609d7a0d812cb92472e`).

```bash
RHWP_FONT_PATH=ttfs:hwp:ttfs/windows venv/bin/python scripts/visual_sweep.py \
  --hwp samples/task2430/1382000_domestic_violence_survey.hwp \
  --pdf pdf/issue2430/1382000_domestic_violence_survey-2020.pdf \
  --rhwp-bin target/release/rhwp --svg-rasterizer rsvg --pages 16-19
```

| 파일 | 내용 |
| --- | --- |
| `native_1382000_compare_019.png` | 19쪽 rhwp ↔ 정본 나란히 비교 (수정 후) |
| `native_1382000_overlay_019.png` | 19쪽 overlay — `pixel_match 94.377%` |
| `native_1382000_overlay_016_control.png` | 16쪽 대조군 overlay — `pixel_match 93.583%` |

## 판독

**19쪽(끝 조각).** 표 상자 하단이 정본과 맞는다. 13줄 구성(항목 4·5·6·7 이 각 2줄)도 정본과
같고, 이는 문서의 저장 `LineSeg` 와도 같다. 괘선 실측 `972.07 → 984.07`(정본 986.03).

**16쪽(대조군, 비끝 조각).** 이 변경으로 움직이지 않는다 — 수정 전후 모두 괘선 989.81 이다.
다만 정본 977.07 보다 12.7px 아래에서 끝난다. 이 차이는 **이 PR 의 회귀가 아니다**: 같은
문서·같은 기준으로 수정 전에도 989.81 이었다. 별도 축이며 #7095 코멘트에 남긴다.

## 남은 차이

두 쪽 모두 **가로**로 글자가 벌어진다(`ink_match` 12~18%). rhwp 의 한글 전진폭이 정본과
달라서이고 세로 조판과는 독립이다. 서명 블록의 가로 위치(rhwp 왼쪽 정렬 ↔ 정본 가운데)도
같은 축이다. 이 PR 은 세로 축(조각 상자 높이)만 주장한다.
