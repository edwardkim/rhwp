# 16 — 재현 트레이스

레시피 06 실측과 상태별 카탈로그. 원문은 `fixtures/traces/`.

## T01 — PASS (exit 0)

`render-diff samples/form-01.hwp --via hwpx`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T02 — STRUCT_MISMATCH (exit 1)

`render-diff samples/form-01.hwp batch_out/0001.hwp`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T03 — STRUCT_MISMATCH (exit 1)

`render-diff samples/form-01.hwp batch_out/0001.hwp --max-disp 0.05`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 0.05px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T04 — PASS (exit 0)

`render-diff batch_out/0001.hwp batch_out/0002.hwp`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T05 — PASS (exit 0)

`render-diff batch_out/0001.hwp batch_out/0001.hwp`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T06 — PASS (exit 0)

`render-diff --batch rd_batch --via hwpx -o rd_out`

```
[           PASS] max_disp=   0.00 struct=0 over=0      5ms  form-01.hwp
[           PASS] max_disp=   0.00 struct=0 over=0      4ms  form-02.hwp

TSV 저장: rd_out\geom_inventory.tsv

=== render-diff 요약 ===
  총 파일         : 2
  PASS            : 2
  WARN_TEXTRUN    : 0
  OVER            : 0
  STRUCT_MISMATCH : 0
  PAGE_MISMATCH   : 0
  LOAD_FAIL       : 0
  전체 최대 변위  : 0.00 px
```

## T07 — PASS (exit 0)

`render-diff <A> --via hwpx`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T08 — STRUCT_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T09 — OVER (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=3 B=3
최대 변위: 279.00 px (page 1)
임계 초과 페이지: 1 / 구조 불일치 페이지: 0 (임계 1.00px)
status: OVER
```

## T10 — PAGE_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=2 B=3
⚠ 페이지 수 불일치 — 시각 회귀 강신호
최대 변위: 0.00 px (page -)
status: PAGE_MISMATCH
```

## T11 — LOAD_FAIL (exit 1)

`render-diff <A> <B>`

```
오류: 파일 읽기 실패 samples/no-such.hwp
```

## T12 — WARN_TEXTRUN (exit 0)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 0.40 px (page 0)
임계 초과 페이지: 0 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max=   0.40 mean=  0.05 nodes=40/41  [STRUCT:TextRun±1]
status: WARN_TEXTRUN
```

## T13 — PASS (exit 0)

`render-diff <A> --via hwpx`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T14 — STRUCT_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T15 — OVER (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=3 B=3
최대 변위: 279.00 px (page 1)
임계 초과 페이지: 1 / 구조 불일치 페이지: 0 (임계 1.00px)
status: OVER
```

## T16 — PAGE_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=2 B=3
⚠ 페이지 수 불일치 — 시각 회귀 강신호
최대 변위: 0.00 px (page -)
status: PAGE_MISMATCH
```

## T17 — LOAD_FAIL (exit 1)

`render-diff <A> <B>`

```
오류: 파일 읽기 실패 samples/no-such.hwp
```

## T18 — WARN_TEXTRUN (exit 0)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 0.40 px (page 0)
임계 초과 페이지: 0 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max=   0.40 mean=  0.05 nodes=40/41  [STRUCT:TextRun±1]
status: WARN_TEXTRUN
```

## T19 — PASS (exit 0)

`render-diff <A> --via hwpx`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T20 — STRUCT_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T21 — OVER (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=3 B=3
최대 변위: 279.00 px (page 1)
임계 초과 페이지: 1 / 구조 불일치 페이지: 0 (임계 1.00px)
status: OVER
```

## T22 — PAGE_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=2 B=3
⚠ 페이지 수 불일치 — 시각 회귀 강신호
최대 변위: 0.00 px (page -)
status: PAGE_MISMATCH
```

## T23 — LOAD_FAIL (exit 1)

`render-diff <A> <B>`

```
오류: 파일 읽기 실패 samples/no-such.hwp
```

## T24 — WARN_TEXTRUN (exit 0)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 0.40 px (page 0)
임계 초과 페이지: 0 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max=   0.40 mean=  0.05 nodes=40/41  [STRUCT:TextRun±1]
status: WARN_TEXTRUN
```

## T25 — PASS (exit 0)

`render-diff <A> --via hwpx`

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## T26 — STRUCT_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## T27 — OVER (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=3 B=3
최대 변위: 279.00 px (page 1)
임계 초과 페이지: 1 / 구조 불일치 페이지: 0 (임계 1.00px)
status: OVER
```

## T28 — PAGE_MISMATCH (exit 1)

`render-diff <A> <B>`

```
페이지 수: A=2 B=3
⚠ 페이지 수 불일치 — 시각 회귀 강신호
최대 변위: 0.00 px (page -)
status: PAGE_MISMATCH
```

## T29 — LOAD_FAIL (exit 1)

`render-diff <A> <B>`

```
오류: 파일 읽기 실패 samples/no-such.hwp
```

## T30 — WARN_TEXTRUN (exit 0)

`render-diff <A> <B>`

```
페이지 수: A=1 B=1
최대 변위: 0.40 px (page 0)
임계 초과 페이지: 0 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max=   0.40 mean=  0.05 nodes=40/41  [STRUCT:TextRun±1]
status: WARN_TEXTRUN
```
