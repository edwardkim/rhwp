# 03 — render-diff --batch 와 geom_inventory.tsv

폴더를 전수 왕복 비교하고 TSV 를 남긴다. CI 아티팩트다.

```bash
mkdir -p rd_batch
cp samples/form-01.hwp samples/form-02.hwp rd_batch/
rhwp render-diff --batch rd_batch --via hwpx -o rd_out
```

레시피 06 실측:

```
[           PASS] max_disp=   0.00 struct=0 over=0      5ms  form-01.hwp
[           PASS] max_disp=   0.00 struct=0 over=0      4ms  form-02.hwp

TSV 저장: rd_out\geom_inventory.tsv
```

## TSV 컬럼

`sample status pages_a pages_b max_disp worst_page struct_pages over_pages elapsed_ms error struct_delta`

픽스처: `fixtures/tsv/geom_inventory_pass.tsv`.

- `max_disp` 는 소수점 3자리
- `worst_page` 없으면 `-`
- `struct_delta` 예: `Line:-4;RawSvg:-1` (음수=손실)
- `error` 는 LOAD_FAIL 행만

## 종료 코드

- 사람 모드: 하드 실패(OVER/STRUCT/PAGE/LOAD)가 하나라도 있으면 1
- `--json`: 로드 실패가 있으면 1 우선, 아니면 회귀 검출 3
- 폴더를 못 읽으면 2 (`오류: 폴더 읽기 실패`)
- 폴더에 .hwp/.hwpx 가 없으면 2

## JSON 배치

stdout 은 NDJSON. 한 파일 한 줄. `error` 키는 실패 행에만 있다.
TSV 저장 안내는 stderr 로 빠진다(stdout 순수성).

요약 줄("총 파일 2 / PASS 2")만 보고 통과시키지 않는다. 행별 status 로
게이트한다. [19_gate_recipes.md](19_gate_recipes.md).
