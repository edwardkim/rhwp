# #7028 Stage 1 — 제목행 반복과 대각선 누락 원인

- 이슈: https://github.com/edwardkim/rhwp/issues/7028
- 날짜: 2026-09-11 (KST)
- 수행계획: [task_m100_7028.md](../plans/task_m100_7028.md), 메인테이너 승인 완료.
- 브랜치: `task_m100_7028`; 수행계획 승인 커밋 `e235f8b57`.
- 조사 소스: `b59323de0448df0a42bcb9cdd12cb80d51fe83d2`. 이후 변경은 문서뿐이다.
- 판정: 원인·수정 경계 조사 완료. 제품 코드 미변경, 구현계획 승인 대기.

## 1. 결론

제목행을 선택하거나 다음 페이지에 배치하는 기능의 누락이 아니다. **제목행은 이미 6쪽 모두
배치되지만, 분할 표 셀 경로에서 대각선 RenderNode를 생성하지 않는다.** 첫 페이지도
`partialTable`을 사용하기 때문에 빠진다. 실제로 잘린 셀과 온전한 반복 제목 셀은 다르다.

따라서 페이지 분할이나 제목행 높이 계산을 바꾸지 않고, 온전한 셀의 대각선 paint를 기존
공통 생성기로 연결하는 것을 권고한다. 실제 잘린 셀과 분할 cell zone의 대각선 정책은 이번
샘플로 검증되지 않으므로 조각마다 임의의 새 대각선을 그리는 변경은 승인 범위로 제안하지 않는다.

## 2. 원본과 한컴 PDF

- HWP: `samples/task2146/21761835_jeonjik_exemption_table.hwp`.
- SHA-256: `d8e7e38b206f6e53b7d2d260115396f8a0dd94bd8b712bbc4428dc11bf66e5dc`.
- PDF: `pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`.
- PDF SHA-256: `99d150b26805a29f3d3153528638f363e33f882d0e38ad11edb374baccdac127`.
- PDF 메타데이터: Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`,
  생성 2026-08-30, 6쪽. 도입 커밋 `630a598672`(Oracle 기준 PDF 재산출본 추가).
  파일명 `2020`을 생성기 버전의 증거로 사용하지 않는다.
- 다른 `21761835_jeonjik_exemption_table-2020.pdf`는 cairo producer이며 이번 기준에서 제외했다.
- 문서 제목·개정일·표 내용·첫 제목행과 2·6쪽 반복 제목행을 직접 대조했다.
  기존 PDF를 이 결함의 기준으로 재사용한다. 현재 한컴 실행환경으로 재생성했다는 뜻은 아니다.

CFB의 압축된 Section0·DocInfo 레코드를 `olefile` + raw-deflate 해제로 확인했다:

| 항목 | 실측 |
| --- | --- |
| TABLE | attr `0x05000006`, RowBreak, repeat_header=true, 78행×5열 |
| 제목 셀 | r0c0~r0c4의 LIST_HEADER header flag `0x0004`, rowspan=1 |
| 대각선 지정 셀 | r0c0 한 개, BF=13, attr `0x0040` |
| 대각선 | backslash bits=2, type=1(실선), width index=1, 검정 |
| cell zone 수 | 0 |

한컴 PDF 1·2·6쪽에서 r0c0의 좌상단→우하단 선이 보이며 후속 쪽에도 같은 제목행이 반복된다.
대각선 속성이 원본에서 사라지거나 none으로 파싱됐다는 가설은 근거와 맞지 않는다.

## 3. 기존 제목행 규칙과 도입 이유

`src/model/table.rs:773`의 `leading_header_rows()`는 다음을 계산한다.

1. `is_header` 셀이 덮는 행을 rowspan까지 반영한다.
2. 행 0부터 연속한 최대 제목행 블록만 선택한다.
3. 중간·하단에 흩어진 header 셀은 반복 대상으로 삼지 않는다.

`src/renderer/typeset.rs:26957`은 continuation·repeat_header·has_header_cells·다행 표 조건에서
cursor_row 이전의 해당 행 높이와 셀 간격을 예약한다. `table_partial.rs:3663,3956`은 같은
함수로 제목행 높이를 준비하고, start_row 이전 제목행을 본문 조각 앞에 붙인다.
`render_rows` 순서로 `render_row_y`를 계산하므로 원본 행 번호와 표시 행 번호는 다르다.

[#1716 결과보고서](../report/archives/task_m100_1716_report.md)에 따르면 이전 구현은 cursor
앞의 모든 header 행을 반복해 본문 곳곳에 header 표시가 있는 문서를 173쪽으로 팽창시켰다.
상단 연속 블록으로 제한해 53쪽으로 줄였으며 당시 한컴 52쪽과의 잔여는 별도로 남겼다.
이번에는 이 정의와 높이 회계를 변경하지 않는다. 과거 계획의 임시 시그니처 대신 현재
`leading_header_rows(&self)` 구현을 기준으로 삼는다.

## 4. 누락 지점과 변경 위험

| 경로 | 현재 동작 | 이번 판단 |
| --- | --- | --- |
| `parser/doc_info.rs`, `style_resolver.rs` | BF attr와 diagonal style 보존 | 변경 불필요 |
| 일반 표 `table_layout.rs:8237` 이후 | cell zone 중복 여부를 확인하고 셀 내용 위에 `render_cell_diagonal` 방출 | 재사용 기준 |
| 분할 표 가로쓰기 `table_partial.rs:3258` 이후 | 네 방향 edge 수집 후 Cell 추가, 대각선 호출 없음 | 누락 연결 대상 |
| 분할 표 세로쓰기 `table_partial.rs:1541` 이후 | edge 수집 후 Cell 추가하고 continue | 같은 누락; 가로쓰기 말단만 수정하면 불충분 |
| `border_rendering.rs:1397` | slash/backslash·꺾은선·중심선·선 없음 해석 | 새 선 해석기를 만들지 않고 재사용 |

`is_repeated_header_cell`, `render_rows`, `is_in_split_row`, `height_override_clip`, rowspan의
fragment 경계가 이미 있다. 전체 셀의 행 범위가 표시되지 않거나 cut/높이 override가 영향을
주는 셀은 온전한 셀로 볼 수 없다. `clip` 한 비트만으로 판정하면 `TablePageBreak::None`의
rowspan 경계나 내용 보호용 bbox 확장을 혼동할 수 있으므로 원본 행 범위와 분할 정보를 함께 쓴다.

대각선 좌표는 `cell_x/y/w/h`의 표 격자 기준을 사용한다. 내용 넘침 보호로 확장된
`cell_node.bbox`나 텍스트 inner_area를 사용하면 선 끝점이 테두리 밖으로 이동할 수 있다.

일반 표는 cell zone 전체 사선과 시작 셀의 중복 BF를 구분하고 나중에 적용된 개별 셀
사선·중심선을 보호한다(#1623/#1633). 반면 분할 표에는 zone 렌더 경로 자체도 없다.
이 샘플에는 zone이 없으므로 이를 새로 일반화하는 것은 별도 근거가 필요한 범위다.

## 5. 실제 실행 결과

기존 baseline 바이너리를 재사용했다. 코드가 같으므로 재빌드하지 않았다.

- 실행 파일: `/home/edward/mygithub/rhwp-shared-review-target/release-test/rhwp`.
- SHA-256: `b32c9bf5db106f89a4ca823521beb62bfb39ce18c951074ee8d7ea65d9d0112c`.
- 기준 PDF / 전체 SVG / 전체 RenderTree: **6 / 6 / 6쪽**, 요청 6쪽 모두 완료.
- 매 페이지 row0 셀 5개 존재. 전체 30개 제목 셀 인스턴스.
- r0c0 bbox: 1쪽 `(72.0,225.3,56.2,52.4)`, 2~6쪽 `(72.0,113.4,56.2,52.4)` px.
- 6쪽 모두 대각선 크기의 Line bbox 0개. 첫 페이지 SVG 직접 검사에서도 대각선 없음.
- 1·2·6쪽 표준 비교 시트를 열어 제목행의 대각선 누락을 재확인했다.

재현 명령(저장소 루트, 페이지 범위는 0-based):

```bash
RHWP_BIN=/home/edward/mygithub/rhwp-shared-review-target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 5 \
  --source samples/task2146/21761835_jeonjik_exemption_table.hwp \
  --reference-pdf pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf \
  --label issue7028-before --reference-grade '기존 Hancom PDF; metadata Hwp 2022' \
  --text-only --export-all-svg --layout-ledger --out-dir output/7028/stage1/fidelity
```

pixel 비교는 같은 direct pair로 `--text-only --export-all-svg --layout-ledger`를 빼고
`--out-dir output/7028/stage1/visual`을 사용했다. 첫 시도는 Chrome PATH 자동 탐색 실패(exit 2),
설치된 `CHROME_BIN=/home/edward/.cache/puppeteer/chrome/linux-146.0.7680.31/chrome-linux64/chrome`
명시 후 6쪽 성공했다. 환경 실패를 렌더 회귀로 분류하지 않는다.

| 물리 쪽 | 비교 시트 | pixel diff 보조값 |
| --- | --- | ---: |
| 1 | `output/7028/stage1/visual/cmp-p000.png` | 14.03% |
| 2 | `output/7028/stage1/visual/cmp-p001.png` | 16.56% |
| 6 | `output/7028/stage1/visual/cmp-p005.png` | 14.84% |

이 수치는 fidelity 도구의 pixel diff이며 visual_sweep의 잉크 일치율이 아니다. 합격선으로
사용하지 않는다. 제목 라벨의 상하 배치·글꼴·본문 행높이도 수정 전부터 PDF와 다르다.
text-only 원장도 1·2쪽 문자 멀티셋은 동일하지만 3~6쪽은 차이가 있다. 따라서 이 PDF와
문서 전체가 이미 일치한다거나, 발견한 모든 차이가 대각선 하나 때문이라고 주장하지 않는다.
기존 차이는 수정 전후 기하·텍스트 비교로 악화 여부를 구분한다.

## 6. 기존 검증 재사용과 미검증 항목

- `src/model/table/tests.rs:1213` 이후 4개: 흩어진 제목행, 연속 다중 제목행, rowspan 제목행,
  제목행 없음/전체 제목행. 기존 source test를 수정하지 않고 회귀 실행 대상으로 사용한다.
- [#2146 결과보고서](../report/archives/task_m100_2146_report.md)는 r0c0 사선과 c1 Fixed 줄간격
  모순을 구분해 높이 52.4px를 보호했다. 기존 실물 테스트는 선 존재를 검사하지 않는다.
- `tests/issue_1623_cellzone_diagonal.rs`: zone 사선, 중복 억제, 개별 셀 우선 적용·중심선 보호.
- 이번 Stage 1에는 Rust 변경·Cargo 회귀·WASM 빌드·최종 메인테이너 시각 판정이 없다.
  기존 성공 기록을 이번 수정의 통과 결과로 사용하지 않는다.

## 7. 승인 요청

[구현계획](../plans/task_m100_7028_impl.md)의 **온전한 셀 및 반복 제목 셀 연결**을 권고한다.
실제 잘린 셀과 active cell zone 영향 셀은 기존 출력 유지 대상으로 명시한다. 이 제한은 샘플명이나
행 번호 예외가 아니라 아직 검증되지 않은 다른 기하 계약을 바꾸지 않는 경계다.
이 경계를 포함한 구현계획 승인을 받은 뒤 Stage 2로 진입한다.

## 용어

- BF (Border Fill): 테두리·배경 속성. 셀의 BF 번호가 문서 스타일을 참조한다.
- IR (Intermediate Representation): 포맷별 파싱 결과를 담는 공통 문서 구조.
- fragment: 여러 쪽으로 나뉜 표/셀의 한 페이지 표시 조각.
- cut: 셀 내용 중 해당 조각에 포함할 범위를 나타내는 분할 정보.
- rowspan: 한 셀이 세로로 차지하는 원본 행 수.
- cell zone: 여러 셀 영역에 공동 적용되는 테두리·배경 속성. 개별 셀 속성과 구분된다.
- paint: 확정된 기하와 스타일로 실제 그릴 노드를 생성·출력하는 과정.
