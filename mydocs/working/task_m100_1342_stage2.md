# Stage 2 완료보고서 — 빌드/렌더 검증 (M100 #1342)

## 변경 요약 (Stage 1 산출물)

`src/renderer/equation/symbols.rs` 단일 파일:
- BIG_OPERATORS 에서 소형 집합연산자 6개 제거: `UNION`, `SMALLUNION`, `CUP`, `INTER`, `SMALLINTER`, `CAP`
- OPERATORS 「집합/논리」섹션에 동일 6개 추가 (∪/∩ 매핑)
- `BIGCUP`/`BIGCAP` 은 BIG_OPERATORS 유지 (진짜 큰 형태)
- 회귀 테스트 `test_set_operators_not_big` 추가

## 검증 결과

### 1. 글리프 크기 정합 (핵심)

16페이지 `P(A∩B)` 의 ∩ 글리프 좌표 추출:

| | 수정 전 | 수정 후 |
|---|---|---|
| ∩ font-size | **18.00** (본문의 1.5배) | **12.00** (본문과 동일) |
| ∩ y좌표 | 21.24 (아래로 처짐) | 17.64 (본문 정렬) |
| A→∩→B 배치 | A∩ B (뒤 공백) | A∩B (밀착) |

페이지 전체 ∩/∪ 6개 모두 `font-size=12.00` 으로 정규화 확인 (수정 전 18.00).

### 2. PDF 시각 정합

`pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf` 16페이지 대비:
- `P(A∩B)`, `P(A∪B)`, `P(A∩B)/P(B)` 모두 본문 크기·밀착 간격으로 PDF 정답지와 일치
- 잔여 `P(A▦B)` 두부는 U+E04D 조건부 막대(별건 #1343)

### 3. 회귀 테스트

```
cargo test --release  → 1614 passed; 0 failed (lib)
                         49 passed; 0 failed (통합)
                         0 failed (전체, exit 0)
cargo clippy --release --lib  → symbols.rs 경고 없음
```

- ∑/∏/∫ 큰연산자, 행렬, 첨자, 극한 관련 기존 테스트 전부 통과 (회귀 없음)
- 신규 `test_set_operators_not_big` 통과

## 범위 외 (기록)

- `SQCUP`/`SQCAP`/`UPLUS`/`OPLUS`/`OTIMES` 등 비-BIG 소형 연산자도 동일 오분류 가능성. 정답지 근거 부재로 본 타스크 미포함 — 최종 보고서 후속 항목.
- U+E04D 조건부 막대 두부 → 이슈 #1343.

## 결론

근본 원인(BIG_OPERATORS 오분류 → 1.5배 확대) 해소. PDF 정합·회귀 무결. Stage 3(최종 보고서) 진행 가능.
