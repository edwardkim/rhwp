# #1950 최종 결과보고서 — HWP3→HWP5 변환 탭 char 위치 단위 통일

- 이슈: edwardkim/rhwp#1950 (M100)
- 브랜치: `local/task1950`
- 유형: HWP3 파서 char 모델 — 변환(직렬화) 정합
- 결과: **유의 결함(376px) 해소, 원본 렌더 불변**

## 1. 문제

HWP3 문서를 HWP5 로 변환 저장 후 렌더가 원본과 이탈(20k 서베이 7건). 최대 376px, 1건 페이지 증가.

## 2. 근본 원인 (측정 기반 재진단)

Stage 1 에서 계획서 가설(4/8유닛 슬롯 + #1915 secd)을 실측 검증한 결과:

- **#1915(secd) 는 원인이 아님**(`RHWP_NO_SECD` ON/OFF 드리프트 동일) — 디커플.
- **7건은 이질 군집**: 유의 결함은 **2955331(탭 376px) 1건**, 나머지 6건(15047877 페이지
  증가 포함)은 razor-thin(1~13px, #1759/#1842 계열) — 분리 승인.

**유의 결함(2955331) 원인 — char 위치 단위 불일치**:
- HWP5 파서는 `char_offsets`/char_shape `start_pos` 를 UTF-16 **code-unit**(탭=8) 으로 산출.
- HWP3 파서는 탭을 **1 code-unit** 으로만 세어 IR 단위가 어긋남
  (ir-diff: `char_offsets[17] A=17 vs B=24`).
- 직렬화기는 탭을 8-unit 확장하므로, HWP3-origin IR 을 HWP5 로 저장하면 char_shape[1]
  (자간 0%)이 code-unit 중간(탭 확장 지점)부터 적용 → 탭 폭 변화·탭 run 3+1 분할·376px.

## 3. 수정 (1줄)

`src/parser/hwp3/mod.rs` 탭(ch==9) 파싱: `utf16_len += 1` → **`utf16_len += 8`**.
char_offsets/char_count/char_shape start_pos 를 HWP5 시멘틱(code-unit)으로 통일.
논리 char↔shape 매핑 불변 → **원본 HWP3 렌더 불변**, 직렬화기와 정합.

## 4. 검증

### 효과
- 2955331 `render-diff --via hwp`: **OVER 376px → PASS**.

### 무회귀 (원본 렌더 불변)
- **golden SVG(`svg_snapshot`)**: 5건 실패는 CRLF 노이즈, `\r` 정규화 후 golden 대비
  **byte-동일**(표 포함) → 렌더 출력 불변 확정.
- `cargo test --lib`: **2126 passed**.
- **HWP3 표본**(admrul 15건 render-diff): 13 PASS / 2 非PASS. 2 非PASS(2912183·2914239)는
  **탭 0개** razor-thin(≤2.67px, node 동수) — 탭 전용 수정이 건드릴 수 없음(무영향 확인).

## 5. 산출물

- 계획: `mydocs/plans/task_m100_1950.md`
- 단계 보고: `mydocs/working/task_m100_1950_stage1~3.md`
- 소스: `src/parser/hwp3/mod.rs`
- 회귀 가드: `tests/issue_1950_hwp3_tab_charoffset.rs`
- 재현 샘플(공개): `samples/issue1950_hwp3_tab_charoffset.hwp`

## 6. 잔여 (분리 승인)

razor-thin 6건(15047877 PAGE 1→2 포함)은 줄/행높이·페이지 경계 razor 계열(#1759/#1842) —
본 이슈 범위 밖, 별도 추적.
