# Task #860 Stage D 구현 계획서

**선행**: `mydocs/plans/task_m100_860_v2.md` (Stage D 수행계획서, 승인 완료)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 단계 분해 (3 단계 + 승인 게이트)

### Stage D-A: 본질 정밀 진단

**D-A.1 rhwp page 1/page 2 boundary 의 paragraph 식별**:

```bash
./target/release/rhwp dump-pages "samples/hwp3-sample14.hwp" -p 0 -p 1 2>&1 | \
  grep -E "=== 페이지|FullParagraph|Table" | head -30
```

확인:
- page 1 마지막 paragraph
- page 2 첫 paragraph
- 그림 paragraph 와 캡션 paragraph 의 page 위치

**D-A.2 한컴 PDF page 1/page 2 의 paragraph 분포**:

```bash
for p in 1 2; do
  echo "=== HANCOM PAGE $p ==="
  pdftotext -layout -f $p -l $p "pdf/hwp3-sample14-hwp5-2022.pdf" -
done
```

매핑: 어떤 paragraph 가 page 1 vs page 2 에 있는지 한컴 정합 식별.

**D-A.3 그림 paragraph (pi=N) 의 controls 및 flag dump**:

```bash
# 그림 paragraph 의 controls (image, page_break_before, column_type)
./target/release/rhwp dump "samples/hwp3-sample14.hwp" -s 0 -p N
# N = page 1 끝의 그림 paragraph index
```

확인:
- 그림 control 의 attr (TAC, wrap mode)
- paragraph 의 column_type (Column / Page / MultiColumn / None)
- page_break_before / spacing_before / spacing_after

**D-A.4 캡션 paragraph 의 식별 + position 결정 로직 추적**:

캡션 ("'P' 명령 이후 d 앞에 ple 가 붙여짐") 가 별도 paragraph 인지 그림 control 의 sub-element 인지 확인.

HWP3 의 caption mechanism — HWP3 spec 참고:
- 그림 control 안에 caption 정보 (별도 paragraph 가 아닌 control attribute)
- 또는 별도 paragraph (paragraph index 가 그림 paragraph 의 직후)

**D-A.5 가설 H1~H4 결판**:

각 가설 ✓/✗ → 본질 위치:
- HWP3 파서 의 caption 추출 위치
- typeset 의 그림 + 캡션 paragraph packing 정책
- layout 의 page boundary 결정

**산출**: `mydocs/working/task_m100_860_stage_d_a.md`
- rhwp vs 한컴 paragraph 분포 매핑 표
- 본질 위치 (소스 코드 line)
- 정정 후보 ≥ 1 개

### Stage D-B: 정정 후보 평가

**D-B.1 정정 후보 회귀 위험 평가**:

각 정정 후보를 다른 sample 에 적용 시 영향:
- hwp3-sample13.hwp (다른 HWP3)
- HWP5/HWPX 변환본
- exam_eng.hwp (HWP5)

**D-B.2 우선순위 결정**:

CLAUDE.md 규칙 (HWP3 전용 로직 → `src/parser/hwp3/` 내부) 정합 확인.

**산출**: `mydocs/working/task_m100_860_stage_d_b.md`

### Stage D-C: 정정 구현 + 검증

**D-C.1 정정 구현**:

진단 결과에 따라 1~3 위치 소스 수정. 작은 변경 우선.

수정 위치 후보 (A 진단 후 확정):
- `src/parser/hwp3/` (HWP3 의 그림/캡션 추출)
- `src/renderer/typeset.rs` (paragraph packing 결정)
- `src/renderer/layout.rs` (그림 + 캡션 page boundary)

**D-C.2 검증**:
1. `cargo build --release`
2. `cargo test --release --lib --bins` (1230 passed 기대)
3. page 2 PNG 시각 비교 한컴 PDF 정합 — 박스 → 캡션 순서
4. dump-pages page 2 paragraph 분포 한컴 정합
5. 다른 sample 회귀:
   - hwp3-sample13.hwp 페이지 수
   - hwp3-sample14-hwp5.hwp / .hwpx 동일 결합 효과
   - exam_eng.hwp 페이지 수
6. WASM build (선택)

**D-C.3 종합 보고서**:

**산출**: `mydocs/working/task_m100_860_v2.md` (Stage D 종합)
**커밋**: `Task #860 Stage D: HWP3 그림+캡션 paragraph 순서 정합 — {본질 위치}`

## 작업 순서 + 승인 게이트

```
D-A 본질 정밀 진단 → 산출 → 단계 완료 → 승인
                                        ↓
D-B 정정 후보 평가 → 산출 → 단계 완료 → 승인
                                        ↓
D-C 정정 구현 + 검증 + 종합 → 커밋 → 단계 완료 → 승인
```

각 단계 후 승인 게이트.

## 위험 + 회피

| 위험 | 회피 |
|---|---|
| HWP3 캡션 처리 본질 변경 광범위 회귀 | D-A 정확 본질 식별 + 보수적 정정 + 4 sample 회귀 검증 |
| Stage A-C BMP fix 와 충돌 | 통합 검증 (BMP 콘텐츠 + paragraph 순서) |
| 캡션 paragraph 가 control 내부 표현 시 광범위 변경 | HWP3 파서 내부에서 처리 (CLAUDE.md 정합) |

## 본 단계 범위 외

- 다른 HWP3 결함
- HWPX 파서 결함
- WASM browser 검증

## 승인 요청

본 구현 계획서 승인 후 → Stage D-A 부터 시작.

📋 **Task #860 Stage D 구현 계획서 승인 요청드립니다.**
