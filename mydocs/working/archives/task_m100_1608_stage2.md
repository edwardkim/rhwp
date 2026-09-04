# Stage 2 완료보고서 — Task #1608

**단계**: tolerance 제거 수정 (GREEN) · **브랜치**: `local/task1608`

## 수정 내용

### `src/parser/hwpx/mod.rs`
- `is_hwp3_origin = (head version == "1.4")` 판정 제거.
- `if is_hwp3_origin { … pagination_bottom_tolerance = margin_bottom.min(1600) }` 블록 제거.
- `hwpml_version` 파싱 + `doc_info.hwpml_version` 무손실 보존은 **유지**(직렬화 재방출용).
- 제거 사유를 #1608 주석으로 명시.

### `src/parser/hwpx/header.rs`
- `parse_hwpx_hwpml_version` docstring 정정: "변환본=1.4 / 직접작성=1.5+ (Task #554 6/6)"
  오버핏 주장을 삭제하고, head version 은 HWPML 스키마 버전일 뿐 변환 지표가 아님을 명시.

## GREEN 확인

```
test native_hwpx_v14_has_no_hwp3_pagination_tolerance ... ok
test result: ok. 1 passed; 0 failed
```

> Stage 1(RED 테스트)과 Stage 2(수정)는 broken commit(빨강 커밋)을 남기지 않도록 하나의
> 구현 커밋으로 묶어 GREEN 상태로 기록한다(CLAUDE.md 단계 커밋 규칙의 취지 유지).
