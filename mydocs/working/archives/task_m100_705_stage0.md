# Task #705 Stage 0 — 사전 측정 + 198 샘플 재조사

## 산출물

- `examples/inspect_705.rs` — aift.hwp 셀 안 PageHide 측정 도구
- `examples/scan_cell_pagehide.rs` — 198 샘플 전수 sweep 도구

## 1. PageHide 모델 정의 (`src/model/control.rs:185-198`)

```rust
pub struct PageHide {
    pub hide_header: bool,        // 머리말 감추기
    pub hide_footer: bool,        // 꼬리말 감추기
    pub hide_master_page: bool,   // 바탕쪽 감추기
    pub hide_border: bool,        // 테두리 감추기
    pub hide_fill: bool,          // 배경 감추기
    pub hide_page_num: bool,      // 쪽 번호 감추기
}
```

→ Stage 3 의 layout.rs 가드 식별자 확정: `hide_fill`, `hide_border`.

## 2. aift.hwp 셀 안 PageHide 측정 (`inspect_705.rs`)

```
[CELL] s0/p[1]/Table[0]/셀[167]/p[3]/ctrl[0]
       text="       년        월        일"
       PageHide(header=true footer=true master=true border=true fill=true page_num=true)

[CELL] s1/p[0]/Table[2]/셀[31]/p[0]/ctrl[0]
       text="최종 목표"
       PageHide(header=false footer=false master=false border=false fill=false page_num=true)

[BODY] s2/p[34]/ctrl[0]   PageHide(...,page_num=true)
[BODY] s2/p[54]/ctrl[0]   PageHide(...,page_num=true)
```

**메인테이너 권위 측정 데이터와 정확 일치** (셀[167]/p[3], 6 필드 모두 true).

**추가 발견**: aift.hwp 의 셀 안 PageHide 가 1건 더 있음 (`s1/p[0]/Table[2]/셀[31]/p[0]`, "최종 목표" text, page_num=true). PR #640 의 H2 측정에서도 누락된 영역.

### 페이지 매핑 (`dump-pages` 결과)

| 페이지 | 외부 paragraph | 셀 안 PageHide |
|--------|---------------|----------------|
| page 2 (idx=1, sec=0, page_num=2) | s0/p[1] (Table 35x27, tac=false) | 셀[167]/p[3] **6필드 true** |
| page 3 (idx=2, sec=1, page_num=3) | s1/p[0] (Table 14x17, tac=false) | 셀[31]/p[0] **page_num true** |

→ 본 환경 결함 #1 (pagination/engine.rs) 정정 시 두 페이지 모두 한컴 정합 (미표시) 으로 전환 예상.

## 3. 198 샘플 전수 sweep (`scan_cell_pagehide.rs`)

### 합계
- 파싱 성공: 198 / 198 (실패 0)
- 본문 PageHide: 95 건
- **셀 안 PageHide: 13 건** (현재 결함 #1 으로 무시되는 영역)
  - 6필드 모두 true: **1** (aift.hwp 셀[167])
  - page_num 만 true: 10
  - 기타: 2 (tac-img-02 의 header 만 true 등)

### 영향 샘플 (셀 안 PageHide 보유, 6 / 198 = 3%)

| 샘플 | 셀안 | 케이스 |
|------|------|--------|
| **aift.hwp** | 2 | 셀[167] HFMBIP (full6) + 셀[31] -----P (pagenum) |
| 2022년 국립국어원 업무계획.hwp | 1 | 셀[0]/p[5] -----P "Ⅱ. 2022년 정책방향	 4" |
| KTX.hwp | 1 | 셀[10]/p[0] -----P " Ⅰ. 사업 개요	 3" |
| kps-ai.hwp | 1 | 셀[0]/p[30] -----P "   4. 제안서 형식	31" |
| tac-img-02.hwp | 4 | 셀[4] H----- "2026. 2. 27." + 셀[0/3/5] -----P (빈) |
| tac-img-02.hwpx | 4 | (HWPX 동일 분포) |

(범례: H=header, F=footer, M=master, B=border, I=fill, P=page_num. `-` = false)

### 인사이트

1. **`s1/p[0]/Table[2]/셀[31]/p[0]`** 의 page_num 케이스는 aift.hwp page 3 의 "최종 목표" (cover-style 두번째 페이지) — PR #640 의 H1 cover-style 휴리스틱이 이 페이지를 잡아낸 이유는 우연 (구조 매칭) 이지만 본질은 셀 안 PageHide.

2. **목차/별첨 페이지 패턴** — 2022 국립국어원, KTX, kps-ai, aift 의 목차 페이지가 모두 큰 표 안에 목차 항목들을 둔 후 셀에 PageHide(page_num) 를 넣은 형식. 한컴 사용자 흔한 패턴.

3. **HWP/HWPX 동치** — tac-img-02 의 .hwp 와 .hwpx 가 동일한 셀 안 PageHide 분포 → HWP5 결함 정정이 HWPX 에도 동일 효과.

4. **중첩 표 (depth 2+) 없음** — sweep 결과 모든 셀 안 PageHide 가 depth 1 (외부 표 1 단계). Stage 2 재귀 함수는 재귀 호출 자체는 유지하되 실측 데이터 기준 1 depth 만 적용.

## 4. Stage 1 테스트 케이스 후보

수행 계획서 / 구현 계획서의 4 건 외 추가 권고:

| 테스트명 | 대상 | 검증 |
|---------|------|------|
| `test_705_aift_page2_cell_pagehide_hides_page_number` | aift.hwp page 2 | footer 글리프 0 (full6) |
| `test_705_aift_page2_cell_pagehide_hides_border` | aift.hwp page 2 | 쪽 테두리 미렌더 |
| `test_705_aift_page2_cell_pagehide_hides_fill` | aift.hwp page 2 | 페이지 배경 미렌더 |
| `test_705_aift_page3_cell_pagehide_hides_page_number` | aift.hwp page 3 | footer 글리프 0 (pagenum) |
| **`test_705_kor2022_cell_pagehide_hides_page_number`** | 국립국어원 목차 페이지 | footer 글리프 0 (pagenum) |
| **`test_705_ktx_cell_pagehide_hides_page_number`** | KTX 목차 페이지 | footer 글리프 0 (pagenum) |

(국립국어원/KTX 케이스는 aift 외 회귀 가드 — 다른 샘플도 본질 정정의 혜택 검증)

tac-img-02 는 표가 페이지 전체를 차지하는 구조 (cover-style) — 페이지 매핑 확인 후 별도 결정.

## 5. 위험 평가 (재확인)

| 항목 | Stage 0 결과 | 영향 |
|------|-------------|------|
| 셀 안 PageHide 분포 | 6 샘플 / 198 (3%) | 회귀 위험 영역 좁음 |
| 중첩 표 (depth 2+) | 실측 0 건 | Stage 2 재귀 1 depth 면 충분 |
| HWP5 vs HWPX 동치 | tac-img-02 동일 분포 | 본 정정으로 양쪽 동일 효과 |
| 본문 PageHide (95건) | 결함 영향 X (이미 정확 처리) | Stage 5 회귀 sweep 으로 보호 |

## 6. Stage 1 진입 결정

- 수행 계획서 / 구현 계획서의 단계 구조 유지 (단계 추가 없음)
- 신규 테스트 2건 추가 (국립국어원 + KTX) — 4 → 6 건
- Stage 2 의 재귀 함수는 1 depth 면 충분 (실측 근거)

## 관련

- 수행 계획서: `mydocs/plans/task_m100_705.md`
- 구현 계획서: `mydocs/plans/task_m100_705_impl.md`
- Issue #705: https://github.com/edwardkim/rhwp/issues/705
- 메인테이너 본질 결함 발견: PR #638 코멘트 (https://github.com/edwardkim/rhwp/pull/638#issuecomment-4402585484)
