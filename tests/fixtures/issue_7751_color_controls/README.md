# #7751 한컴 저장 RGB 대조군

실제 한컴2024가 저장한 HWP5개다. 각 파일의 대응 Print PDF·SHA-256·저장 및
인쇄 job은 [독립 Print 기준](../../../pdf/issue7751/README.md)에 기록했다.
원본 편람의 피델리티를 대신하는 자료가 아니라 밝기·대비·회색조·흑백의 합성 색조 계약이다.

각 행은 같은11색이다: 검정, 회색50/100/128/170/206, 흰색,
`(80,100,120)`, 빨강, 초록, 파랑. 각10행의 조건은 다음과 같다.

| 입력 이름 | 효과·밝기·대비(B/C) |
| --- | --- |
| `hancom-color-probe-2024.hwp` | RealPic0/0,8/0,0/8,20/0,0/20,20/20,−20/0,0/−20; GrayScale0/50; BlackWhite10/0 |
| `hancom-color-probe-cross-2024.hwp` | RealPic의 대비−50/−20/20/50 각각 밝기−20/20; GrayScale0/50; BlackWhite50/0 |
| `hancom-color-probe-boundary-2024.hwp` | RealPic의 대비−100/100 각각 밝기−100/−20/0/20/100 |
| `hancom-blackwhite-zero-2024.hwp` | 기본 대조군과 같고 마지막 BlackWhite 행의 밝기만0 |
| `hancom-blackwhite-axis-probe-2024.hwp` | BlackWhite 밝기−100/−50/−20/0/10/50/100 각각 대비0; 밝기0 각각 대비−100/−50/100 |

수동 HWPX 틀은 독립 한컴 저장 전90% 미달이어서 정식 fixture로 등록하지 않았다.
이 HWP들은 한컴이 실제 저장한 동반 대조군으로, 그 틀의 통과를 주장하지 않는다.
검증 원본의 바이트를 수정하거나 저장 LineSeg를 수동 보정하지 않는다.

Native/fresh WASM은 입력 전체1쪽을 각각 비교했고5개 모두 raw100%였다.
브라우저 검사는 해당 HWP를 실제 WASM/Canvas에서 열고 source 그림의 bbox 안에서
각 패치의 RGB를 읽는다. 실제 문서의 화면 위치를 절대 px로 고정하는 검사가 아니다.
Native 공개 PNG 검사도 Print PDF에서 직접 읽은 RGB와 흑백 임계점을 보호한다.
