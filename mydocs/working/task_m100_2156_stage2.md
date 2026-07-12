# Task M100 #2156 — Stage 2 완료보고서 (Haansoft 대체 가설 확정 + 구현 사양)

**날짜**: 2026-07-10 / **브랜치**: `local/task2156`

## 1. 정밀 판별 (k=1600 사다리)

| 클래스 | 한글 실측 구간(em) | HCR Batang | Haansoft Batang |
|--------|-------------------|-----------|-----------------|
| digit '0' | [0.571, 0.596) | 0.550 **배제** | **0.583 ✓** |
| middot '·' | [0.323, 0.348) | 0.320 **배제** | ≈0.333 ✓ |

stage1 의 A/a/괄호/쉼표와 합쳐 **전 판별 클래스에서 Haansoft 적합, HCR 배제**.

## 2. 확정 사양

**한글은 함초롬바탕 문서의 비한글 문자(라틴·숫자·구두점·middot)를
Haansoft Batang(한컴바탕, HBATANG.TTF) 메트릭으로 렌더한다.** 한글 음절만
HCR Batang hmtx. 공백은 useFontSpace=0 → 고정 0.5em (rhwp 이미 정합).

## 3. 구현 계획 (stage 3)

1. `src/renderer/layout/text_measurement.rs`: 함초롬 계열(HCR *) 폰트의
   ASCII(0x20~0x7E)+middot advance 를 Haansoft 테이블로 대체.
   - 테이블: `output/poc/task2156/haansoft_ascii_table.rs.txt`
     (추출 도구 `tools/extract_haansoft_table.py`, upm=1024)
   - 주요 값: 괄호 0.500, 쉼표·마침표 0.291, 숫자 0.583, A 0.750, a 0.500
2. **#2150 em 공식과 동시 적용** (NO_LS 셀 마지막 줄=em — 적용 지점 5개는
   `mydocs/working/task_m100_2150_stage3.md` 참조): 두 축이 함께 맞아야
   규제영향분석서 계열의 오차 상쇄 의존이 풀린다.
3. 게이트: 21761835 세그 오라클(`tools/hangul_row_heights2.py` 누적 ±수 px),
   issue_1891(76076=82/80168=157/80250=17/86712=65), issue_1842/1623/2146,
   byeolpyo4=26, 359 recount(기준선 recount2146d, REGRESSED 0),
   전체 cargo test (svg_snapshot 4건 CRLF 로컬 노이즈 제외).
4. 범위 확인(후속): 함초롬돋움/비한컴 폰트 문서의 대체 규칙.
