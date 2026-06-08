# 최종 결과보고서 — 수식 집합연산자(∩/∪) 1.5배 확대 렌더 수정 (M100 #1342)

## 1. 개요

| 항목 | 내용 |
|------|------|
| 이슈 | #1342 |
| 브랜치 | `local/task1342` |
| 증상 | 16페이지 문24 `P(A∩B)` 등에서 ∩/∪이 본문 1.5배로 확대되고 뒤 공백 발생 |
| 정답지 | `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf` 16페이지 (한글2022) |
| 샘플 | `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` |

## 2. 근본 원인

수식 파서는 토큰을 대문자 변환(`cap`→`CAP`) 후 `is_big_operator()` 로 큰 연산자 여부를 판정한다. 소형 이항 집합연산자 키(`UNION`,`SMALLUNION`,`CUP`,`INTER`,`SMALLINTER`,`CAP`)가 **`BIG_OPERATORS` 테이블에 잘못 등록**되어 있어, ∑·∏·∫ 처럼 `parse_big_op()` → `layout_big_op()` 의 `op_fs = fs * BIG_OP_SCALE(1.5)` 경로를 타고 1.5배 확대 + 큰연산자 trailing 박스폭이 부가되었다.

SVG 글리프 좌표 추출로 확정: ∩ `font-size=18` (주변 본문 12) = 정확히 1.5배.

소문자 `cup`/`cap` 은 이미 OPERATORS 에 있었으나, 파서의 대문자 변환 경로가 BIG_OPERATORS 를 먼저 조회하여 무력화되었다.

## 3. 수정 내용

`src/renderer/equation/symbols.rs` (단일 파일):

1. BIG_OPERATORS 에서 6개 키 제거: `UNION`,`SMALLUNION`,`CUP`,`INTER`,`SMALLINTER`,`CAP`
2. OPERATORS 「집합/논리」섹션에 동일 6개 추가 (∪/∩)
3. `BIGCUP`/`BIGCAP` 은 BIG_OPERATORS 유지 (진짜 큰 형태)
4. 회귀 테스트 `test_set_operators_not_big` 추가

**설계 근거**: `lookup_symbol` 은 case-sensitive 조회이므로 BIG 에서 단순 제거 시 대문자 키가 어디서도 매핑되지 않아 `Text("CUP")` 로 깨진다. 따라서 **BIG → OPERATORS 키 이동**으로 `is_big_operator()=false` 와 정상 기호 매핑을 동시 달성. OPERATORS 매핑 → `EqNode::MathSymbol` → `layout_math_symbol()` → 본문 크기·무패딩 렌더.

## 4. 검증

| 검증 | 결과 |
|------|------|
| ∩/∪ font-size | 18 → 12 (본문 동일), 6개 전부 정규화 |
| 글리프 정렬 | y 21.24→17.64, A∩B 밀착 |
| PDF 시각 정합 | 16페이지 정답지와 일치 |
| `cargo test --release` | 1614+49 passed, 0 failed |
| `cargo clippy --lib` | symbols.rs 경고 없음 |
| 회귀 (∑/∏/∫/행렬/첨자) | 무결 |

## 5. 후속 과제

| 항목 | 처리 |
|------|------|
| `P(A\|B)` 조건부 막대 U+E04D(PUA) 두부(▦) 렌더 | 별도 이슈 **#1343** 등록 |
| `SQCUP`/`SQCAP`/`UPLUS`/`OPLUS`/`OTIMES` 등 비-BIG 소형 연산자 동일 오분류 가능성 | 정답지 근거 확보 시 후속 검토 (본 타스크 범위 외) |

## 6. 결론

집합연산자 ∩/∪ 의 1.5배 확대 버그를 BIG_OPERATORS 오분류 교정으로 해소. PDF 정답지와 시각 정합하며 전체 회귀 무결. 이슈 #1342 종료 가능.
