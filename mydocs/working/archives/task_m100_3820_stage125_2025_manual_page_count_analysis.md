# Stage 125 - 2025 행정업무운영 편람 383쪽 조판 분석

## 수용 기준

rhwp의 페이지네이션 결과가 Hancom Office 2020 기준 PDF의 383쪽과 일치해야 한다. HWP5 저장 후 Hancom PDF의 fidelity와 rhwp 자체 renderer의 페이지 수는 별도 지표로 유지한다.

## 고정한 측정값

| 대상 | rhwp 페이지 수 | 기준 대비 |
| --- | ---: | ---: |
| 현재 원본 HWPX | 386 | +3 |
| 현재 원본 HWP | 393 | +10 |
| Hancom 2020 기준 PDF | 383 | 0 |

원본 HWP의 HWPX 대비 +7은 section 7(+1), section 10(+2), section 11(+4)의 표 분할에 모인다. `--respect-vpos-reset`은 HWPX/HWP 어느 쪽의 페이지 수도 바꾸지 않았다.

## 기준 PDF와 HWPX 흐름 대조

section 11의 103행 x 2열 규정 대조표를 anchor로 비교했다.

| 구간 | Hancom 2020 PDF | 현재 rhwp HWPX | 차이 |
| --- | --- | --- | ---: |
| 규정표 시작 | physical p311 | physical p321 | rhwp가 +10쪽 늦음 |
| 규정표 점유 | p311-p367, 57쪽 | p321-p369, 49쪽 | rhwp가 8쪽 덜 사용 |
| 규정표 뒤 section 시작 | p368 부근 | p370 | rhwp가 +2쪽 |
| 문서 끝 | p383 | p386 | rhwp가 +3쪽 |

따라서 전역 height 배율이나 페이지 수 강제 보정은 금지한다. section 10 전후의 과도한 페이지 분할, 103행 표의 과도한 압축, 표 뒤 한 페이지 잔차가 서로 상쇄되어 현재 총 +3이 된다.

## 대표 표의 모델 사실

- section 7 para 239: 3 x 7 표, `RowBreak`, 140.1 x 68.9 mm. HWPX/HWP의 의미 모델은 같지만 HWP에는 raw common attr와 paragraph attr2 tail이 추가로 존재한다.
- section 10 para 4: 1 x 1 표, 73 문단, `RowBreak`, 140.1 x 168.4 mm. HWPX table attr는 `0x04000006`, HWP는 `0x00000006`이다.
- section 11 para 3: 103 x 2 표, `RowBreak`, 215.9 x 145.7 mm. HWPX table attr는 `0x04000006`, HWP는 `0x00000006`이다.

section 7에서도 HWP/HWPX 페이지 차이가 있으므로 table attr bit 26을 전체 원인으로 단정하지 않는다.

## HWP 직접 입력의 deferred fragment 추적

- 직접 HWP의 `DECL_DEFER` 경로는 section 7 para 239, section 10 para 4 및 para 14에서 확인됐다.
- 저장 anchor guard를 임시 적용하면 section 10 para 4와 para 14는 새 페이지 배치 대신 HWPX와 같은 현 페이지 fragment scan으로 들어간다. 각각 남은 본문 높이는 `616.4px`, `174.1px`였다.
- 그러나 총 페이지 수는 HWP `393`, HWPX `386`으로 변하지 않았다. 따라서 이 두 local defer는 조판 차이의 증거이지만 독립적으로 제거할 수 있는 페이지 수 초과 원인은 아니다.
- 위 guard와 `RHWP_DIAG_SCAN` 임시 계측은 분석 후 제거했으며 구현 커밋에 포함하지 않는다.

## 실행 가능한 383쪽 기준선

- 분리 worktree의 `07555d200`을 독립 target directory에 빌드한 뒤, **현재 checkout의 HWP/HWPX 절대 경로**를 직접 입력으로 실행했다.
- 과거 binary는 현재 HWP에서 `383페이지`, 현재 HWPX에서도 `383페이지`를 출력했다.
- 두 입력의 SHA-256은 historical worktree 안의 동명 fixture와 각각 일치했다. 즉 fixture 변경이나 기준 PDF 교체가 아니라, `07555d200` 이후 renderer pagination의 회귀다.
- 과거 기준과 현재 구현 사이에는 대규모 pagination 추출과 리팩터가 포함되어 있어 일괄 되돌리기는 금지한다.

## 다음 구현 게이트

1. 현재와 `07555d200`의 section별 페이지 수 및 103행 표의 row cut을 비교해 공통 동작 차이를 확정한다.
2. renderer/table pagination의 한 책임으로 원인을 좁히고, 축소 fixture와 회귀 테스트를 먼저 만든다.
3. 수정 후 원본 HWPX와 원본 HWP가 각각 383쪽인지 측정한다. 한 입력만 맞추는 source-specific 우회는 수용하지 않는다.
4. 구현 결과에는 page count, bit 28 raw record, Hancom raster metric을 함께 남긴다.
