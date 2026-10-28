# PR #2425 검토 — 다중 페이지 표 경계 판정 page 한정 (postmelee, #2400)

- #2401 에서 분리된 #2400 의 자기 해결 — 계획서·단계 2 동봉 정식 타스크
- 본질: hitTest 는 정확했으나 isTableBorderClick 이 page 무전달 →
  getTableBBox 가 page 0 첫 fragment 반환 → page 113 로컬 좌표와 이좌표계
  비교 (3.7px 오염 vs 실제 18.6px). 신설 getTableBBoxAtPage 는 **타 page
  fallback 없는 명시 실패** 설계. #2215 시멘틱 불변 명시.

## 재실증

| 게이트 | 결과 |
|--------|------|
| Rust 스위트/fmt/clippy + wasm_api 신규 테스트 | 0 실패 / OK / 0 |
| wasm Docker 재빌드 (신규 export 반영 — 스테일 pkg 로 tsc 실패 재확인 후) | getTableBBoxAtPage 실재 |
| studio tsc / npm test | OK / 452/452 |
| red→green (엔진 TS 원복) | 4 FAIL → 6/6 |
| **이미지 스왑 검증 (CDP, 확정 좌표 재현)** | before: tableSelected=true (핸들 8개) / after: caret para2499·offset77 정확 착지 — assets/pr2425_table_border_page_scope_before_after.png |

## 처리 결과

merge 완료(admin) + 스왑 이미지 첨부 감사 코멘트. #2400 은 close-issues 대기.
