# Task #463 최종 결과보고서

## 제목

exam_kor 14p: 본문 외곽선이 셀 단락 leakage 로 4개 박스로 쪼개지는 현상 수정

## 결과

✅ **완료**. PDF 와 동일하게 (가) 학생 대화 전체가 단일 외곽선 박스 1개로 렌더링됨. 1069 단위테스트 통과, 회귀 없음.

## 변경 요약

### 코드 변경 (1파일, 7줄 추가)

`src/renderer/layout/paragraph_layout.rs:2516`:

```rust
// 변경 전
if para_border_fill_id > 0 {

// 변경 후
// Task #463: 셀 안 단락은 본문 큐에 leakage 하지 않도록 cell_ctx 게이팅.
if para_border_fill_id > 0 && cell_ctx.is_none() {
```

### 문서 산출물

- `mydocs/plans/task_m100_463.md` — 수행계획서
- `mydocs/plans/task_m100_463_impl.md` — 구현계획서
- `mydocs/working/task_m100_463_stage2.md` — Stage 2 단계별 보고서
- `mydocs/working/task_m100_463_stage3.md` — Stage 3 단계별 보고서
- `mydocs/report/task_m100_463_report.md` — 본 최종 보고서

## 원인 분석

### 증상

`samples/exam_kor.hwp` 14페이지의 좌측 단 (가) 학생 대화는 PDF 에서 **단일 외곽선 박스 1개**로 둘러싸여 표시되지만, rhwp SVG 출력은 **4개의 분리된 외곽선 박스**로 쪼개져 그려짐.

### 분석 과정

1. **가설 A (파서 오류)** — `border_fill[7]` 의 4면 borders 가 정상 파싱되는지 확인 → 정상 (4면 모두 Solid w=1).
2. **가설 B (BorderFill::attr)** — attr=0x0000 (특별 비트 없음).
3. **가설 C (ParaShape::attr1 게이팅)** — attr1=0x10000080 의 비트 28 은 pyhwp/한컴 스펙 기준 `level` 의 일부이지 외곽선 표시 게이팅 비트가 아님.
4. **가설 D (BorderFill 의 cell-only 의도)** — 다른 페이지의 동일 ParaShape 사용 케이스로 보아 일반 본문에도 적용됨.
5. **가설 E (셀 단락 leakage) — ✅ 정답**: `RHWP_DEBUG_BORDER` 임시 환경변수로 `para_border_ranges` 큐를 출력해 본 결과, 본문 단락 사이사이에 인용 따옴표(｢｣) 3×2 표의 **셀 안 단락들**이 push 되어 있었음. 이로 인해 merge 로직(`layout.rs:1604+`)이 다음과 같이 망가짐:
   - 셀 단락 `bf=4` (시그니처 `None`) 이 끼면 본문 그룹 단절
   - 셀 단락 `bf=6` (시그니처 `(Solid,1,0)` — bf=7 과 동일) 이 끼면 본문 그룹이 셀 좌표(x≈533, w=0)로 흡수되어 다음 본문이 다시 새 그룹 시작

### 근본 수정

`paragraph_layout.rs` 의 push 호출에 `cell_ctx.is_none()` 게이팅을 추가. 셀 단락은 더 이상 본문 큐에 들어가지 않음. 셀 외곽선은 `table_layout`/`border_rendering` 별도 경로에서 이미 처리되므로 시각적 손실 없음.

## 검증

### 좌측 단 외곽선 비교 (page 14)

| 항목 | 변경 전 | 변경 후 | PDF |
|------|---------|---------|-----|
| stroked 본문 rect 개수 | 4 | 1 | 1 |
| 박스 좌표 | y=435/642/850/1202 (분리) | y=263.88 h=1098.77 (단일) | (가) 전체 단일 |

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed; 1 ignored; 0 measured
```

### 회귀 (다른 샘플)

`2010-01-06.hwp`, `2022년 국립국어원 업무계획.hwp`, `biz_plan.hwp`, `21_언어_기출_편집가능본.hwp` 모두 정상 렌더링. exam_kor 전체 20페이지의 본문 외곽선 stroked rect 개수도 0~1 개 범위로 정상.

## 부수 발견

분석 과정에서 별도 이슈 후보로 기록할 가치가 있는 항목:

- **PDF 의 [38~42] 영역에는 외곽선이 없음** — `border_fill[5]` 의 4면이 모두 `line_type=None` 이라서 정상. rhwp 도 strokeless rect 만 emit 하고 있어 수정 불필요.
- **Task #321 v6 의 cross-bf_id stroke 시그니처 매칭 merge 정책**: 현 로직은 시그니처가 같으면 30px 갭 내에서 병합. 본 이슈에서는 bf=6 (sig None) 와 bf=7 (sig Some) 이 다르므로 자연 분리. 다른 샘플에서 잠재 부작용 가능성은 별도 검토 권장.

## 커밋 이력

```
6a6bbf8 Task #463 Stage 2: 셀 단락이 본문 외곽선 큐에 leakage 하지 않도록 게이팅
2a074be Task #463: 수행계획서 + 구현계획서
```

(Stage 3 보고서, 본 최종 보고서, orders 갱신은 본 stage4 커밋에 포함)

## 머지 절차

1. `local/task463` → `local/devel` no-ff merge
2. `local/devel` → `devel` push (별도 시점, 메인테이너 판단)
3. 이슈 #463 close

## 참조

- GitHub Issue: [#463](https://github.com/edwardkim/rhwp/issues/463)
- 관련 이전 이슈: Task #321 v6 (stroke signature merge)
- 관련 코드:
  - `src/renderer/layout/paragraph_layout.rs:2516`
  - `src/renderer/layout.rs:1604-1748`
- 샘플: `samples/exam_kor.hwp`, `samples/exam_kor.pdf`
