# #1950 Stage 3 — 구현 + 검증

- 브랜치: `local/task1950` / 범위: 2955331(탭 376px) — razor-thin 6건 분리 승인됨

## 1. 수정 내용 (1줄 실질 변경)

`src/parser/hwp3/mod.rs` 탭(ch==9) 파싱에서 `utf16_len += 1` → **`utf16_len += 8`**.

HWP5 시멘틱상 탭은 PARA_TEXT 에서 8 code-unit(0x0009 + 확장 7)을 차지한다. HWP3 파서가
탭을 1 code-unit 으로만 세어 `char_offsets`/`char_count`/char_shape `start_pos`
(`hwp3_char_to_utf16_pos`)의 단위가 HWP5 와 어긋났다. 8 로 통일해 IR 을 HWP5 시멘틱과
일치시킨다.

- 논리 char↔char_shape 매핑은 불변(같은 char 가 같은 shape) → **원본 HWP3 렌더 불변**.
- 직렬화기(탭 8-unit 확장)와 정합 → 변환·재파스 후 char_shape 정렬 유지 → 376px 해소.

## 2. 검증

### 효과 (2955331)
- `render-diff --via hwp`: **OVER 376px → PASS**. (탭 run 4개 유지, 자간 −5% 유지)

### 무회귀 (핵심 — 원본 렌더 불변)
- **golden SVG(`svg_snapshot`)**: 5건 "실패" 는 전부 CRLF 노이즈, `\r` 정규화 후 golden
  대비 **byte-동일**(표 케이스 `table-text/page-0` 포함). → 수정이 **렌더 출력을 전혀
  바꾸지 않음** 확정.
- `cargo test --lib`: **2126 passed, 0 failed**.
- **HWP3 표본 회귀**(admrul 15건 render-diff --via hwp): 13 PASS / 2 非PASS.
  2 非PASS(2912183·2914239)는 **탭 0개** 문서의 razor-thin(≤2.67px, node 동수) — 본 수정이
  건드릴 수 없음(탭 전용 변경). 즉 **탭 없는 문서 완전 무영향**, razor-thin 별개 축 그대로.

### 국소성 근거
수정은 탭 문단에만 영향. 탭 없는 문서는 char_offsets 불변 → 렌더 bit-identical(svg_snapshot·
탭0 문서 확인). 탭 문단도 논리 매핑 불변이라 원본 렌더 불변, 변환본만 정합.

## 3. 회귀 가드 / 재현

- 공개 샘플: `samples/issue1950_hwp3_tab_charoffset.hwp`(법원 예규 미제사건보고서, HWP3).
- 회귀 테스트: `tests/issue_1950_hwp3_tab_charoffset.rs` — (1) HWP3 파싱 탭 char_offset 증가폭
  == 8, (2) HWP5 왕복 후 탭 문단 char_count 정합(수정 전 31→88 팽창 검출).

## 4. 잔여 (분리 승인)

- razor-thin 6건(15047877 PAGE 1→2 포함, 1~13px)은 줄/행높이·페이지 경계 razor 계열
  (#1759/#1842) — 본 이슈 범위 밖, 별도 추적.

## 5. 게이트 결론

2955331 376px 해소 + 원본 렌더 bit-identical(golden 불변) + lib 2126 + HWP3 표본 무회귀
(탭 없는 문서 무영향). 풀 스위트·스냅샷은 CI 확인.
