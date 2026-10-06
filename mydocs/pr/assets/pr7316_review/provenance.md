# PR #7316 Visual Sweep 증적

| 항목 | 값 |
| --- | --- |
| 검토 head | `878ba0d8035b6466c53ba3ef6a0598904d7b7060` |
| Native + provenance 보정 head | `4c663976b` |
| 입력 HWP | `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` |
| 입력 SHA-256 | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| 기준 PDF | `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` |
| 기준 PDF SHA-256 | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |
| 비교 쪽 | 13·14쪽, 96 dpi |
| HWP 저장 정보 | `hancom-office-2010` / `8.5.8.1677`; 기준 변환 프로필 `engine 2020` |

Native와 fresh WASM 각각의 review·overlay를 보관한다. 두 경로 모두 Visual Sweep complete,
구조 flag 0건이었다. 이 증적은 14쪽 경계 복원과 기존 그림 좌표 계약의 유지 범위만 보인다.
표·그림·글꼴의 기존 raster 잔차가 남아 있어 전체 PDF 시각 일치를 주장하지 않는다.
