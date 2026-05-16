# Task #930 최종 결과보고서

**Issue**: #930 — 글상자 내부 텍스트에 도형 행렬변환(matrix) 세로 스케일 미적용
**브랜치**: `local/task930`
**마일스톤**: v1.0.0 (M100)

## 1. 개요

`samples/table-in-tbox.hwp` 2페이지가 PDF(한글 2022) 정답지와 크게 달랐다. 글상자(사각형 도형) 안의 본문 텍스트가 극히 작게 렌더되어 거의 보이지 않았고 "검사항목" 표 글자가 흩어졌다.

원인 조사 중 작업지시자 피드백으로 `shortcut.hwp` 자동번호 "1" 과소 렌더가 추가 확인되어, 작업지시자 결정으로 #930 범위를 확장하여 두 결함을 함께 수정했다.

## 2. 결함 분석

`src/renderer/layout/shape_layout.rs` `layout_textbox_content()` 의 #874(commit `a466e2ea`) 글꼴 축소 휴리스틱이 원인이었다.

- **결함 A** (table-in-tbox 2p): 글상자가 원본 대비 확대되면 글꼴을 `1/max_ratio`로 축소하는 로직이, 한 축만 강하게 늘어난 이방 확대 글상자(sx≈1.07, sy≈8.2)에도 발동했다. 이미 current 박스 기준으로 조판이 끝난 본문 글꼴이 1/8.2로 깨졌다.
- **결함 B** (shortcut 1p): 등방 확대 글상자(자동번호)에는 축소가 발동되는 것이 맞으나, 축소 계수 `1/max_ratio`가 한컴 2022 PDF 대비 약 2배 과축소였다.

## 3. 수정 내용

`src/renderer/layout/shape_layout.rs` `layout_textbox_content()`:

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| 발동 조건 | `max_ratio > 1.5` | `min_ratio > 1.5` (등방 확대만) |
| 축소 계수 | `(1.0 / max_ratio).min(1.0)` | `(2.0 / max_ratio).min(1.0)` |

- 발동 조건을 두 축 모두 1.5배 초과로 좁혀 이방 확대 글상자(결함 A)를 제외했다.
- 축소 계수를 PDF 정합 측정값으로 정정했다(결함 B).
- `[Task #874 #3 / #930]` 주석으로 변경 사유·경험적 보정·재검증 필요성을 명시했다.

## 4. 검증

### 4.1 table-in-tbox.hwp 2페이지 (결함 A)

- 글상자 본문 char `font-size`: `2.44`/`2.28` → `22.67`/`20`/`18.67` 등 정상 본문 크기
- 본문 텍스트·"검사항목" 표 모두 PDF(한글 2022) 2페이지와 시각 정합

### 4.2 shortcut.hwp 1페이지 (결함 B)

자동번호 "1" 글리프 높이(96dpi flood-fill 측정):

| | 높이 | 폭 |
|---|------|-----|
| 변경 전 | 93px | 45px |
| 변경 후 | **187px** | 90px |
| PDF (한글 2022) | **187px** | 95px |

높이 PDF 정확 일치.

### 4.3 회귀 테스트

`cargo test --release --lib`: **1258 passed; 0 failed** (회귀 0).

## 5. 잔존 사항 / 후속

- `shortcut.hwp` 자동번호 "1" 세로 위치가 PDF 대비 약 17px 아래(우리 y557–743 vs PDF y540–726). 글꼴 *크기*는 정합하며 위치 오프셋은 #930(글상자 matrix 글꼴 스케일) 범위 밖이다. 미세 오차로 별도 이슈 등록 보류 — 작업지시자 판단에 위임.
- 축소 계수 `2.0/max_ratio`는 등방 확대 글상자 단일 샘플(`shortcut.hwp` 자동번호) 기반 경험적 보정이다. 다른 등방 확대 글상자 샘플 확보 시 재검증 필요(코드 주석 명시).

## 6. 단계별 산출물

| 단계 | 산출물 |
|------|--------|
| 수행계획서 | `mydocs/plans/task_m100_930.md` |
| 구현계획서 | `mydocs/plans/task_m100_930_impl.md` |
| Stage 1 | `mydocs/working/task_m100_930_stage1.md` — 판별자 측정·확정 |
| Stage 2 | `mydocs/working/task_m100_930_stage2.md` — 축소 계수 재유도 |
| Stage 3 | `mydocs/working/task_m100_930_stage3.md` — 구현 |
| 최종보고서 | `mydocs/report/task_m100_930_report.md` |

## 7. 결론

#930 의 두 결함(이방 확대 글상자 오발동, 등방 확대 글상자 과축소)을 모두 해소했다. `table-in-tbox.hwp` 2페이지와 `shortcut.hwp` 1페이지가 한컴 2022 PDF와 시각 정합하며, 전체 회귀 테스트를 통과했다.
