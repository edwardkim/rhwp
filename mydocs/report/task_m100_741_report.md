# Task #741 최종 결과 보고서

**Issue**: #741 — HWP3 외부 file path 그림 + 폰트 매핑 + ParaShape tab + 제목차례 자동 장식 (`hwp3-sample10.hwp` 결함)
**브랜치**: `local/task741`
**선행 의존**: PR #732 (Task #724) — 머지 완료
**완료 commits**: a63114e (Stage 1~4) + 86bf0bd (Stage 5) + ccbb0b6 (Stage 6) + d03109e + b77d071 (Stage 7) + 6f09ece (Stage 8)

## 1. 정정 영역 요약

| Stage | 영역 | 상태 | commit |
|-------|------|------|--------|
| 1 | 본질 진단 — Document IR Picture + HWP3 파서 그림 + 폰트 추적 | 완료 | a63114e |
| 2 | image placeholder Document IR 확장 (`ImageAttr.external_path`) | 완료 | a63114e |
| 3 | TAC 그림 paragraph line_spacing 정합 (HWP3 파서) | 완료 | a63114e |
| 4 | HWP5 변환본 paragraph 26 페이지 분할 정합 (vpos-reset 후속 가드) | 완료 | a63114e |
| 5 | HWP3 사적 graphic char (0x0080~0x7FFF) cross-ref 매핑 | 완료 | 86bf0bd |
| 6 | HWP3 ParaShape tabs[40] → Document IR TabDef 변환 + 필드 순서 bug 정정 | 완료 | ccbb0b6 |
| 7 | HWP3 leader → HWP5 fill_type 매핑 + 제목차례 자동 장식 inject + char_shape 위치 정정 | 완료 | d03109e + b77d071 |
| 8 | HWP3 차례 inline page 번호 (ch=1) decode + display | 완료 | 6f09ece |

## 2. 핵심 본질 발견

### 2.1 HWP3 사적 인코딩 영역 (Stage 5)

HWP3 hchar **0x0080~0x7FFF** 범위는 표준 KSSM 조합형 (0x8000+) 외 **한컴 사적 인코딩**. johab decoder 가 '?' 반환 → 가로선/▷/■ 등 visible char 누락. 매핑 자료 부재.

**해결**: HWP3 ↔ HWP5 변환본 paragraph cross-ref 로 매핑 자동 도출. 상위 6값 = 98.5% coverage.

### 2.2 Hwp3TabDef 필드 순서 bug (Stage 6)

기존 records.rs `Hwp3TabDef`: `(position:u16, type:u8, leader:u8)` — 실제 byte stream 어긋남. HWP3_DIAG_TABS 진단으로 default tab pattern (slot N: position=1000×(N+1) hunit) 검증.

**해결**: 실제 byte 순서 `tab_type(u8) → leader(u8) → position(u16 LE)` 로 정정. 30+ 사이클 동안 미발견 본질 결함.

### 2.3 한컴 viewer 자동 장식 영역 (Stage 7)

HWP3 paragraph 26 (cc=8 "￼￼ 제목차례 ") → 한컴 viewer 가 "════════════════════■ 제목차례 ■══════════════════════" 자동 장식 inject. **HWP3 spec 외 한컴 사적 로직** (ParaShape `border` / char_shape `attr` 모두 부재 확정).

**해결**: 한컴 변환본 cross-ref 로 trigger 조건 도출 (새번호 + 쪽번호위치 controls + visible text ≤ 6 chars). 보수적 영역으로 광범위 sweep 회귀 위험 최소화.

### 2.4 HWP3 차례 inline page 번호 ch=1 (Stage 8)

HWP3 차례 entries (paragraph 28~339) 가 페이지 번호를 **inline `ch=1` control** 로 저장. 본 환경 catch-all 분기에서 ch=1 처리 시 header_val1 + ch2 byte 값 unused → page 번호 placeholder (`￼`) 만 표시되어 "EXPORT/IMPORT Q & A → " 처럼 page 번호 누락.

**해결**: HWP3_DIAG_CTRL1 진단으로 byte 패턴 도출 — header_val1 second u16 + ch2 = ASCII digit (0x0030~0x0039), ch2=0x000D 는 1-digit terminator. 3-digit 이상은 literal char 자연 결합 (paragraph 339 page 757 = ch=1("75") + literal "7"). 290 ch=1 occurrences 모두 정합.

## 3. 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1166 passed** (모든 stage 동일) |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `scripts/svg_regression_diff.sh` (devel..HEAD) | TOTAL pages=170 same=170 **diff=0** (회귀 0) |
| 단계별 sweep (a63114e..86bf0bd..ccbb0b6..d03109e..b77d071) | 각각 170/170 same diff=0 |

## 4. 시각 정합 (한컴 viewer + PDF cross-check)

### 페이지 1 (창원대학교 데이타베이스 연구실)

- ✓ image placeholder (점선 사각형 + file path 텍스트) 표시 — 한컴 viewer 정합

### 페이지 2 (제목차례 + TOC)

- ✓ "════════════════════■ **제목차례** ■══════════════════════" 자동 장식 표시 — 한컴 viewer 정합 ★
- ✓ TOC entries `▷ EXPORT/IMPORT/LOADER/ODBC/ORACLE GRAPHICS/RDBMS` markers
- ✓ tab leader 점선 (────) fill (HWP5 conversion fill=3 정합)
- ✓ 페이지 번호 우측 정렬 (TOC entry 끝 page_num)
- ✓ TOC entries 페이지 번호 (1, 4, 5, ..., 134) 표시 — Stage 8 ch=1 정합

## 5. 자료 추가

| 경로 | 용도 |
|------|------|
| `samples/hwp3-sample10.hwp` | HWP3 native sample (Oracle 기술 문서) |
| `samples/hwp3-sample10-hwp5.hwp` | 한컴 HWP5 변환본 (cross-ref 권위) |
| `samples/hwp3-sample10-hwpx.hwpx` | 한컴 HWPX 변환본 |
| `pdf/hwp3-sample10-hwp5-2022.pdf` | 한글 2022 편집기 PDF (PDF 권위 자료, 90MB) |

## 6. 단계별 보고서

| Stage | 보고서 |
|-------|--------|
| Stage 1~4 통합 | `mydocs/working/task_m100_741_stage1to4.md` |
| Stage 5 | `mydocs/working/task_m100_741_stage5.md` |
| Stage 6 | `mydocs/working/task_m100_741_stage6.md` |
| Stage 7 | `mydocs/working/task_m100_741_stage7.md` |
| Stage 8 | `mydocs/working/task_m100_741_stage8.md` |

## 7. 후속 영역 (본 task 외)

- HWP3 leader 값 2+ (점선/파선) 등장 시 추가 매핑 (현재 leader=1 만 처리)
- PUA char (U+F080F, U+F0827 등) 폰트 fallback — Stage 5 매핑 영역의 한컴 사적 폰트 부재 시 표시
- 다른 HWP3 sample 의 "제목차례 type" 패턴 추가 발견 시 trigger 조건 확장
- HWP3 char encoding 0x0080~0x7FFF 영역 잔여 ~298 unique values (1.5% 빈도) 매핑 — 별도 task 영역

## 8. 권위 사례 강화

- **`reference_authoritative_hancom`**: PDF (한글 2022) + HWP5 변환본 cross-ref 영역의 권위 자료 활용 (Stage 5/6/7 매핑 도출)
- **`feedback_visual_judgment_authority`**: 작업지시자 시각 판정으로 본질 결함 발견 (제목차례 ═══ 장식, char_shape 위치 어긋남)
- **`feedback_hancom_compat_specific_over_general`**: 제목차례 inject trigger 조건 보수적 영역 (≤6 chars) 으로 광범위 회귀 회피
- **HWP3 파서 본질 권위 사례 강화**: Stage 6 의 `Hwp3TabDef` 필드 순서 bug — 30+ 사이클 미발견 본질 결함, 진단 영역 (HWP3_DIAG_TABS) 으로 byte pattern 발견
