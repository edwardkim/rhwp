# Task M100 #1407 — 2단계 완료 보고서 (증상 ② 귀속 + 본문 colPr 정정)

- 브랜치: `local/task1407`
- 작성일: 2026-06-14
- 수정 파일: `src/serializer/hwpx/section.rs`

## 1. 증상 ②(143E RT 페이지 수 1→2) 정체 확정

1단계(newNum 슬롯) 정정 후에도 143E RT 페이지 수는 1→2 그대로였다. `dump-pages`
대조로 정체를 확정:

- **원본**: 단 0(items=11) + 단 1(items=8) — **같은 페이지의 2개 컬럼**(2단)
- **RT**: 페이지1 단 0(items=11) + 페이지2 단 0(items=8) — **단일 단으로 페이지 넘김**

XML colPr 대조:
- 원본: `colCount="2" sameSz="1" sameGap="2268"` + colLine(구분선)
- RT: `colCount="1" sameSz="1" sameGap="0"`

근본 원인: **본문 섹션 colPr 이 IR(2단)이 아니라 템플릿 하드코딩 colCount=1 로 방출**.
본문(depth 0) ColumnDef 는 `render_runs` 인라인 슬롯에서 제외되고(#1379), 템플릿
`empty_section0.xml` 의 고정 colPr 만 남았다. **#1388(secPr 여백 템플릿 하드코딩)과
동형**이며 newNum 슬롯(증상 ①)과는 별개 결함. 작업지시자 판단으로 #1407 내 통합 처리.

(IR 자체는 2단 정보를 정확히 보존 — dump 문단 0.0: `[1] 단정의: 2단, 간격=8.0mm(2268)`.)

## 2. 수정 — 본문 colPr 템플릿 치환 (#1388 동형)

`write_section` 에서 첫 문단 IR 의 ColumnDef 를 찾아 템플릿 colPr anchor 를 IR 값으로
치환:

```rust
const TEMPLATE_BODY_COL_PR: &str =
    r#"<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" sameGap="0"/></hp:ctrl>"#;
// replace_page_pr 직후:
if let Some(Control::ColumnDef(cd)) = first_para_controls.find(ColumnDef) {
    out = out.replacen(TEMPLATE_BODY_COL_PR, &render_col_pr_ctrl(cd), 1);
}
```

`render_col_pr_ctrl`(#1379 셀 colPr 방출에 쓰던 함수)을 재사용 — colCount/sameGap/
colLine 구분선까지 IR 그대로 재현.

## 3. 검증 — 두 증상 모두 해소

- **증상 ②**: RT colPr `colCount="2" sameGap="2268"` 복원, **RT 페이지 수 1→1**
  (원본 동형). 143E ir-diff 전체 **차이 0**.
- **증상 ①**(1단계): 유지 — newNum 슬롯 정상.
- 단위 테스트:
  - `task1407_body_col_pr_reflects_ir_column_def`: 2단 IR → colCount=2/sameGap=2268,
    colCount=1 잔존 없음.
  - `task1407_single_column_doc_unaffected`: ColumnDef 없으면 colCount=1 유지(회귀 가드).
  - `task1407_field_end_not_stolen_by_newnum_slot`(1단계): 유지.
- **전수 배치**: PASS 53 / IR_DIFF 0 / SERIALIZE_FAIL 0 / ROUND2 0 (회귀 없음).
- `serializer::hwpx::section` 39→42(신규 3) passed.
- baseline 4 passed — **B=0 유지**.

## 4. 게이트 사각 — 페이지 수/colPr 비교 미포함

143E 는 ir-diff·baseline IR diff 가 0인데도 페이지 수가 달랐다 → colPr(단 정의) 손실은
IR 뼈대 비교에 안 잡혔다. ColumnDef 는 controls 비교 대상이나 본문 colPr 의 **방출 손실**
(IR→XML)은 재파싱 시 colCount=1 로 읽혀 양쪽 IR 이 같아 보이는 사각이었다. 본 수정으로
방출 자체가 정정돼 표면화는 종결. (게이트에 본문 colPr 방출 검증 추가 여부는 3단계 검토.)

## 5. 다음 단계

- 3단계: 트러블슈팅·매뉴얼 갱신 + CI급(`--profile release-test --tests` + fmt + clippy)
  + 최종 보고서.
