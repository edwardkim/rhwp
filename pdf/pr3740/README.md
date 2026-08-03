# PR #3740 / Issue #3738 시각 기준 증적

이 디렉터리의 PDF는 개인정보를 제거한 같은 정책연구용역 문서를 한컴오피스 2020으로
각각 HWP와 HWPX에서 변환한 기준 출력이다. 두 PDF 모두 215쪽 A4이며 Stage 1 visual
sweep의 page-local 기준으로 사용한다.

| 입력 | 기준 PDF | SHA-256 |
| --- | --- | --- |
| `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` | `hwp/…-2020.pdf` | `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a` |
| `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx` | `hwpx/…-2020.pdf` | `b5882db7b4583f9f26eb9598132d7d1188d023120edb3d9039658914bca6b384` |

입력 SHA-256은 각각 HWP `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`,
HWPX `8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`다.

상세 판정과 review PNG는
[`mydocs/working/task_m100_3738_stage1_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage1_visual_sweep.md)와
[`mydocs/working/task_m100_3738_stage2_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage2_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage3_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage3_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage4_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage4_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage5_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage5_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage6_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage6_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage7_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage7_visual_sweep.md),
[`mydocs/working/task_m100_3738_stage8_visual_sweep.md`](../../mydocs/working/task_m100_3738_stage8_visual_sweep.md)를
따른다. 이 파일들은 일반 Git 추적 대상이며, 현재 `.gitattributes`의 LFS 범위(`pdf-large/**/*.pdf`)에는
들지 않는다.
