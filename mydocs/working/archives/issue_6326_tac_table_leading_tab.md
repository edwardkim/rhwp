# #6326 탭 한 글자만 있는 문단의 자리차지 표가 탭 폭만큼 우측으로 밀려 용지 밖으로 잘림

## 무엇을

사용자가 제공한 실제 문서(SBS미디어넷 참여기업 모집공고, `.hwp`) 10쪽 "제출서류" 표가
오른쪽으로 밀려 용지 오른쪽 끝을 넘어갔다. 같은 쪽 아래의 "가점관련 증빙서류" 표(폭이 거의
동일)는 정상 위치였다. rhwp-studio에서 열었을 때 표가 가운데 정렬이 안 되고 오른쪽으로 쏠려
보인다는 사용자 신고로 시작했다.

## 왜 (원인)

문단#155의 텍스트는 탭(U+0009) 한 글자뿐이고 바로 뒤에 `treat_as_char` 표가 온다. 표 폭이 줄
폭의 90% 이상이라 `is_tac_table_inline_in_para`가 false → block 취급되어
`composed.tac_controls`에 없고, `compute_tac_leading_width`(`src/renderer/layout.rs`)가
`None` 분기로 가서 줄 0의 모든 run(=탭 하나)의 폭을 leading 으로 합산해 표가 그만큼 오른쪽으로
밀렸다.

`#6167`이 도입한 구제 조건(`stored_ladder_gives_tac_table_its_own_line`)은 `.skip(1)`이라
표가 문단 첫 글자여서 자기 줄이 `ls[0]` 하나뿐인 경우(이 문서가 정확히 이 형상)를 구제하지
못한다. 같은 함수의 같은 갈래를 짚는 인접 OPEN 이슈 `#6298`(표 *뒤* 공백이 leading 으로 실리는
사례)도 같은 원인 설명을 남겼지만 트리거가 다르다(표 뒤 공백 vs 표 앞 탭).

## 어떻게 (변경)

`compute_tac_leading_width`의 block 취급(`tac_pos_opt.is_none()`) 분기에 탭 전용 가드를
추가했다: 줄 0의 모든 run 이 탭(`\t`)뿐이면 leading 을 0 으로 반환한다.

**스페이스가 아니라 탭만 특정한 이유**: 기존 단위 테스트
`test_tac_leading_width_block_table_full_line`(Task #146 v3, `text-align.hwp` 문단 0.2
오라클)은 스페이스 4개의 leading 36.8px 를 요구한다 — 스페이스는 일반 문자처럼 실측 폭만큼
흐름에 남는 게 정상이다. 반면 탭은 "다음 탭 위치로 점프"라는 그리드 스냅 의미라, 뒤에 오는
내용이 곧바로 자기 줄을 차지하는 block 표일 때는 그 점프 목표 자체가 무의미해진다 — 한글도
이런 표를 자기 줄 좌단에 그린다(사용자가 실제 한글 뷰어로 확인).

`복학원서.hwp` pi=16(PUA 필러 U+F081C 기반 leading, `#1195`)이나 스페이스 기반 leading
(`text-align.hwp`)은 탭이 아니므로 이 조건에 걸리지 않아 그대로 보존된다.

## 검증

### 실제 문서 (render tree 좌표, px @96dpi)

| | 수정 전 x | 수정 후 x | 폭 | 우측 끝(수정 후) |
| --- | ---: | ---: | ---: | ---: |
| 제출서류 표 (문단#155) | 160.6 (용지 793.7 초과 → 795.6) | **77.5** | 635.0 | 712.5 |
| 가점관련 증빙서류 표 (문단#157) | 77.5 | 77.5 (불변) | 633.0 | 710.5 |

두 표가 동일한 좌단(77.5)에 정렬되고 더 이상 용지를 넘지 않는다. `export-pdf` → `pdftoppm`
150dpi 렌더링으로 육안 확인.

### 무회귀

- `cargo test --lib -p rhwp`: 3889 passed / 0 failed(수정 전 `test_tac_leading_width_block_table_full_line`
  1건 실패 발견 → 탭 전용으로 조건을 좁혀 재통과 확인).
- `samples/복학원서.hwp`(제어 사례, PUA 필러 기반 leading): 수정 전/후 세 표 x 좌표
  (79.4 / 85.0 / 63.7) 완전 동일 — `export-render-tree` 비교 + 저장된 오라클 PDF
  (`samples/복학원서.pdf`)와 픽셀 대조로 무회귀 확인.
- `cargo fmt --check`, `cargo clippy --lib -- -D warnings` 통과.

## 남은 범위

인접 이슈 `#6298`(표 *뒤* 공백이 leading 으로 실리는 사례)은 같은 함수의 다른 갈래이며 이번
PR로 해결되지 않는다 — 별도 조사·수정이 필요하다(이슈에 명시).
