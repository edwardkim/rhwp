# #7333 Stage 5 — 글앞 장식 TAC 표 뒤 본문 흐름

## 분석

13·16~21쪽에서 글앞(`InFrontOfText`) 장식 도형 뒤의 글자처럼 표(TAC)는 기준 PDF의 표 위치와
일치했지만, 표 뒤 첫 본문 줄이 14.7px 또는 16.0px 아래에 배치됐다. 이 형상에서는 장식 도형이
호스트 줄 흐름에 참여하지 않으므로 기존 `tac_seg_applied`의 `host_seg`가 의도적으로 비어 있다.
따라서 표의 물리 하단과 `outer_margin_bottom`만 흐름에 더해지고, 표의 실제 저장 LineSeg에 있던
음수 Fixed 줄간격(`-1100` 또는 `-1200` HWPUNIT)이 빠졌다.

수정은 HWP5 저장 조판, TAC, 앞선 컨트롤이 글앞 Shape뿐, 표 소유 LineSeg가 표와 상하 외곽여백을
포함하고 음수 줄간격인 경우로 제한한다. 표 paint와 외곽여백은 유지하고, 뒤 문단의 흐름에서만
그 음수 간격을 공제한다. 양수 줄간격 및 일반 가시 개체 조합은 기존 경로를 사용한다.

## 결과

- `src/renderer/layout.rs`: 위 저장 계약을 판별하고 TAC 흐름의 마지막 여백 계산 뒤에만 공제했다.
- `tests/cases/issue_7333_overlapping_picture_lines.rs`: 13·16~21쪽 첫 본문 `TextLine`의 PDF 기준 y를
  회귀 검사로 고정했다. 수정 전 13쪽은 `868.2px`로 실패했고 수정 후 결과는 다음과 같다.

| 쪽 | RHWP y | Hancom 2020 PDF y | 차이 |
| --- | ---: | ---: | ---: |
| 13 | 853.6px | 853.5px | +0.1px |
| 16 | 835.4px | 835.4px | 0.0px |
| 17 | 814.5px | 814.5px | 0.0px |
| 18 | 816.8px | 816.8px | 0.0px |
| 19 | 662.5px | 662.5px | 0.0px |
| 20 | 685.2px | 685.2px | 0.0px |
| 21 | 729.0px | 729.0px | 0.0px |

## 검증

```text
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test \
  --test regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture
# 4 passed; 0 failed

CARGO_TARGET_DIR=target/pr-review cargo build --locked --release --bin rhwp

python3 scripts/visual_sweep.py --key issue7333-fixed-line-spacing \
  --hwp samples/issue7333/aaaaaa.hwp \
  --pdf pdf/issue7333/aaaaaa-2020.pdf \
  --rhwp-bin target/pr-review/release/rhwp \
  --pages 13,16,17,18,19,20,21 --dpi 96 \
  --out /tmp/rhwp-issue7333-fixed-line-spacing-20260922
# 표 밖 본문 줄 밴드: 최대 1.5px

python3 scripts/visual_sweep.py --key issue7333-decoration-regression \
  --hwp samples/issue7333/aaaaaa.hwp \
  --pdf pdf/issue7333/aaaaaa-2020.pdf \
  --rhwp-bin target/pr-review/release/rhwp \
  --pages 14,22,23 --dpi 96 \
  --out /tmp/rhwp-issue7333-decoration-regression-20260922
# 표·빨간 주석 위치 유지

python3 scripts/visual_sweep.py --key issue7333-stage5-full \
  --hwp samples/issue7333/aaaaaa.hwp \
  --pdf pdf/issue7333/aaaaaa-2020.pdf \
  --rhwp-bin target/pr-review/release/rhwp \
  --pages 1-50 --dpi 96 \
  --out /tmp/rhwp-issue7333-stage5-full-20260922
# run_state=complete, 50/50, structural flags=0
```

전체 스윕의 평균 pixel match는 93.51%, 최저 84.87%이며, 글리프 래스터와 기존 이미지 차이를 포함한
지표다. 이번 수정 범위의 표 뒤 본문 위치는 위의 PDF 독립 좌표와 focused overlay로 판정했다.
