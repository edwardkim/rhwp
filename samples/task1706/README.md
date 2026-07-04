# Task #1706 검증용 한글 문서 (표 직후 빈 문단 누락 수정)

블록 표(tac/TopAndBottom) 직후의 빈 문단이 페이지 배치에서 누락되던(`rhwp_pNone`) 회귀 방지용
표본. 출처: 공공 공개 문서.

| 파일 | 원본 ID | 구조 | 기대 동작(수정 후) |
|------|---------|------|------------------|
| `empty_para_between_tac_tables.hwp` | 2957879 | tac 표 + **빈 문단(pi=3)** + tac 표 | `dump-pages` 에 `FullParagraph pi=3 "(빈)"` 가 페이지 1에 배치. 한글과 MATCH. |
| `empty_para_before_pagebreak.hwpx` | 36397647 | tac 표 + 빈 문단 + 표(**[쪽나누기]**) | 빈 문단(pi=3)이 드롭되지 않고 현재 페이지에 흡수. 한글과 MATCH. |

## 재현

```bash
cargo build --release
rhwp dump-pages samples/task1706/empty_para_between_tac_tables.hwp | grep -E "페이지|pi=3"
# 수정 전: pi=3 누락(rhwp_pNone)  /  수정 후: FullParagraph pi=3 "(빈)" 가 페이지 1에 표시
```

수정: `typeset.rs` 의 빈 문단 fit-실패 분기 2곳에서 `continue`(드롭) 대신 현재 페이지에 0-높이로
흡수 기록(`hide_empty_line` 동일 시멘틱). 페이지를 advance 하지 않아 단독 빈 페이지 회귀 없음.
