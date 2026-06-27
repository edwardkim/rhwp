# Task #1584 — Stage 1 완료보고서

**단계**: 드롭 재현 회귀 테스트 박제 (RED)
**브랜치**: `local/task1584`

## 작업 내용

`src/serializer/hwpx/section.rs` 테스트 모듈에 회귀 가드 추가:
`task1584_body_first_para_two_columndefs_roundtrip`.

- 본문 첫 문단에 `ColumnDef` 2개를 둔 최소 Document 를 구성.
- `serialize_hwpx → parse_hwpx` 전체 roundtrip 수행.
- reparse 후 첫 문단의 `ColumnDef` 개수가 2인지 단언.

## 결과 (RED 확인)

```
assertion `left == right` failed: ... ColumnDef 2개가 ... 보존돼야 한다: 1
  left: 1
 right: 2
```

- **현재 코드: 2번째 인라인 ColumnDef 드롭** → reparse 1개. 버그 정확 재현.
- 근본원인(수행/구현 계획서 §2) 일치: 템플릿 앵커가 첫 ColumnDef 1개만 흡수,
  본문 인라인 슬롯 필터가 ColumnDef 전부 제외 → 2번째+ 드롭.

## 다음 단계

Stage 2 — Option A 구현(C1~C4)으로 GREEN 전환 + baseline 회귀 0 확인.
