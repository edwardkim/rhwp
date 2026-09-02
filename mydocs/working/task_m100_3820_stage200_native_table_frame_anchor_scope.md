# Task M100 #3820 Stage 200 - native HWP 확장 표 프레임 앵커 범위

## 목적

Stage 199 뒤 전체 integration test에서 `samples/hwpspec.hwp`가 한컴 기준 178쪽 대신
176쪽으로 계산된 회귀를 해결한다. #3820의 HWP/HWPX 383쪽 Q&A와 저장 fragment 소유권은
그대로 유지한다.

## 관찰과 원인

- 최신 `upstream/devel`부터 Stage 198까지의 분리 검증에서 `8cf6b1ec`은 178쪽이고,
  `cf61bd4afda273324fff9d3bfb8df5bf74ad1b25`부터 176쪽이었다.
- 해당 커밋은 저장 bounds가 현재 흐름과 겹치기만 하면 tail fit으로 인정하도록 넓혔다.
- native HWP의 TAC 표 `LineSeg`는 표 프레임 전체를 하나의 줄 높이로 보관할 수 있다.
  따라서 수백 px 높이의 프레임 내부에 현재 cursor가 있기만 해도, 실제 표 앵커 줄과
  관계없이 현재 쪽 배치가 허용됐다. 이 과소 이월이 `hwpspec.hwp`에서 두 쪽을 합쳤다.
- HWPX의 저장 bounds는 physical fragment owner를 나타내므로 같은 축소를 적용하면
  `2025 행정업무운영 편람` p144 자동날인 표와 붙임 안내 block의 소유 쪽이 깨진다.

## 수정

- `saved_table_bounds_fit_at_flow_tail`을 추가했다. native HWP TAC 표의 tail fit은
  저장 표 프레임과 현재 흐름이 기존 공통 "같은 줄 앵커" 계약을 만족할 때만 허용한다.
- 단일 TAC 사전 fit과 실제 TAC 배치의 두 호출부에만 이 판정을 적용했다.
- HWPX와 일반 저장 tail은 종전 `saved_bounds_fit_at_flow_tail`을 유지한다. 이들은
  physical fragment와 RowBreak tail의 source 증거를 사용하므로 범위를 축소하지 않는다.
- 문서별 페이지 수, 파일명, tail allowance를 추가하지 않았다. native HWP/HWPX의
  저장 좌표 provenance와 표 프레임의 의미로만 분기한다.

## HWP 2020 PDF 증적

Stage 199에서 새 KoPub 글꼴 환경으로 MCP HWP 2020 변환을 다시 수행했다.

| 원본 | 증적 PDF | PDF 버전 | 페이지 |
| --- | --- | --- | --- |
| HWP | `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf` | 1.7 (`PrintToPDFEx`) | 383 |
| HWPX | `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2020.pdf` | 1.7 (`PrintToPDFEx`) | 383 |

두 증적은 `file`, `pdfinfo`로 MCP HWP 2020 산출물과 페이지 수를 확인했다. 로컬에는
`ubuntu-ted:/usr/local/share/fonts/hwp-convert-mcp-kopub`의 KoPub 글꼴 12종을
`~/Library/Fonts/hwp-convert-mcp-kopub`에 설치하고 font cache를 갱신했다.

## 검증

다음 명령을 2026-08-14에 실행했다.

```sh
cargo test --profile release-test --test issue_1086 --test issue_1035_alignment \
  --test issue_3930_hwpx_hwp_save_layout \
  --test issue_3820_rowbreak_rowspan_band \
  --test issue_3820_stored_reset_fragment_geometry \
  --test issue_3820_tac_caption_first_text_owner \
  --test issue_3820_body_top_table_border_clip -- --nocapture
```

결과는 19 passed, 0 failed다.

- `issue_1086`: 4 passed, `hwpspec.hwp` 178쪽 복구
- `issue_1035_alignment`: 4 passed, HWP3 sample16 HWP5 변환 64쪽 유지
- #3820 저장 reset, RowBreak/rowspan, TAC caption, body-top border: 8 passed
- `issue_3930_hwpx_hwp_save_layout`: 3 passed, HWP/HWPX Q&A 383쪽과 p144 자동날인
  owner 유지

## 결론

native HWP 확장 표 프레임을 텍스트 줄 앵커로 오인하던 공통 범위를 표 배치 두 지점으로
한정했다. HWPX physical fragment 계약과 #3820 저장 tail은 보존하면서, `hwpspec.hwp`의
178쪽 기준을 회복했다. 다음 단계에서 전체 lib 및 integration regression gate를 실행한다.
