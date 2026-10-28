# Task M100 #3930 Stage 1 - 저장 경로 조판 입력 차이 계측

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 기준: `upstream/devel` `d3fb9de7c`
- 기록일: 2026-08-04 KST
- 상태: 완료

## 범위

이 이슈는 `samples/2025 행정업무운영 편람(최종).hwpx`와 rhwp 저장 HWP의 조판 입력 차이를
다룬다. #3931의 declared 선이월·행높이 측정 축과 달리, 동일 문서를 저장한 뒤 복원한 형식이
원본과 다르게 배치되는 문제다.

1. HWPX에서만 p144의 `⇒ 기안문에 작성한 붙임 문서를 첨부...` 블록이 다음 쪽으로 이월되어
   HWPX 387쪽과 저장 HWP 386쪽이 갈린다.
2. p30 부근에서 HWPX는 책 제목 머리말, 저장 HWP는 장 제목 머리말을 선택한다.

## 조사 계획

1. 두 sample의 file fingerprint, section/page count, p144 anchor 앞뒤 문단·표·LINE_SEG를 추출해
   최초 조판 입력 차이를 좌표화한다.
2. 원본 HWPX를 현재 `devel` serializer로 HWP에 저장하고 다시 열어, 제공 HWP와 동일한 차이가
   재현되는지 분리한다. 원본 fixture 자체의 차이와 serializer 결함을 혼동하지 않는다.
3. p30 전후 페이지의 active header/footer 후보와 section 시작 조건을 HWPX·HWP에서 비교한다.
4. 원인이 serializer 복원 누락이면 최소 IR/변환 보정과 focused regression을 추가한다. 조판 계산
   자체의 차이면 #3931 등 기존 축으로 연결하고 이 이슈의 수용 범위를 재분류한다.

## 수용 기준

- 원본 HWPX와 rhwp 저장 HWP가 같은 본문 anchor의 쪽 경계를 유지한다.
- 같은 구역/쪽에서 HWPX와 저장 HWP가 같은 머리말을 선택한다.
- 새 회귀는 실제 편람 fixture 또는 충분히 작은 구조 fixture로 저장 전후를 고정한다.
- parser/serializer 변경 범위에 맞는 focused test, release-test, fmt, clippy, 저장 후 재열기
  검증을 완료한다.

## 조사 결과

### p144 표 이월

- 원본 HWPX는 387쪽, 기존 rhwp 저장 HWP는 386쪽으로 재현됐다.
- 원본 p144(0-based 143)의 표는 `treatAsChar=1`, `flowWithText=0`이며,
  `partialTable`의 `endCut=[21]`로 끝난다. 다음 p145는 같은 표의 `startRow=2`,
  `startCut=[21]` 연속 쪽이다.
- HWP5 CTRL_HEADER를 다시 읽을 때 `table.attr` bit 0이 `treat_as_char`로 채워진다.
  기존 조판기는 HWPX 출처 저장본에서도 이 비트만 보고 글자처럼 취급(TAC)하여 표를 한 쪽에
  통째로 배치했다. `flow_with_text`가 거짓인 HWPX 표는 TAC 흐름이 아니므로, HWPX 저장
  계보에서는 두 속성이 모두 참일 때만 TAC으로 판단하도록 보정했다.

### p30 바탕쪽

- 원본 p30은 짝수 쪽으로, 이전 구역의 `Even` 바탕쪽인 `2025 행정업무운영 편람`을
  상속한다. 현재 구역에는 `Odd` 바탕쪽인 `제2장. 공문서 관리`만 선언돼 있다.
- HWPX는 `Both`/`Odd`/`Even` 선언을 희소하게 두고 앞 구역 바탕쪽을 상속할 수 있지만,
  HWP5 `LIST_HEADER`의 저장 순서는 적용 범위를 나타낸다. 기존 저장은 현재 구역의 단일
  `Odd` 항목을 저장해 재열기에서 `Both`로 해석했고, 짝수 p30에도 장 제목이 표시됐다.
- HWPX에서 HWP로 저장할 때 현재 조판의 유효 짝수 바탕쪽을 `Both`, 유효 홀수 바탕쪽을
  `Odd` 슬롯으로 명시화했다. 따라서 HWP5에서도 짝수 쪽은 책 제목을 유지하고 홀수 쪽은
  현재 구역 장 제목으로 덮어쓴다.

## 구현

- `src/renderer/typeset.rs`: HWPX 저장 계보의 TAC 표 흐름 판단을
  `treat_as_char && flow_with_text`로 분리했다.
- `src/document_core/converters/hwpx_master_page_slots.rs`: 희소 HWPX 바탕쪽과
  상속 결과를 HWP5 `Both`/`Odd` 슬롯으로 정규화하는 공통 모듈 및 단위 회귀를 추가했다.
- `src/document_core/converters/hwpx_to_hwp.rs`: HWPX 출처 어댑터에만 위 정규화를 연결하고
  수행 횟수를 `AdapterReport`에 기록했다. HWP3/HWP 출처는 변경하지 않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`: 실제 편람 fixture를 HWP로 저장·재열기해
  387쪽 유지와 구역 2의 `Both` 책 제목/`Odd` 장 제목 슬롯을 고정했다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout -- --nocapture` | 통과 (1 passed) |
| `cargo test --profile release-test --test hwpx_to_hwp_adapter -- --nocapture` | 통과 (50 passed, 15 ignored) |
| `rhwp convert 'samples/2025 행정업무운영 편람(최종).hwpx' ... --verify-pages` | 저장·재열기 387쪽, 검증 통과 |
| 저장본 `dump-pages` | p144 `endCut=[21]`, p145 `startRow=2/startCut=[21]` 연속 확인 |
| 저장본 p30 `export-render-tree` | `2025 행정업무운영 편람` 바탕쪽 텍스트 확인 |
| `cargo clippy --lib -- -D warnings` | 통과 (경고 없음) |
| 최종 `cargo test --profile release-test --lib hwpx_master_page_slots` | 통과 (1 passed) |
| 최종 `cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout` | 통과 (1 passed) |
| 최종 `cargo test --profile release-test --test hwpx_to_hwp_adapter` | 통과 (50 passed, 15 ignored) |
| `rustfmt --check`, `git diff --check` | 통과 |

위 실제 파일 검증 산출물은 Git 비추적 `output/task3930-stage1/`에 보관한다. 전체 렌더 트리의
도형 구조 차이는 HWPX→HWP 일반 저장 손실 범위이며, 본 이슈의 쪽 경계와 바탕쪽 선택 수용 기준에는
영향이 없다.

## 추가 HWP 2020 MCP 검증 계획

rhwp 저장·재열기 검증만으로는 한컴 Office 2020의 실제 변환 결과를 보장할 수 없으므로, 인증된
원격 MCP 서버에 같은 편람의 HWP/HWPX 원본을 전송해 다음 네 경로를 추가 확인한다. 입력·출력은
검증 호스트의 로컬 파일이며, server 내부 경로나 인증 값은 기록하지 않는다.

1. `HWPX -> HWP`: 저장본을 rhwp로 재열기하여 387쪽과 p144 표 이월을 다시 확인한다.
2. `HWP -> HWPX`: ZIP/HWPX 구조와 rhwp 재열기를 확인한다.
3. `HWPX -> PDF`, `HWP -> PDF`: `pdfinfo`, `pdftotext` 및 1쪽 raster를 확인한다.
4. 장문서이므로 MCP 비동기 `start/status/download`를 사용하고, 각 job 완료 뒤 결과를 즉시
   내려받아 server 임시 output 정리를 허용한다.

`dump-pages`의 출력은 stdout JSON 계약이다. MCP HWP 저장본을 처음 확인할 때 `-o`를 사용한
명령은 지원하지 않는 옵션으로 종료했으며, 변환/download 자체와 무관한 CLI 호출 오류였다.
같은 파일에 대해 바로 `dump-pages <file> --json > <result>.json`으로 재실행해 아래 결과를 얻었다.

### 결과: HWPX -> HWP

- MCP 비동기 job은 56초에 성공했고, client download 단계의 SHA-256 검증도 통과했다.
  산출물은 10,595,328 byte HWP5 파일이다.
- 다만 이 Hancom Office 2020 저장본을 rhwp로 재조판하면 393쪽이며, 원본 HWPX 387쪽과
  일치하지 않는다. p144 표도 한 쪽에 완전히 들어간다.
- 따라서 이 결과는 MCP 서버의 `HWPX -> HWP` 기능과 응답 전달이 정상임을 보이는 추가 증적일
  뿐, HWPX 원본의 저장 조판 입력을 보존해야 하는 #3930 수용 기준의 oracle로 사용하지 않는다.
  #3930의 수용 검증은 위의 rhwp 저장·재열기 387쪽 fixture 회귀로 고정한다.

### 결과: HWP -> HWPX

- MCP 비동기 job은 39초에 성공했고, client download의 SHA-256 검증을 통과했다.
  산출물은 9,841,182 byte HWPX다.
- `unzip -t`로 ZIP 무결성을 확인하고 rhwp로 재열기한 결과는 387쪽이다.

### 결과: HWPX -> PDF

- MCP 비동기 job은 624초에 성공했고, 19,657,247 byte PDF를 SHA-256 검증 후 내려받았다.
- 서버는 `PrintToPDFEx`, `PrintMethod=0`(1-up)로 출력했으며 편집기 쪽수와 PDF 쪽수가
  모두 383으로 일치한다고 보고했다. PDF 본문 text 검증도 통과했다
  (`source_text_characters=232420`, `pdf_text_characters=206191`).
- 로컬 `pdfinfo`도 383쪽을 확인했다. 1쪽을 144dpi PNG로 raster한 결과는 표지의
  `2025 행정업무운영 편람`과 도형을 정상 표시했다.

### 결과: HWP -> PDF

- MCP 비동기 job은 610초에 성공했고, 19,609,732 byte PDF를 SHA-256 검증 후 내려받았다.
- 서버는 동일하게 `PrintToPDFEx` 1-up을 사용했고 편집기/PDF 쪽수 383의 일치를 확인했다.
  로컬 `pdfinfo`도 383쪽, `pdftotext`는 전체 274,398자를 추출했다.
- 이 HWP 원본의 PDF 1·2쪽 raster는 비어 있고 3쪽부터 내용이 표시된다. 원본 HWP의
  페이지 항목도 1·2쪽은 빈 문단 및 무텍스트 항목이며, 3쪽부터 본문/도형이 있다. 따라서
  전체 빈 PDF가 아닌 원본 HWP의 선행 빈 쪽이 보존된 결과로 기록한다.

## 최종 판정

- #3930의 직접 수용 기준인 rhwp `HWPX -> HWP -> 재열기`는 387쪽, p144/p145 표 연속,
  p30의 책 제목 바탕쪽으로 통과했다.
- HWP 2020 MCP의 `HWPX -> HWP`, `HWP -> HWPX`, `HWPX -> PDF`, `HWP -> PDF` 네 경로는
  모두 server validation과 client download SHA-256 검증을 통과했다.
- Hancom 저장본의 재조판 쪽수는 rhwp 저장 계약의 oracle가 아니므로, HWPX→HWP 한컴
  저장본이 rhwp에서 393쪽으로 읽힌 사실은 별도 호환성 관찰값으로만 남긴다.
