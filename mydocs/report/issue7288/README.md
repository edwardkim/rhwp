# #7288 시각 증적 — 표 «쪽 경계에서» 값 0 «나누지 않음»

## 실행한 것

| 항목 | 값 |
|---|---|
| source head | `4d74bea62` (base `1966af77f`) |
| 입력 | `samples/task1725/text_footnote_tail_overpagination.hwp` |
| 기준 PDF | `pdf/text_footnote_tail_overpagination-2024.pdf` (`Hwp 2024 13.0.0.3622`) |
| 쪽 | 62~64 (Native, DPI 96) |
| 명령 | `python scripts/visual_sweep.py --file-target task1725 <hwp> <pdf> --rhwp-bin <release-test rhwp> --pages 62-64 --dpi 96 --out <out>` |
| 수정 전 바이너리 | 같은 worktree 의 `1966af77f`(devel) 빌드 |

두 번 실행해(수정 전·후) compare·overlay·review 를 산출하고 직접 열어 판독했다.

## 판독 — 표 뒤 마지막 문단

`overlay_p63_before.png` / `overlay_p63_after.png` 는 같은 쪽 같은 영역의 overlay 다.

- **수정 전**: `러 싸여야 한다. 그러나, 그러한 구역의 격리 격벽이 30% 이상 개방되어 있으면,`
  줄 전체가 빨간 불일치 상자로 덮여 있다 — rhwp 가 이 줄을 정본보다 한 줄(25.6px) 아래에
  그렸다.
- **수정 후**: 같은 줄에서 빨간 상자가 사라지고 정본과 겹친다.

윗줄(`고속 화물선의 …` 캡션)에 남은 빨간 흔적은 수정 전·후가 동일하며, 아래 «한계»의
쪽 짝짓기 오프셋에서 온다.

좌표로도 같은 결론이다 — 저장 사다리가 지시하는 `pi=1374` 의 절대 위치 `364.12px`
(`21640HU / 75 + 본문 상단 75.5867px`)와 정본 PDF 의 같은 줄 `363.9px`
(`272.93pt × 96/72`)가 0.3px 안에서 일치하고, 수정 후 rhwp 가 `364.1px` 에 그린다.
수정 전은 `389.7px` 였다.

## 한계 — 이 sweep 으로 판정하지 않은 것

- **쪽 짝짓기가 한 쪽 어긋난다.** 이 문서는 rhwp 242쪽 · 정본 PDF 243쪽이라, sweep 의
  `N ↔ N` 짝짓기가 발산 지점 뒤에서 맞지 않는다. 따라서 **pixel/ink 자동 보조값
  (5.88% → 5.96%)은 이 변경의 지표가 아니다.** 위 판독은 같은 쪽 안에서 같은 문단을
  직접 대조한 것이고, 좌표 대조는 본문 앵커로 정렬했다.
- **Native 단독이다.** fresh WASM 비교는 실행하지 않았다.
- 62·64쪽은 수정 전후 차이가 없었다.
