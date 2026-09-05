# 예제 — 배치 전원 PASS

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp render-diff --batch rd_batch --via hwpx -o rd_out
```

## 실측·카탈로그 출력

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

## 읽는 법

TSV 두 행 모두 PASS. 요약 줄과 행이 일치하는지 본다.

관련: `references/` 같은 번호 장, `fixtures/transcripts/batch_pass.txt`.
