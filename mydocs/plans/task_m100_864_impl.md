# Task #864 구현계획서

**선행**: `mydocs/plans/task_m100_864.md` (수행계획서, 승인 완료)
**브랜치**: `local/task864`
**작성일**: 2026-05-13

## 단계 분해 (3 단계 + 승인 게이트)

### Stage A: 본질 정밀 진단

**A.1 WMF binary 의 모든 record 추출 + dump**:

- HWP3 sample14 의 bin_id=2 binary 가 WMF format
- rhwp 의 WMF parser 호출 시 debug print 추가 — 모든 record (SetMapMode, SetViewportOrg, SetViewportExt, SetWindowOrg, SetWindowExt, StretchDIBits, ExtTextOut, Polygon, Rectangle 등) 와 좌표 값 출력
- record 순서 + 좌표 변화 추적

```bash
RHWP_DEBUG_WMF=1 ./target/release/rhwp export-svg "samples/hwp3-sample14.hwp" -p 1 -o /tmp/864a/ 2>&1 | grep "WMF_" > /tmp/864a_records.log
wc -l /tmp/864a_records.log
```

**A.2 WMF spec 검증**:

Microsoft WMF spec 의 좌표 변환 규칙:
- Logical → Device 변환: `device = (logical - window_org) * (viewport_ext / window_ext) + viewport_org`
- SetMapMode 의 각 mode 별 viewport_ext / window_ext 처리
- MM_ANISOTROPIC: SetWindow* + SetViewport* 가 명시적 정의
- MM_ISOTROPIC: aspect ratio 유지

WMF spec 참고: `mydocs/tech/` 또는 외부 reference.

**A.3 한컴 PDF actual element 좌표 추출**:

- 한컴 PDF page 2 의 image stream + text bbox 좌표 (pdftotext / pdfimages)
- rhwp SVG 의 element y 좌표 와 정량 비교

**A.4 가설 H1~H4 결판**:

각 가설 ✓/✗ → 본질 코드 위치 식별.

**산출**: `mydocs/working/task_m100_864_stage_a.md`
- WMF record dump (전체 record 순서 + 좌표 값)
- 한컴 PDF vs rhwp 좌표 비교
- 가설 결판 + 본질 위치

### Stage B: 정정 후보 평가

**B.1 정정 후보 식별**:

- 후보 1: SetViewportOrg / SetViewportExt 처리 추가 (`src/wmf/converter/svg/mod.rs` 또는 `device_context.rs`)
- 후보 2: SetMapMode 처리 (specific map mode 별)
- 후보 3: 좌표 변환 공식 적용 (logical → device)
- 후보 4: y-flip transform 적용 (특정 조건)

**B.2 회귀 위험 평가**:

- exam_eng, exam_math 등 다른 sample 의 EMF/WMF 사용 여부
- 회귀 위험 평가

**산출**: `mydocs/working/task_m100_864_stage_b.md`

### Stage C: 정정 구현 + 검증

**C.1 정정 구현**:

진단 결과에 따라 1~3 위치 소스 수정. 작은 변경 우선.

**C.2 검증**:
1. cargo build --release
2. cargo test --release --lib (1230 passed)
3. hwp3-sample14 page 2 PNG → 한컴 PDF 정합 (박스 위, 캡션 아래)
4. hwp3-sample14-hwp5.hwp / .hwpx 동일 검증
5. 다른 EMF/WMF 보유 sample 회귀 검사 (현재 sample 들 중 EMF/WMF 사용 sample 식별 후)

**C.3 종합 보고서**:

**산출**: `mydocs/working/task_m100_864.md` (종합)
**커밋**: `Task #864: WMF metafile element y 좌표 처리 정정 — {본질 위치}`

## 작업 순서 + 승인 게이트

```
A 본질 정밀 진단 → 산출 → 단계 완료 → 승인
                                        ↓
B 정정 후보 평가 → 산출 → 단계 완료 → 승인
                                        ↓
C 정정 구현 + 검증 + 종합 → 커밋 → 단계 완료 → 승인
```

각 단계 후 승인 게이트.

## 위험 + 회피

| 위험 | 회피 |
|---|---|
| WMF spec 복잡성 (MapMode + Viewport + Window 조합) | A 의 정밀 binary 분석 + spec 정합 |
| 다른 WMF sample 회귀 | 회귀 검증 필수 (먼저 sample 식별) |
| CLAUDE.md HWP3 규칙 | EMF/WMF format 자체의 fix → 모든 포맷 정합 |

## 본 단계 범위 외

- 다른 HWP3 결함
- WASM browser 검증 (선택)

## 승인 요청

본 구현계획서 승인 후 → Stage A 부터 진행.

📋 **Task #864 구현계획서 승인 요청드립니다.**
