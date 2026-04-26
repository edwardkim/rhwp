# 수행계획서 — Task #356 페이지 분기 오버플로 (vpos 권위값/spacing 누적 오차)

- **이슈**: [#356](https://github.com/edwardkim/rhwp/issues/356) 페이지 분기 오버플로 — vpos 권위값/spacing 누적 오차로 본문이 페이지 박스 초과
- **마일스톤**: M05x (v0.5.x)
- **브랜치**: `local/task356` (base: `devel`)
- **샘플**: `samples/2022년 국립국어원 업무계획.hwp` (PDF 37쪽 vs 현재 SVG 35쪽)

## 1. 배경 및 증상

- 페이지 3 에서 `body_area h=933.5px`, `used=913.9px` 로 px 잔여 공간 19.6px 남는 것으로 평가하여 pi=40~42 를 같은 페이지에 추가 배치.
- 그러나 HWP 권위값(LINE_SEG `vpos`) 기준으로는 pi=39 누적 vpos = 68,681 HU, body_area ≈ 70,012 HU 로 잔여 1,331 HU. pi=40 의 spacing_before=1000 HU + 첫 줄 1600 HU 만으로 한계 초과.
- HWP 자체는 pi=40 의 첫 LINE_SEG `vpos` 를 0(또는 작은 값)으로 리셋하여 새 페이지 시작 신호를 남기지만, 현재 페이지네이터는 px 누적(`st.current_height`) 기반 평가만 사용해 이 신호를 잡지 못한다.
- 결과로 본문이 페이지 박스를 넘어 푸터 `- 1 -` 가 SVG 영역 밖으로 밀려나고, 누적 오차가 다른 페이지로도 전파되어 전체 페이지 수가 불일치(SVG 35 vs PDF 37).

## 2. 문제의 본질

`paginate_text_lines()` (src/renderer/pagination/engine.rs:589) 가 페이지 분기를 결정할 때:

1. **인접 문단 간 vpos 리셋 신호 미사용** — pi=40 의 첫 ls.vpos 가 직전 문단 vpos_end 보다 크게 작은 경우(=HWP 가 새 페이지로 보낸 경우)를 강제 break 로 처리하지 않는다. 현재 `--respect-vpos-reset` 은 *문단 내부* LINE_SEG 리셋만 본다 (engine.rs:595–604).
2. **px 누적 vs vpos 권위값 불일치** — `st.current_height += para_height` 누적이 HWP 가 기록한 ls.vpos 와 점진적으로 어긋난다. engine.rs:257–282 의 vpos 보정은 fallback 으로만 동작.

`compute_hwp_used_height()` (document_core/queries/rendering.rs:1618–1672) 는 이미 권위값 기반 계산을 갖고 있으나 페이지 분기 결정에는 쓰이지 않는다.

## 3. 목표

1. 인접 문단 간 vpos 리셋을 **강제 페이지 분기**로 인식해 본 샘플 페이지 3 의 오버플로를 제거한다.
2. 부수적으로 SVG 페이지 수가 PDF 와 일치(37쪽)하는지 확인한다.
3. 다른 샘플(특히 form-002, exam_eng 등 골든) 회귀가 없도록 보장한다.

## 4. 범위

### 포함
- `paginate_text_lines()` 의 인접 문단 vpos 리셋 감지 로직 추가
- 동작 게이팅: 기본은 옵트인(보수적) 또는 자동 활성화(적극) 중 단계 1 에서 결정
- dump-pages / SVG export 결과로 회귀 검증

### 제외
- HWP 가 vpos 를 기록하지 않는 경로(빈 문단, 메모, 각주 본문 등)에서의 추정 분기 — 별도 이슈
- 다단/머리말꼬리말 전체 재설계
- 표 분할 정책 변경 (#356 본문 외 영향 최소화)

## 5. 검증 방법

| 단계 | 검증 |
|------|------|
| 재현 | `rhwp dump-pages "samples/2022년 국립국어원 업무계획.hwp" -p 2` 로 페이지 3 의 pi=20..42 배치 확인 |
| 단위 검증 | 동일 명령으로 fix 후 pi=39 까지만 페이지 3 에 포함, pi=40 부터 페이지 4 로 이동 확인 |
| 종합 검증 | `rhwp export-svg` 후 페이지 수 = 37 (또는 PDF 와 일치) |
| 회귀 | `cargo test` 전체 + form-002, exam_eng 골든 SVG diff 확인 |

## 6. 위험과 대응

| 위험 | 대응 |
|------|------|
| HWP 의 vpos 리셋이 의도된 페이지 break 가 아닌 경우(예: 단 변경, 표 내부) 오탐 | 인접 문단 간 비교 시 같은 단(column)·같은 섹션·표 외부 등 제한 조건 적용 |
| 골든 SVG 다수 변경으로 인한 노이즈 | 단계 2 에서 변경 범위를 측정 후 작업지시자 승인 받음 |
| `--respect-vpos-reset` 와 동작 중복 | 신규 로직은 *인접 문단* 만 다루고, 기존 *문단 내부* 로직과 명확히 분리 |

## 7. 일정

총 4~5 단계 (구현 계획서에서 세분화). 단계별 승인 후 진행.

## 8. 산출물

- `mydocs/plans/task_m05x_356_impl.md` — 구현 계획서
- `mydocs/working/task_m05x_356_stage{N}.md` — 단계별 보고서
- `mydocs/report/task_m05x_356_report.md` — 최종 보고서
- 코드: `src/renderer/pagination/engine.rs` 외
