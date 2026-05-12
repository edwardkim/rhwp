# Task #860 구현 계획서

**선행**: `mydocs/plans/task_m100_860.md` (수행계획서, 승인 완료)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 단계 분해 (3 단계 + 승인 게이트)

### Stage A: 본질 정밀 진단

**A.1 한컴 PDF vs rhwp page 2 이미지 비교**:

```bash
mkdir -p output/860
# 한컴 PDF page 2 의 image stream 추출
pdfimages -all "pdf/hwp3-sample14-hwp5-2022.pdf" output/860/pdf_img 2>&1 | tail -3
ls -la output/860/pdf_img*

# rhwp SVG 의 image element 추출
./target/release/rhwp export-svg "samples/hwp3-sample14.hwp" -p 1 -o /tmp/860/
grep -o "<image[^>]*" /tmp/860/hwp3-sample14_002.svg | head -3
```

**A.2 HWP3 의 BinData bin_id=2 raw byte 추출**:

```bash
# HWP3 파서가 BinData 어떻게 처리하는지 확인
grep -rn "BinData\|bin_data\|fn parse_bin_data" src/parser/hwp3/ | head -10

# HWP3 binary 의 bin_id=2 byte 추출 (debug print 추가 또는 cargo example)
```

format 식별:
- PNG magic: `89 50 4E 47` (b"\x89PNG")
- JPEG magic: `FF D8 FF`
- EMF magic: `01 00 00 00` (header)
- WMF magic: `D7 CD C6 9A`
- Custom HWP format ?

**A.3 rhwp SVG 의 image element 검사**:

- `<image href="data:image/png;base64,..."/>` 형식?
- 빈 placeholder 또는 src 없음?
- crop coords 정상?

**A.4 가설 H1~H5 결판**:

각 가설 ✓/✗ → 본질 위치:
- 본질 코드 line (HWP3 parser 또는 image rendering)
- 정정 방향 명확화

**산출**: `mydocs/working/task_m100_860_stage_a.md`
- BinData 추출 결과 (format / size / first bytes)
- SVG image element 분석
- 가설 결판
- 정정 후보 ≥ 1 개

### Stage B: 정정 후보 평가

**B.1 정정 후보 회귀 위험 평가**:

다른 sample 검사:
- `samples/hwp3-sample13.hwp` (다른 HWP3 sample)
- 기존 HWP3 image 보유 sample 들

**B.2 정정 후보 우선순위**:

CLAUDE.md 규칙 (HWP3 전용 로직 → `src/parser/hwp3/` 내부) 정합 확인.

**산출**: `mydocs/working/task_m100_860_stage_b.md`
- 정정 후보 + 회귀 위험
- 정정 순서

### Stage C: 정정 구현 + 검증

**C.1 정정 구현**:

진단 결과에 따라 1~3 위치 소스 수정. 작은 변경 우선.

수정 위치 후보 (A 진단 후 확정):
- `src/parser/hwp3/` (image binary 추출 / format 변환)
- 또는 image format conversion (예: EMF → PNG via emf crate)

**C.2 검증**:
1. `cargo build --release`
2. `cargo test --release --lib --bins` (1246 passed 기대)
3. page 2 PNG 시각 비교 한컴 PDF 정합 — 박스 내부 텍스트 표시
4. 다른 HWP3 sample 회귀:
   - `samples/hwp3-sample13.hwp`
   - HWP3 image 보유 다른 sample
5. HWP5 변환본 (hwp3-sample14-hwp5.hwp) 도 동일 결과 확인 — 같은 image bin_id 사용 시 같은 결함이므로 fix 후 동일 정합

**C.3 종합 보고서**:

**산출**: `mydocs/working/task_m100_860.md` (종합)
**커밋**: `Task #860: HWP3 sample14 그림 내부 콘텐츠 렌더 정정 — {본질 위치}`

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
| HWP3 image format 의 복잡성 | format 별 단계적 fix |
| EMF/WMF decoder 의존성 추가 필요 | crate (emf, wmf 등) 사용 또는 fallback PNG 변환 |
| 다른 sample 회귀 | hwp3-sample13 + 기존 sample 회귀 검증 |
| CLAUDE.md HWP3 전용 로직 제약 | 공통 모듈에 HWP3 분기 추가 금지, parser 내부만 |

## 본 단계 범위 외

- HWP3 의 다른 결함 (별도 issue)
- WASM browser 검증 (선택)

## 승인 요청

본 구현 계획서 승인 후 → Stage A 부터 시작.

📋 **Task #860 구현 계획서 승인 요청드립니다.**
