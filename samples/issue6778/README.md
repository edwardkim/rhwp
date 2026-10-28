# #6778 정식 회귀 입력

[이슈 #6778](https://github.com/edwardkim/rhwp/issues/6778)에서 식별한 공개 문서
**동물복지 사육관리 지침서 4종 첫 발간**(농촌진흥청 보도자료)의 원본 바이트를 등록했다.
비공개 PC 경로 또는 환경 변수가 없어 검사를 건너뛰는 방식은 사용하지 않는다.

- 파일: [156757920-animal-welfare-husbandry-guidelines.hwp](156757920-animal-welfare-husbandry-guidelines.hwp)
- 원본 파일명: `156757920_22-2_동물복지사육관리지침서4종첫발간동물복지농가전용기술기준제시(축산원).hwp`
- SHA-256: `309494bf58da7c092eca2c6fe55918063d2f538820cbc3fc6f8c8196fcd3586a`
- 크기: 1478144바이트
- 저장 제품: `hancom-office-2024`, `13.0.0.564` — **오라클 판(한/글 2024)과 정확히 같다.**
- 기준 엔진: `2024` (저장 제품이 `hancom-office-2024`). 한/글 `PageCount` = 12쪽 = rhwp.

기준 PDF: [`pdf/156757920-animal-welfare-husbandry-guidelines-2024.pdf`](../../pdf/156757920-animal-welfare-husbandry-guidelines-2024.pdf)
— 696,994 bytes, SHA-256 `de6a57ee0d28cfb66acab4e477cc0b95e31a7939d3ca2e165de46fd5a487296e`.

| MCP 산출 증적 | 값 |
| --- | --- |
| job id | `f60133c0-ad7a-4d6a-9542-83fcb524af6e` |
| status | `succeeded` → `success` (phase `completed`) |
| 요청 engine / 응답 engine | `2024` / `2024` |
| `engine_profile` / `hancom_version` | `2024` / `13.0.0.3901` |
| backend | `hwp-managed-direct-dll-host` |
| blob 검증 | client·server byte 수·SHA-256 일치 (`verified`) |
| `pdf_page_count` | 12 (rhwp 와 동일) |

⚠ **이 문서는 `printMethod=4`(인쇄 모아찍기)다.** 한/글이 논리 쪽을 A4 시트의 부분
영역에 얹어 내보내므로 시트 절대 좌표는 rhwp 쪽 좌표와 1:1 이 아니다(정본 footer 는
모든 쪽에서 `820.2px`, 본문 최상단 `104.5px`). 세로 판정은 **본문 최상단 0.000,
footer 1.000 으로 정규화**해 비교한다.

## 정본 대조 — 1쪽 레인 (baseline, 정규화)

| 줄 | 한/글 2024 | rhwp 수정 전 |
| --- | --- | --- |
| 레인 첫 줄 `국립축산과학원이 …` | **0.692** | 0.909 |
| 레인 둘째 문단 `특히 축산 현장에서 …` | **0.853** | 1.006 |
| 레인을 벗어난 첫 항목 `설계 인증 기준을 …` | **0.950** | **1.102** |

한/글은 레인 세 줄과 그 뒤 항목을 **모두 footer 위(< 1.000)** 에 둔다. 수정 전 rhwp 는
레인을 0.909 에서야 시작해 마지막 항목이 **1.102 — 쪽번호 아래이자 용지 밖**으로 나간다.
`layout-anomaly` 의 용지밖 2·넘침 6 이 이 세 줄이다.

측정 명령: 정본은 `PyMuPDF` span `origin` (px = pt × 4/3), rhwp 는
`rhwp export-svg -p 0` 의 `<text y=…>` baseline.

⚠ 위 rhwp 열은 **수정 전(#6778 hunk 미적용)** 실측이다. 수정 후 좌표는 이 브랜치를
빌드해 같은 두 명령으로 재측정해야 한다 — 시험이 잠그는 값은 소스 머리말에 적힌
`pi=13 흐름 y = 668.1px`(저장 사다리·페이지네이터와 일치)다.

- 회귀 계약: Square(어울림) 표 옆 레인에서 **렌더 흐름도** host 줄만 전진한다.
  표 높이는 세로 배제 밴드로 잡고, **레인 술어를 만족하지 않는 첫 항목**에서 닫는다
  (그 항목은 표 바닥 아래에서 시작한다). 레인 술어는 저장 `column_start` 가 개체의
  **오른쪽 경계 밖**이고 `segment_width` 가 개체 폭의 절반 이상 좁을 때만 참이다 —
  `column_start == 0` 인 왼쪽 레인은 이 축이 다루지 않는다.

새 sample이 기존 코퍼스 래칫에 주는 영향은 통합 검증에서 실측한다. 알려지지 않은
회귀를 숨기려고 기존 문서 기준선을 올리거나 보안 탐지 신호를 포괄적으로 무시하지 않는다.
