# Stage 125 - 2025 행정업무운영 편람 383쪽 조판 분석

## 수용 기준

rhwp의 페이지네이션 결과가 Hancom Office 2020 기준 PDF의 383쪽과 일치해야 한다. HWP5 저장 후 Hancom PDF의 fidelity와 rhwp 자체 renderer의 page count를 별도 지표로 유지한다.

## 고정한 측정값

| 대상 | rhwp pages | 기준 대비 |
| --- | ---: | ---: |
| 원본 HWPX | 386 | +3 |
| 원본 HWP | 393 | +10 |
| 기준 PDF | 383 | 0 |

원본 HWP의 HWPX 대비 +7은 section 7(+1), section 10(+2), section 11(+4)의 표 분할에 모인다. `--respect-vpos-reset`은 HWPX/HWP 어느 쪽의 page count도 바꾸지 않았다.

## 기준 PDF와 HWPX 흐름 대조

section 11의 103행 x 2열 규정 대조표를 anchor로 비교했다.

| 구간 | Hancom 2020 PDF | rhwp HWPX | 차이 |
| --- | --- | --- | ---: |
| 규정표 시작 | physical p311 | physical p321 | rhwp가 +10쪽 늦음 |
| 규정표 점유 | p311-p367, 57쪽 | p321-p369, 49쪽 | rhwp가 8쪽 덜 사용 |
| 규정표 뒤 section 시작 | p368 부근 | p370 | rhwp가 +2쪽 |
| 문서 끝 | p383 | p386 | rhwp가 +3쪽 |

즉 단순한 전역 height factor나 페이지 수 강제 보정은 금지한다. section 10 전후의 과도한 페이지 분할, 103행 표의 과도한 압축, 표 이후 한 페이지 잔차가 서로 상쇄되어 현재 총 +3이 된다.

## 대표 표의 모델 사실

- section 7 para 239: 3 x 7 table, RowBreak, 140.1 x 68.9 mm. HWPX/HWP의 의미 모델은 같지만 HWP raw common attr와 paragraph attr2 tail이 추가로 존재한다.
- section 10 para 4: 1 x 1 table, 73 문단, RowBreak, 140.1 x 168.4 mm. HWPX table attr는 `0x04000006`, HWP는 `0x00000006`이다.
- section 11 para 3: 103 x 2 table, RowBreak, 215.9 x 145.7 mm. HWPX table attr는 `0x04000006`, HWP는 `0x00000006`이다.

section 7에서도 HWP/HWPX page difference가 있으므로 table attr bit 26을 전체 원인으로 단정하지 않는다.

## 다음 구현 게이트

1. section 10과 section 11에서 첫 번째 서로 다른 page boundary를 PDF text anchor와 rhwp `partialTable` split state로 각각 특정한다.
2. 하나의 renderer/table pagination 책임으로 원인을 좁힌 뒤, 축소 fixture와 회귀 테스트를 먼저 만든다.
3. 수정 후 HWPX 383쪽, 원본 HWP 383쪽을 각각 측정한다. 어느 하나만 맞추는 source-specific 우회는 수용하지 않는다.
4. 그 후 Stage 126 결과 보고서에 page count, bit 28 raw record, Hancom raster metric을 함께 기록하고 커밋한다.
## Stage 125-2: direct HWP deferred-fragment trace (2026-08-11)

- The direct HWP `DECL_DEFER` path was confirmed at section 7 paragraph 239 and section 10 paragraphs 4 and 14.
- A stored-anchor guard changed section 10 paragraphs 4 and 14 from fresh-page placement to the same current-page fragment scan used by the HWPX input (`remaining=616.4` and `174.1` respectively), but total counts remained `HWP 393` and `HWPX 386`.
- Therefore those two local defers are layout divergence evidence, not an independently removable page-count surplus. The guard remains under investigation and is not accepted as a final count correction.
- The temporary `RHWP_DIAG_SCAN` instrumentation used to prove the branch was removed before the next baseline comparison.
## Stage 125-3: executable 383-page baseline (2026-08-11)

- Detached worktree `07555d200` was built with an isolated target directory and executed against the same fixture names. Its legacy CLI reports `2025 행정업무운영 편람(최종).hwp (383페이지)` and `2025 행정업무운영 편람(최종).hwpx (383페이지)`.
- The current renderer reports `HWP 393` and `HWPX 386`; therefore the 383-page PDF agreement was previously achievable in rhwp and the present mismatch is a pagination regression, not a source-file or reference-PDF mismatch.
- The direct-HWP stored-anchor probe moved section 10 paragraphs 4 and 14 into current-page fragment scanning, but did not change either total. It was removed and is not part of an implementation commit.
- The source interval includes major pagination extraction/refactor work, so wholesale reversion is unsafe. The next implementation stage must isolate a common behavior delta with executable count checkpoints, then retain only a fixture-independent rule that restores both HWP and HWPX to 383.
