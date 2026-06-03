# Task #537 Stage 1 완료 보고서

**제목**: Baseline 캡처 + TDD 단위테스트 추가
**브랜치**: `local/task537`
**이슈**: https://github.com/edwardkim/rhwp/issues/537

---

## 1. 수행 내용

### 1.1 Baseline SVG 캡처
`samples/21_언어_기출_편집가능본.hwp` 전체 15페이지를 `output/svg/task537_baseline/` 에 보존 (gitignore, 로컬 보관).

### 1.2 RHWP_VPOS_DEBUG 출력 분석
`base=716` 발생 paragraph 10건 (page 2 의 pi=39~42, page 3 의 pi=74~79 등):
```
VPOS_CORR: path=lazy pi=39 prev_pi=38 prev_lh=1100 prev_ls=716 base=716 ...
VPOS_CORR: path=lazy pi=40 prev_pi=39 prev_lh=1100 prev_ls=716 base=716 ...
VPOS_CORR: path=lazy pi=41 prev_pi=40 prev_lh=1100 prev_ls=716 base=716 ...
VPOS_CORR: path=lazy pi=42 prev_pi=41 prev_lh=1100 prev_ls=716 base=716 ...
VPOS_CORR: path=lazy pi=74 prev_pi=73 prev_lh=1100 prev_ls=716 base=716 ...
... (9p,12p,17p 등의 답안 ②~⑤)
```
모두 동일 패턴: 직전 paragraph 의 `prev_ls = 716` 그대로 base 에 박힘.

다른 base 값(1560, 9014, 992 등)은 trailing-ls 외 원인(spacing_after, Shape 높이, vpos reset 등) 으로 본 task 수정 범위 밖.

### 1.3 TDD 단위테스트 추가

`src/renderer/layout/integration_tests.rs` 에 `test_537_first_answer_after_tac_table_line_spacing` 추가.

검증 내용:
- 페이지 2 SVG 에서 ① ② ③ 의 baseline y 추출
- gap(①→②) ≈ gap(②→③) (두 gap 이 같은 ParaShape 동일 라인수 → 일치 필수)
- gap(①→②) ≈ IR vpos delta (5448 HU = 72.64 px)

### 1.4 테스트 실행 결과 (수정 전, Stage 1 종료 시점)

```
test result: FAILED. 1116 passed; 1 failed; 1 ignored
└── test_537_first_answer_after_tac_table_line_spacing FAILED
    ①→② gap(63.09) 와 ②→③ gap(72.64) 가 일치해야 함.
    y1=765.11, y2=828.20, y3=900.84.
```

기존 1116 테스트 전부 통과 — 신규 테스트만 의도대로 실패.
TDD 사이클: Red ✅ (Stage 2 에서 Green 으로 전환).

## 2. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | `test_537_first_answer_after_tac_table_line_spacing` 추가 (+76 LOC) |
| `mydocs/working/task_m100_537_stage1.md` | 본 보고서 |

## 3. 다음 단계 (Stage 2)

`src/renderer/layout.rs:1497-1509` 의 lazy_base 산출 로직에 trailing_ls_hu 보정 적용 (구현계획서 §2.5 A'안).
- 신규 테스트가 통과되어야 함
- 한컴 PDF 200dpi 와 시각 비교
- Stage 1 baseline 과 픽셀 단위 비교 (수정 전후)

## 4. 승인 요청

Stage 1 완료. Stage 2 진행 승인 요청.
