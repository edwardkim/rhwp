# 18 — geom_inventory.tsv 스키마

`--batch -o <폴더>` 가 쓰는 파일 이름은 항상 `geom_inventory.tsv` 다.

탭 구분, 헤더 1행. 컬럼 11개:

1. sample
2. status
3. pages_a
4. pages_b
5. max_disp (%.3f)
6. worst_page (`-` 또는 정수)
7. struct_pages
8. over_pages
9. elapsed_ms
10. error
11. struct_delta

실측 PASS 2행은 레시피 06 과 `fixtures/tsv/geom_inventory_pass.tsv`.
혼합 카탈로그는 `fixtures/tsv/geom_inventory_mixed.tsv`.

게이트는 헤더를 건너뛰고 `$2` (status) 를 읽는다. PASS 와
WARN_TEXTRUN 만 통과로 둘지, STRUCT 를 경로 대조 큐로 보낼지는
소비자가 정한다. 스킬 기본은 STRUCT 를 즉시 실패로 접지 않는 것이다.
