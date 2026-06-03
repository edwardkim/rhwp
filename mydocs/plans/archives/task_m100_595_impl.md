# Task #595 구현 계획서

**Issue**: #595 — exam_math.hwp 2페이지부터 수식 더블클릭 hitTest 오동작
**브랜치**: `local/task595`
**상위 계획서**: `task_m100_595.md`

---

## 단계 분해 (3 단계)

### Stage 1 — e2e 진단 테스트 작성 + 실행 + 본질 가설 정립

**산출물**: `rhwp-studio/e2e/issue-595.test.mjs`

**시나리오 (단일 테스트)**:

1. `samples/exam_math.hwp` 로드 (helpers `loadHwpFile`)
2. **page 1 (0-based 0) 의 수식 클릭**:
   - 대상: paraIdx=18 ci=0 ("2.함수 ..." 본문 수식, x=130.8 y=818.3 w=108.0 h=17.5)
   - 클릭 좌표: 페이지 내부 (130.8 + 50, 818.3 + 8) ≈ (180.8, 826.3) 페이지 px
   - DOM 좌표 변환: contentX = pageLeft + 180.8 * zoom, contentY = pageOffsets[0] + 826.3 * zoom
3. **page 2 (0-based 1) 의 수식 클릭**:
   - 대상: paraIdx=65 ci=0 (이슈 명세 수식, x=589.5 y=191.7 w=131.7 h=37.3)
   - 클릭 좌표: 페이지 내부 (589.5 + 65, 191.7 + 18) ≈ (654.5, 209.7) 페이지 px
   - DOM 좌표 변환: contentX = pageLeft + 654.5 * zoom, contentY = pageOffsets[1] + 209.7 * zoom
4. **각 클릭 직전/직후 진단 데이터 캡처** (page.evaluate):
   - `pageIdx`, `pageX`, `pageY`, `pageOffset`, `cx`, `cy`, `zoom`, `pageHeights[]`, `pageOffsets[]`
   - `wasm.getPageControlLayout(pageIdx)` controls 배열의 type=equation 항목 list
   - `inputHandler.findPictureAtClick(pageIdx, pageX, pageY)` 직접 호출 결과
   - 클릭 후 `cursor.isInPictureObjectSelection()` 상태
5. **결과 비교 + 본질 가설 정립**

**기존 코드 변경**: 없음 (e2e 신규 1 파일만 추가)

**진단 hook 노출 필요 시**: stage 종료 후 revert

**완료 기준**:
- e2e 테스트 실행 성공 (host 또는 headless 모드)
- 진단 데이터 캡처 완료
- 본질 가설 1개 이상 데이터로 뒷받침
- `mydocs/working/task_m100_595_stage1.md` 작성

### Stage 2 — (조건부) 본질 정정 또는 추가 진단

**진입 조건**: Stage 1 본질 가설이 명확 + 정정 영역 식별

**케이스 분기**:
- **케이스 A**: 본질 명확 + 영향도 낮음 → 본질 정정 + 회귀 테스트
- **케이스 B**: 본질 추정만 가능 → 추가 진단 hook 또는 다른 fixture 비교
- **케이스 C**: Rust 측 결함 발견 → 별도 task 분리 검토

**완료 기준**:
- 케이스별 산출물 + 검증 통과 + Stage2 보고서

### Stage 3 — 회귀 테스트 + 최종 보고서

**산출물**:
- `mydocs/report/task_m100_595_report.md`
- 회귀 sweep 결과 (cargo test, clippy, build, e2e 회귀 0)
- `mydocs/orders/20260506.md` 갱신

---

## 위험도 영역

| 영역 | 위험도 | 대응 |
|------|--------|------|
| e2e host/headless 환경 | 낮음 | 둘 다 시도 (helpers 지원) |
| inputHandler.findPictureAtClick private | 낮음 | JS 런타임 접근 가능 (TS private은 컴파일 시만) |
| 진단 hook 노출 시 prod 영향 | 매우 낮음 | `import.meta.env.DEV` 가드 (현재 `__inputHandler` 와 동일 영역) |
| 본질 정정 회귀 | 중 | Stage 2 진입 시 영역 식별 후 회귀 sweep 사전 정의 |

## 5. 작업지시자 승인 요청

- 본 구현 계획서 (3단계) 승인
- Stage 1 진행 가능 여부

승인 후 Stage 1 e2e 테스트 작성 시작.
