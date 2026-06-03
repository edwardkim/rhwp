# Task #301 Stage 1 완료 보고서: 회귀 검증 테스트 추가 (RED)

- **이슈**: #301
- **브랜치**: `local/task301`
- **단계**: 1 / 4

## 작업 내용

### 신규 파일
- `tests/issue_301.rs`: z-table 셀 수식 1회 출현 검증 통합 테스트 추가

### 검증 로직
페이지 12의 SVG 출력에서 z-table 값 출현 횟수를 카운트:

| 값 | 기대 횟수 | 사유 |
|----|----------|------|
| `0.1915` | 1 | z-table 1회 |
| `0.3413` | 1 | z-table 1회 |
| `0.4332` | 1 | z-table 1회 |
| `0.4772` | 2 | z-table 1 + 동일 페이지 다른 본문 1 |

`0.4772`는 #29 본문 ("이용하여 구한 것이 0.4772일 때") + z-table에 모두 등장하므로 2회가 정상.

## RED 확인

```
$ cargo test --test issue_301 --release
running 1 test
test z_table_equations_rendered_once ... FAILED

assertion `left == right` failed: z-table value "0.1915" expected 1 occurrence, found 2 (이중 렌더링 회귀)
  left: 2
 right: 1
```

수정 전 상태에서 의도대로 실패함을 확인 → 이중 렌더링 버그가 본 테스트로 정확히 검출됨.

## 다음 단계

Stage 2: `src/renderer/layout/table_layout.rs:1602` Equation 분기에 `tree.get_inline_shape_position()` 가드 추가.

## 승인 요청

본 단계 결과에 대한 작업지시자 승인 요청. 승인 시 Stage 2 (본 수정) 진행.
