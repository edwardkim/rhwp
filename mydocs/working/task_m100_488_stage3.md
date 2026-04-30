# Task M100 #488 — Stage 3 완료 보고서

## 작업 내용

Stage 3에서는 토크나이저 + 렌더러 변경의 회귀 영향을 광범위 샘플에서 검증하고, 시각 검증 중 발견된 4건의 별개 영역 문제를 별도 이슈로 분리.

## 회귀 검증 결과

### 핵심 샘플 8개 (전체 페이지 SVG export)

| 샘플 | 페이지 수 | raw `rm`/`it`/`bold` prefix |
|------|----------|----------------------------|
| `samples/eq-01.hwp` | 1 | 0건 |
| `samples/equation-lim.hwp` | 1 | 0건 |
| `samples/atop-equation-01.hwp` | 1 | 0건 |
| `samples/exam_math.hwp` | 20 | 0건 |
| `samples/exam_kor.hwp` | 20 | 0건 |
| `samples/exam_eng.hwp` | 8 | 0건 |
| `samples/exam_social.hwp` | 4 | 0건 |
| `samples/exam_science.hwp` | 4 | 0건 (수정 전 60건) |
| **합계** | **59 페이지** | **0건** |

### 수식 출력 spot check

- `eq-01.hwp`: 평점, 입찰가격평가, 배점한도, ×, 최저입찰가격, 해당입찰가격 — 모두 정상
- `equation-lim.hwp`: lim, → — 정상
- `exam_math.hwp` 페이지 1: italic 텍스트 29건 (수학 변수 default italic 정상)
- `exam_science.hwp` 페이지 1: K⁺, X⁻, Y⁻, Ca²⁺, O²⁻, mol, KOH, H₂O, CH, C 등 화학 기호 정상 (이전엔 `rmK`, `rmCa`, `rmmol` 등 raw)

### 단위 테스트

- 토크나이저 모듈: 20 passed (기존 12 + Stage 1 신규 8)
- SVG 렌더러 모듈: Stage 2 신규 6 passed
- 수식 모듈 전체: 66 passed
- **라이브러리 전체**: **1092 passed, 0 failed, 1 ignored**

## 별도 이슈 등록

Stage 2 시각 검증 중 발견되었으나 본 task 범위(폰트 스타일 처리) 밖인 4건을 별도 이슈로 분리:

| Issue | 제목 | 영역 |
|-------|------|------|
| [#489](https://github.com/edwardkim/rhwp/issues/489) | exam_science.hwp 페이지 1 5번 문제 그림이 본문 첫 줄을 가림 | 그림 wrap / 텍스트 둘러쌈 |
| [#490](https://github.com/edwardkim/rhwp/issues/490) | 페이지 1 3번 표 28/36 셀 중앙정렬 | 표 셀 horizontal alignment |
| [#491](https://github.com/edwardkim/rhwp/issues/491) | 페이지 1 2번 답안지 위치 약간 아래로 | 표 높이 계산 / 문단 spacing |
| [#492](https://github.com/edwardkim/rhwp/issues/492) | 페이지 1 컬럼 2 5번 문제 밑단 짤림 | 페이지네이션 / 컬럼 used height |

## 회귀 검증 한계

devel HEAD vs local/task488 의 SVG diff 를 모든 샘플에 대해 수행하지 않았다. 대신:
- 변경 영역이 명확히 한정 (수식 토크나이저 1개 함수 + svg/canvas 렌더러 Text arm)
- 단위 테스트 1092건 통과 (회귀 없음)
- raw prefix 0건 검증 (8개 샘플 59 페이지)

본질 정정에 가까운 변경이지만, hwpeq 룰(키워드 prefix 분리, italic 파라미터 honor)에 부합하는 수정이며 회귀 위험은 단위 테스트와 샘플 spot check 로 검증된 것으로 판단.

## 최종 보고

`mydocs/report/task_m100_488_report.md` 작성 → 승인 후 `local/task488` → `local/devel` merge.
