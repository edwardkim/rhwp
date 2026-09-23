# #7095 비끝 조각 상자의 줄 뒤 간격 — Visual Sweep 증적

검증 head `f7f01eaea` (PR #7360 위). 정본은
`pdf/issue2430/1382000_domestic_violence_survey-2020.pdf`
(`Hancom PDF 1.3.0.550` / `Hwp 2020 0.0.0.0`, `hancom_version 11.0.0.9136`,
`sha256 59baa27df0aa73f776af7e3db6ab539824c50012ef17b609d7a0d812cb92472e`).

```bash
RHWP_FONT_PATH=ttfs/hwp:ttfs/windows venv/bin/python scripts/visual_sweep.py \
  --hwp samples/task2430/1382000_domestic_violence_survey.hwp \
  --pdf pdf/issue2430/1382000_domestic_violence_survey-2020.pdf \
  --rhwp-bin target/release/rhwp --svg-rasterizer rsvg --pages 16-19
```

| 파일 | 내용 |
| --- | --- |
| `native_1382000_border_016_before_after.png` | **16쪽 상자 하단 확대** — 위 수정 전, 아래 수정 후 |
| `native_1382000_overlay_016.png` | 16쪽 overlay — `pixel_match 93.586%` |
| `native_1382000_overlay_017.png` | 17쪽 overlay |
| `native_1382000_overlay_019_terminal.png` | 19쪽 **끝 조각** overlay — 이 변경으로 불변 |

## 판독

**16쪽 확대(대표).** 수정 전에는 정본 상자 하단(파랑)과 rhwp(빨강)가 확연히 갈렸다. 수정 뒤에는
조각 마지막 줄 바로 아래에서 두 선이 겹친다.

**16·17쪽 괘선.**

| 쪽 | 수정 전 | 수정 후 | 정본 | 차 |
| --- | ---: | ---: | ---: | ---: |
| 16 | 989.81 | **975.15** | 977.07 | 12.74 → **1.92** |
| 17 | 992.76 | **978.09** | 979.95 | 12.81 → **1.86** |
| 19 (끝 조각) | 984.07 | 984.07 | 986.03 | 불변 **1.96** |

세 쪽의 잔차가 같은 계통이다.

## 남은 차이

- **18쪽**은 여전히 1005.92(정본 970.99)다. 마지막 유닛이 중첩 표(`para=43` h=141.1)인데 실제
  표 높이는 92.4px(872.00 .. 964.40)로 **48.7px 크다**. 줄간격 하나로 설명되지 않는 별도 축이라
  **미검증**으로 남긴다. 그 쪽의 중첩 표 자체는 정본과 0.9px 안에 있다.
- 가로로 글자가 벌어지는 것(`ink_match` 12~18%)은 한글 전진폭 축이고 세로 조판과 독립이다.
