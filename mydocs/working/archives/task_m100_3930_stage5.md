# Task M100 #3930 Stage 5 - HWP5 CharShape inactive sentinel canonicalization

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `928cb282e` (`feat(diagnostic): CharShape sentinel probe 추가`)
- 기록일: 2026-08-05 KST
- 상태: 완료 (전역 canonicalization 보류)

## 목표

Stage 4에서 Hancom direct HWP의 inactive CharShape sentinel을 적용하면 7쪽이 단조 개선되고
4쪽이 기준 PDF와 완전히 일치하는 것을 확인했다. HWPX/IR 유래 CharShape를 HWP5로 저장할 때
Hancom 2020의 canonical inactive underline/strikeout/shadow 표현을 재현한다.

## 범위

- 대상: `src/serializer/char_shape.rs`(신규), `src/serializer/doc_info.rs`의 호출부와 focused unit test.
- inactive underline은 HWP5 canonical type/shape sentinel로 기록한다.
- inactive strikeout은 Hancom direct HWP의 sentinel bits로 기록한다.
- inactive shadow color는 Hancom default `#C0C0C0`로 기록한다.
- active underline/strikeout/shadow의 기존 field encoding은 유지한다.
- `raw_data`가 있는 원본 HWP CharShape는 기존처럼 raw payload 우선이므로 이 변경 대상이 아니다.
- MCP server/client와 PDF paper option은 수정하지 않는다.

## 구현 계획

1. `doc_info.rs`가 이미 1,022줄이므로 `serialize_char_shape`를 전용 `char_shape.rs`로
   이동한다. 호출 공개 범위는 기존 crate 내부로 유지하고, 단일 속성 직렬화 책임을 분리한다.
2. serializer에 named HWP5 canonical constants와 inactive branch를 추가한다. 모델의 logical
   boolean과 HWP5 raw sentinel을 분리해 표현한다.
3. direct HWP canonical attr (`0x3c0400f8` 계열)과 shadow color를 검증하는 unit test를 추가한다.
4. active underline/strikeout/shadow의 bit/color가 canonical inactive branch로 덮이지 않는
   regression test를 추가한다.
5. focused unit test, fmt, build를 통과시킨다.
6. 원본 HWPX를 새 HWP로 저장해 p53 CharShape raw payload를 확인하고, 실제 HWP 2020 MCP
   async PDF와 383쪽 raster comparison으로 Stage 4 probe 개선이 production path에 재현되는지
   검증한다.

## 성공 기준

- inactive semantic style은 direct Hancom HWP와 같은 raw sentinel을 출력한다.
- active style 및 raw-data roundtrip을 손상하지 않는다.
- 실제 HWP 2020 PDF에서 baseline보다 나빠지는 쪽 없이 Stage 4와 같은 7쪽 개선을 재현한다.

## 테스트 결과

### 1. serializer 분리와 단위 검증

`serialize_char_shape`를 `src/serializer/char_shape.rs`로 이동하고 기존
`serializer::doc_info::serialize_char_shape` 공개 경로는 re-export로 유지했다.

- `cargo fmt`, `git diff --check`: 성공
- `cargo test --lib serializer::doc_info::tests -- --nocapture`: 23 passed
- `cargo build --bin rhwp`: 성공
- `doc_info.rs`: 1,022줄에서 902줄, 새 `char_shape.rs`: 86줄

inactive 및 active line/shadow field가 기존 바이트 계약을 유지하는 focused test도 추가했다.

### 2. 전역 canonicalization 후보의 raw HWP5 검증

작업 트리에서 한 번만 실행한 후보는 inactive underline/strikeout/shadow를 Hancom direct HWP의
sentinel로 일괄 기록했다. 원본 HWPX를 HWP로 저장한 뒤 structured CFB/deflate DocInfo 비교를
수행했다.

| 저장본 | direct Hancom과 sentinel 일치 | 다름 | unmatched | ambiguous | CharShape 수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Stage 2 baseline | 0 | 733 | 22 | 182 | 937 |
| 전역 canonicalization 후보 | 733 | 0 | 22 | 182 | 937 |

attr의 sentinel bit를 logical inactive 값으로 정규화해 다시 비교하면 candidate는 direct HWP와
913건 exact, 23건 unmatched, 1건 ambiguous였다. 그러나 이 raw 일치는 HWPX 원본 PDF와의 시각
동치를 보장하지 않았다.

### 3. 실제 HWP 2020 MCP 전송·변환·다운로드

전역 canonicalization 후보 HWP는 실제 async MCP job으로 검증했다.

- job ID: `8b8ff5af-b727-4c94-93e0-1f36f552ba75`
- 변환: 557초, `succeeded`, `run_status=0`, validation `ok`, timeout 없음
- 출력: `PrintToPDFEx`, `PrintMethod=0`, Hancom editor/PDF pages `383/383`
- PDF: `20,569,645` bytes, SHA-256
  `31fa954c8563446b79ce80c282e879645716bae16ca87088e9b8f494410992ee`
- download checksum이 server result와 같고, post-download delivery는 `response_finished`
  (1회, 27,428,253 bytes, 4,084ms, error 없음)였다.

따라서 아래 raster 판정은 실제 HWP 2020 출력과 client 응답 전송까지 완료된 유효한 PDF를
대상으로 한다. 인증 정보와 server 내부 경로는 기록하지 않았다.

### 4. 383쪽 raster 판정과 전역 적용 보류

기준 HWPX PDF와 candidate는 모두 383쪽, `556 x 754 pts`, PDF 1.7이었다. 96dpi 전 페이지를
`pixelmatch(threshold=0.1, includeAA=false)`로 비교했다.

| 비교 | byte-identical | pixel changed pages | pixel total |
| --- | ---: | ---: | ---: |
| 기준 vs Stage 2 baseline | 273 | 109 | 392,833 |
| 기준 vs Stage 4 fail-closed probe | 277 | 105 | 392,203 |
| 기준 vs 전역 canonicalization 후보 | 273 | 109 | 392,833 |
| Stage 4 probe vs 전역 후보 | 376 | 7 | 631 |

전역 후보는 baseline보다 좋아지지 않았고, Stage 4가 개선했던 7쪽(79, 82, 149, 222, 223, 231,
369)의 631픽셀 차이를 되돌렸다. 따라서 모든 logical inactive style을 한 가지 HWP5 sentinel로
치환하는 구현은 production에 채택하지 않는다.

### 5. 커밋 대상의 fail-closed 복원 검증

전역 canonicalization branch를 제거하고, 전용 모듈이 Stage 2와 정확히 같은 바이트를 내는지
동일 원본 HWPX로 재저장해 확인했다.

- HWP 전체 SHA-256: Stage 2 baseline과 refactor-only 저장본이 모두
  `691797542116ad03dff64225c5ee3822628b47aa630129b9e717343bc5a9b97f`
- `cmp -s`: byte-identical
- CHAR_SHAPE payload: 937/937 byte-identical

이번 커밋에는 serializer 분리와 그 동치 test만 포함한다. Stage 6은 이 커밋을 시작점으로,
원본 HWPX logical style에서 7쪽의 안전 부분집합을 식별할 수 있는 추가 provenance/매핑이 있는지
분석한 뒤에만 새 production 변경을 계획한다.
