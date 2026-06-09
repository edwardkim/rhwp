# 구현계획서 — 수식 집합연산자(∩/∪) 1.5배 확대 렌더 수정 (M100 #1342)

## 설계 결론

파서 경로 분석:
- `parser.rs:337` `is_big_operator(cu)` 가 true 면 `parse_big_op()` → `layout_big_op()` → `op_fs = fs*1.5`.
- false 로 떨어지면 `parser.rs:544` `lookup_symbol(cmd)` → `EqNode::MathSymbol` → `layout_math_symbol()` → 본문 크기(fs), 패딩 없음. **이것이 원하는 결과.**

`lookup_symbol` 은 case-sensitive HashMap 조회(1차 원본 case, 2차 대문자)를 한다. 따라서 BIG_OPERATORS 에서 제거만 하면 대문자 키(`CUP`,`INTER` 등)는 어디서도 매핑되지 않아 `Text("CUP")` 로 깨진다.

**∴ 단순 제거가 아니라 BIG_OPERATORS → OPERATORS 로 키를 이동**해야 `is_big_operator()=false` 이면서 `lookup_symbol()` 정상 매핑이 동시에 성립한다.

## 대상 키 (BIG_OPERATORS → OPERATORS 이동)

| 키 | 기호 |
|----|------|
| `UNION` | ∪ |
| `SMALLUNION` | ∪ |
| `CUP` | ∪ |
| `INTER` | ∩ |
| `SMALLINTER` | ∩ |
| `CAP` | ∩ |

- `BIGCUP`/`BIGCAP` 및 lowercase `bigcup`/`bigcap` 은 **BIG_OPERATORS 유지** (진짜 큰 형태).
- lowercase `cup`/`cap` 은 이미 OPERATORS(symbols.rs:250-251)에 존재 → 변경 없음. (이동 후 대문자 변환 경로가 BIG 를 먼저 잡지 않으므로 자연 정상화)
- 본 PDF/샘플이 사용하지 않는 `SQCUP`/`SQCAP`/`UPLUS`/`OPLUS`/`OTIMES` 등은 동일 오분류 가능성이 있으나 **정답지 근거 부재로 본 타스크 범위 외** (보고서에 후속 검토 기록).

## 단계별 구현

### Stage 1 — 소스 수정 + 단위 테스트
1. `src/renderer/equation/symbols.rs`
   - BIG_OPERATORS 에서 위 6개 키 제거
   - OPERATORS 의 「집합/논리」섹션에 위 6개 키 추가 (∪/∩)
2. 단위 테스트 추가 (`symbols.rs` tests 모듈)
   - `assert!(!is_big_operator("CAP"))`, `!is_big_operator("CUP")`, `!is_big_operator("SMALLINTER")`
   - `assert_eq!(lookup_symbol("CUP"), Some("∪"))`, `lookup_symbol("CAP")==Some("∩")`, `lookup_symbol("SMALLINTER")==Some("∩")`
   - 회귀: `assert!(is_big_operator("BIGCUP"))`, `is_big_operator("SUM")`
3. `layout.rs` tests 에 ∩ 크기 회귀 테스트 추가 (선택)
   - `A{cap}B` 레이아웃에서 ∩ MathSymbol box 높이가 fs(확대 없음) 인지 확인

### Stage 2 — 빌드/렌더 검증
1. `cargo build --release`
2. `export-svg -p 15` → ∩/∪ `font-size` 가 본문(12)과 동일한지 글리프 좌표 추출 확인
3. PDF 16페이지 대비 ∩/∪ 크기·간격 시각 정합 (crop 비교)
4. `cargo test` 전체 통과 (회귀: ∑/∏/∫/행렬/첨자)
5. 단계 완료보고서 `mydocs/working/task_m100_1342_stage2.md`

### Stage 3 — 최종 보고서
1. `mydocs/report/task_m100_1342_report.md` 작성 (원인/수정/검증/회귀/후속(#1343, 비-BIG 기타 연산자))
2. 계획서 `plans/archives/` 이동
3. `git status` 확인 후 커밋

## 검증 기준 (완료 정의)

- ∩/∪ `font-size` == 본문 글자 크기 (확대 0)
- PDF 16페이지와 ∩/∪ 크기·간격 시각 정합
- `cargo test` 전부 통과, ∑/∏/∫ 큰연산자 회귀 없음
