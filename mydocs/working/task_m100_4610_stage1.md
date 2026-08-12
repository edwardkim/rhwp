# Task #4610 Stage 1 — 공백-전용 TAC 캐리어 문단 페인트 변위

Issue: #4610 (#4599 캠페인 Phase 3, ④ 계열 1차 사이클)
Branch: `fix/4610-whitespace-tac-carrier-paint-y` (base: devel 572786d02)

## 무엇을

서울시 결재문서 템플릿에서 공백 텍스트만으로 treat_as_char 표(문서번호란 1×2 등)를
실어 나르는 문단이, 선행 문단이 앵커한 자리차지(TopAndBottom) 표에 밀려 표 하단
아래로 렌더되던 결함을 고쳤다. 렌더 y 만 저장 lineseg vpos 위치(표 위 틈)로 되돌리고
흐름 전진량은 보존하는 페인트 변위 계약을 추가했다.

- `src/renderer/layout.rs` — 게이트 `whitespace_tac_carrier_stored_paint_y` +
  FullParagraph 두 배치 경로(일반 `layout_paragraph` / `layout_inline_table_paragraph`)에
  페인트-흐름 분리 적용
- `src/renderer/layout/tests.rs` — 게이트 단위 테스트 7종

## 왜 (실측 근거)

야간방호일지 36374873 p1 — #4599 Phase 1b CONFIRMED 최대 편차(821.6px) 대표:

- 한글 2022 캐시 PDF: 문서번호 1×2 표가 결재란(3×4 TAC) 아래·본표(13×8 자리차지) 위
  틈 y≈265.6 에 위치. 저장 lineseg(pi4 seg0 vpos=13575 → y 256.6)도 같은 위치 증언.
- 종전 rhwp: 자리차지 표가 흐름 커서를 표 하단(≈1064)까지 밀어 pi4 전체가 y≈1085
  → 문서번호 표 821px 하방. 꼬리 문단(pi5~7)은 PDF 와 정합(frame +2.1px)이므로
  흐름 전진은 유지해야 한다 — 페인트 변위로 좁힌 이유.

한글 2022 의 실제 규칙은 TopAndBottom 자리차지 표 뒤의 줄 단위 band-flow(틈에 들어가는
줄은 틈에 배치)이지만, 전면 도입은 #3386 전면 스냅 반증 이력과 같은 회귀 위험이 있어
시각 결함의 실체(공백-전용 TAC 캐리어)로 서명-한정했다.

## 게이트 (서명-한정·방향-한정)

1. hwpx stored layout 프로파일
2. 공백-전용 문단 (시각 문자는 있으나 비공백 문자 없음)
3. 컨트롤이 treat_as_char 표 정확히 1개 (float host·그림/도형 문단 제외)
4. 모든 lineseg 저장-태그 (합성 사다리 제외)
5. 문단-내 세그 간 간격 > 7500HU(100px) — 저장 당시 레이아웃의 개체 밴드 증거.
   낡은 세대 사다리(#4599 QUIET 47 의 문단 간격 누락류, 13.3px 스케일)는 이런
   거대 간격을 만들지 않는다.
6. TAC 가 첫 줄에 탑승 (comp.tac_controls[0].pos < seg1.text_start)
7. 후방 변위만: 흐름 − 저장 y ≥ 세그 간 간격 × 0.5

## 검증 실측

### 대상 문서 (야간방호일지 36374873)

| 항목 | 종전 | 수정 후 | 오라클 |
|---|---|---|---|
| 문서번호 1×2 표 y | 1079.5 | **263.1** | PDF 265.6 / 사다리 256.6+om |
| pi5~7 (붙임 꼬리) | 1146.8/1168.1/1189.5 | 동일 (불변) | PDF frame +2.1 정합 |
| phase1b 판정 | CONFIRMED maxdev 821.6 | **WEAK maxdev 5.2** (n=19 동일) | |

### 게이트 단위 테스트

`cargo test --release --lib whitespace_tac_carrier` — **7 passed / 0 failed** (되돌림
정위치 / 프로파일·본문 텍스트·float host·소간격(낡은 사다리)·전방 변위·합성 세그 거부)

### hwpx 3418 스윕 (baseline 클린 devel 572786d02 vs 수정판)

`sweep_combo.py`(vld_guarded 판정 + 표 지오메트리 서명, 캠페인 홈에 보존) — baseline 은
worktree 클린 빌드(가드레일: 기존 바이너리 재사용 금지 준수).

- 판정 분포 동일: DRIFT 244 / OK 2561 / SKIP 604 / ERR 9 (전후 완전 일치, 이동 0건)
- worst_px +2px 초과 악화 **0건**, 개선 0건
- 야간방호일지가 DRIFT 로 남는 것은 예상 동작 — 검출기는 렌더 vs 저장 사다리를 재는데
  비가시 공백 줄(pi3)은 흐름 유지를 위해 이동하지 않았다. 실물 판정은 PDF 대조가 정본
  (CONFIRMED→WEAK).

산출물: `_oracle_pdf_2022/sweep3418_5727_base_combo.tsv` · `sweep3418_4610fix_combo.tsv`

### 전수 Table-diff (표 지오메트리 서명 비교)

전 3418 문서의 모든 Table 노드 y/x/h/w 서명 비교 — **변동 1건: 야간방호일지 36374873**
(의도된 수정 대상). 그 외 표 이동 문서 없음.

### #4599 Phase 1b 재판정 (backlog 242)

`phase1b_confirm.py` 를 수정 바이너리로 재실행 (입력·판정 규약 동일):

- **이동 1건**: 야간방호일지 36374873 CONFIRMED(maxdev 821.6) → WEAK(5.2)
- **불변 241건**, 비CONFIRMED→CONFIRMED 0건, maxdev +15px 이상 증가 0건

서명-한정 게이트가 의도대로 대상 형상에만 개입함을 확인. SWAP 형상 군집의 나머지
CONFIRMED 는 하위 형상이 달라(비공백 캐리어 등) 별도 사이클 대상.

### 시각 증적 (SVG 전후)

`export-svg -p 0` — 1×2 문서번호 표 테두리 요소(x=79.36): baseline y=**1079.49** →
수정 y=**263.07** (PDF 실측 265.6 과 2.5px 이내, 문서 전역 frame +2.1px 감안 시 정합).
꼬리 문단 요소들의 y 는 전후 동일.

### 저장소 게이트

- cargo fmt --check: 통과 (변경 파일 포맷 적용 후 무출력)
- cargo clippy --release: 경고 0
- cargo test --release 전체: exit 0 (lib 3505 passed / 0 failed / 13 ignored,
  통합 테스트 타깃 전부 ok)
- wasm-pack build --target web: 성공 (exit 0)
- 시각 증적: SVG 전후 비교 (위 절)
