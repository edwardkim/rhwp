# Stage 2 완료보고서 — Task #1611 (RED)

**단계**: RED 테스트 + fixture · **브랜치**: `local/task1611`

## fixture
`samples/hwpx/opengov/36387725_footer_page_bottom.hwpx` — corpus 36387725 복사(46 KB).
opengov 결재문서 fixture 선례(`samples/hwpx/opengov/`)에 정합. 한글 정답지 2쪽(통제셋).

## RED 테스트
`tests/issue_1611_footer_page_bottom_pagination.rs` — `page_count()==2` 단언.

수정 전 실행 — **RED 확인**:
```
left: 1   right: 2
발신명의 footer(Page+Bottom) page-fit 누적 과소로 1쪽에 흡수됨 (#1611 요인 B)
```
