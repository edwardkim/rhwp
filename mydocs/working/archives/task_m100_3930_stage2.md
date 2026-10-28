# Task M100 #3930 Stage 2 - Hancom 저장본 호환성 재검증

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `53af9e7e3` (`fix: HWPX 저장 조판 입력 보존 (#3930)`)
- 기록일: 2026-08-05 KST
- 상태: 조사 진행 중

## 배경

Stage 1은 rhwp의 `HWPX -> HWP -> 재열기` 저장 경로에서 387쪽, p144 표 이월, p30
바탕쪽 선택을 회복했다. 같은 편람을 Hancom Office 2020 MCP로 추가 변환했을 때 다음 두
관찰값이 남았다.

1. Hancom `HWPX -> HWP` 저장본을 rhwp가 재조판하면 393쪽으로 읽힌다.
2. Hancom `HWP -> PDF` 출력의 1·2쪽 raster가 비어 있고 3쪽부터 내용이 보인다.

이 두 결과를 단순히 #3930의 oracle 차이라고 종료하지 않는다. 원본 문서의 실제 Hancom 조판,
HWP5 parser IR, rhwp renderer, MCP 인쇄 결과를 분리해 어느 계층의 결함인지 확정한다.

작업지시자 결정: **Hancom Office 2020 MCP 출력이 표준이다.** 따라서 rhwp 내부 페이지 수나
재조판 결과만으로 수용하지 않는다. HWPX 원본, Hancom 저장 HWP, rhwp 저장 HWP를 모두
Hancom MCP PDF 인쇄 결과와 대조해 저장 계약을 판정한다. HWP 원본 PDF의 앞쪽 빈 페이지도
Hancom가 출력한 그대로를 표준으로 취급한다.

## 계획

1. 원본 HWPX, Hancom `HWPX -> HWP`, rhwp `HWPX -> HWP`를 각각 Hancom MCP PDF로
   인쇄해 쪽수·본문·앞쪽 raster를 대조한다.
2. Hancom HWP 저장본을 rhwp가 393쪽으로 읽는 차이는 HWP5 parser/renderer 조사 대상으로
   기록하되, HWP 2020 PDF가 표준과 일치하면 단독 수용 실패로 취급하지 않는다.
3. rhwp 저장 HWP의 Hancom PDF가 원본 HWPX Hancom PDF와 불일치하면 직렬화/어댑터를
   최소 보정하고, 실제 MCP PDF 증적을 회귀에 연결한다.
4. HWP 원본 PDF의 앞쪽 빈 페이지는 Hancom 출력 그대로가 표준이므로 제거하거나 채우지
   않는다. all-page blank 또는 본문 누락만 결함으로 처리한다.

## 수용 기준

- rhwp 저장 HWP를 Hancom MCP로 PDF 인쇄했을 때, 원본 HWPX의 Hancom PDF와 쪽수·본문
  유효성·앞쪽 표지/빈쪽 계약이 일치한다.
- Hancom HWP 저장본의 rhwp 393쪽 관찰값은 Hancom PDF와 비교해 실제 보정 필요 여부를
  판정한다. rhwp 내부 결과만으로 파일을 수정하지 않는다.
- 모든 구현은 실제 편람 fixture와 focused Rust 회귀, 기존 adapter 회귀, HWP 2020 MCP 재검증을
  통과한다.

## 조사 결과

### Hancom HWPX -> HWP 저장 계약

- 원본 HWPX의 Hancom PDF와 Hancom `HWPX -> HWP` 저장본을 다시 Hancom PDF로 인쇄한
  결과는 모두 383쪽, 19,657,247 byte다.
- 두 PDF의 전체 `pdftotext` 결과는 모두 274,632자다. 144dpi raster p1, p144, p383 PNG도
  byte-identical이다.
- 따라서 Hancom `HWPX -> HWP` 자체는 원본 HWPX의 실제 인쇄 결과를 보존한다. rhwp가 이
  Hancom HWP를 393쪽으로 재조판하는 현상은 rhwp renderer의 진단 호환성 범위이며, HWP 2020
  저장 파일의 실제 변환 결함이 아니다.
- 다음 검증은 rhwp Stage 1 저장 HWP를 Hancom PDF로 인쇄해 동일한 표준 결과를 보존하는지
  확인한다. 이 결과가 일치해야 #3930의 저장 경로 보정도 Hancom 표준을 만족한다고 판정한다.

### 실패: rhwp HWP -> Hancom PDF

- Stage 1의 rhwp 저장 HWP를 같은 MCP로 PDF 인쇄한 job은 시작 뒤 약 30초에 실패했다.
  `run_status=139`, `validation=fail`, `validation_detail=invalid_output`, PDF 0 byte다.
- 따라서 rhwp 재열기 387쪽만으로 Stage 1 저장본을 수용할 수 없다. 이 파일은 Hancom Office
  2020 호환 HWP가 아니며, HWP5 serializer/adapter 레코드 계약을 Hancom 저장본과 대조해
  보정해야 한다.
- 다음 구현 전 조사 대상은 Stage 1 바탕쪽 `Both`/`Odd` 슬롯 물질화와 HWP5 `LIST_HEADER`
  payload/순서다. Hancom 저장 HWP와 rhwp 저장 HWP의 inventory·raw record를 비교해 crash를
  일으키는 최소 차이를 격리한다.

### 원인 확정: SECTION_DEF 바탕쪽 개수 플래그

- HWPX 원본 구역 2는 희소 `Odd` 바탕쪽 한 개이므로 원본 `SECTION_DEF.flags`의 상위 바탕쪽
  개수 비트가 `0x4000_0000`이다.
- Stage 1은 앞 구역의 짝수 바탕쪽을 복제해 구역 2를 `Both` + `Odd` 두 슬롯으로 바꿨지만,
  기존 어댑터는 **원래 flags가 이미 2개용 `0x8000_0000`일 때만** 다중 슬롯 Hancom 값
  `0xC000_0000`으로 변경했다. 따라서 실제 저장 HWP는 LIST_HEADER 두 개와 1개용
  `0x4000_0000` 플래그를 함께 기록했다.
- Stage 1 저장본 덤프의 구역 2 flags가 그대로 `0x40080000`인 것으로 재현했다. 이는 HWP5
  레코드 계약 불일치이며, HWP 2020 MCP의 `run_status=139`와 일치한다.
- 보정: `master_pages.len()`을 사실상 단일 진실원으로 삼는다. 한 개면
  `0x2000_0000`, 두 개 이상이면 `0xC000_0000`을 상위 세 비트에 기록한다. 원래 HWPX
  플래그 값은 슬롯 물질화 전의 개수이므로 조건으로 사용하지 않는다.

## 구현 및 중간 검증

### 코드 보정

- `materialize_single_master_page_flags`와 `materialize_multi_master_page_flags`가 이전 HWPX
  flags가 아니라 최종 `master_pages.len()`으로 HWP5 상위 개수 비트를 동기화하도록 수정했다.
- 단일→다중 슬롯으로 바뀌는 회귀를 `materialized_second_master_page_updates_stale_single_master_flag`
  단위 테스트로 고정했다.
- 실제 편람 회귀는 저장 HWP의 구역 2가 `Both`/`Odd` 두 바탕쪽 슬롯과
  `flags & 0xe000_0000 == 0xc000_0000`을 함께 보유하도록 검증한다.

### 로컬 결과

- `cargo test --lib stale_single_master_flag -- --nocapture`: 1 passed.
- `cargo test --test issue_3930_hwpx_hwp_save_layout -- --nocapture`: 1 passed (387쪽).
- `cargo test --test hwpx_to_hwp_adapter -- --nocapture`: 50 passed, 15 ignored.
- `cargo clippy --lib -- -D warnings`: passed.
- `cargo fmt --check`, `git diff --check`: passed.
- 수정된 `release-test`로 편람을 다시 저장했다. 저장본은 9,086,976 byte이고
  `--verify-pages` 결과가 before=387, after=387, identical=true다. 구역 2 덤프도
  `Both`/`Odd` 두 슬롯 및 `flags=0xC0080000`을 확인했다.

### HWP 2020 MCP 진행 상태

- 수정 저장 HWP를 PDF로 비동기 전송했다. 초기 상태는 `queued`, 이후 `converting`으로 전이했고
  출력 바이트가 1,249,280 byte에서 3,350,528 byte로 증가했다.
- 이전 Stage 1 저장본의 약 30초 `run_status=139` 종료와 달리, 이 시점에는 서버가 정상 인쇄를
  수행 중이다. 완료 후에는 SHA-256 검증 download, PDF 쪽수·전체 본문 텍스트, p1/p30/p144/p383
  raster를 원본 HWPX의 HWP 2020 PDF와 비교해 수용 여부를 확정한다.

## 재판정: 실제 HWP 2020 MCP 표준 결과

앞 절의 `Both`/`Odd` 확장 가설은 최종 Hancom raw HWP 비교와 실제 MCP 출력으로 기각했다.
HWPX의 단일 `Odd` 바탕쪽은 HWP 2020 저장본에서 `LIST_HEADER` 하나와
`SECTION_DEF.flags & 0xe000_0000 == 0x8000_0000`으로 표현되며, 이전 구역의 짝수 바탕쪽을
상속한다. 따라서 `Both + Odd`로 확장하면 한컴 표준 표현과 달라진다.

최종 Stage 2 보정은 다음과 같다.

- 단일 `Odd` HWPX 바탕쪽을 슬롯 하나와 `0x8000_0000` flag로 보존한다.
- HWP5 parser가 이 단일 `Odd` flag를 `Both`로 오독하지 않는다.
- HWPX `numberingType="PICTURE"` 일반 개체에도 Hancom HWP5 공통 개체 bit 28을 보존한다.
- HWPX 그림의 `SC_PICTURE` 18-byte extra와 한컴의 brightness/contrast 저장 순서를 보존한다.

새 HWP 2020 MCP server/client timeout 계약(`10..3600`, default `900`)을 배포한 뒤 최종 후보를
`timeout_seconds=3600`으로 인쇄했다. 실제 job은 735초에 `PrintToPDFEx`, `PrintMethod=0`,
`run_status=0`, `validation=ok`, 편집기/PDF 모두 383쪽으로 완료했고, client SHA-256 검증 후
PDF를 저장했다.

원본 HWPX의 Hancom PDF와 이 최종 후보 PDF는 383쪽 및 556 x 754 pt 용지가 일치한다. 그러나
동일 `pdftoppm -r 96 -png`의 383쪽 전수 비교에서 273쪽만 byte-identical이고 110쪽은 다르다.
첫 불일치는 4쪽이며, 304·383쪽도 다르다. 따라서 timeout·인쇄 method·페이지 수·용지 설정은
수용됐지만 저장 정합성의 시각 차이는 남아 있다.

## Stage 2 종료 기준

- focused Rust regression을 다시 실행해 단일 Odd flag, HWPX master slot, PICTURE bit 28,
  실제 편람 저장 path를 고정한다.
- 코드와 이 Stage 문서를 함께 커밋한 뒤 최신 `upstream/devel`을 반영한다.
- 이후 Stage 3에서 첫 불일치 p4 및 대표 후반 p304/p383의 Hancom HWP raw record와 rhwp 저장
  record를 대조한다. 110쪽 불일치를 수용 완료로 선언하지 않는다.

### 최종 focused regression: HWP5 단일 Odd parser

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 \
  cargo test --lib hancom_single_odd_master_flag_is_not_parsed_as_both
```

결과: exit code `0`, `1 passed; 0 failed`. `0x8008_0000`의 단일 master slot을 `Odd`로 복원하고
단일/다중 다른 flag 조합을 `Both`로 오독하지 않는 회귀를 확인했다.

### 최종 focused regression: HWPX 희소 Odd 슬롯

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 \
  cargo test --lib sparse_odd_master_keeps_hancom_single_slot_contract
```

결과: exit code `0`, `1 passed; 0 failed`. 이전 구역의 Even과 현재 구역의 단일 Odd가 있어도
HWPX adapter는 현재 구역을 `Both + Odd` 두 슬롯으로 확장하지 않고 한컴의 `Odd` 슬롯 하나를
유지한다.

### 최종 focused regression: 단일 Odd SECTION_DEF flag

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 \
  cargo test --lib single_odd_master_page_flags_preserve_hancom_inherited_even_contract
```

결과: exit code `0`, `1 passed; 0 failed`. 단일 Odd HWP5 master page의 상위 flag를
`0x8000_0000`으로 materialize해 한컴의 이전 Even 상속 계약을 유지한다.

### 실제 편람 저장 회귀의 환경 실패

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 \
  cargo test --test issue_3930_hwpx_hwp_save_layout -- --nocapture
```

결과: exit code `101`. Rust assertion 실패가 아니라 build archive 작성 중 다음 환경 오류가 발생했다.

```text
failed to build archive at .../target/task3930-stage2/.../librhwp.rlib:
No space left on device (os error 28)
```

다음 단계는 공유 target·사용자 산출물을 삭제하지 않고, 이 stage에서 만든 전용 target과 전수 raster
산출물의 용량을 먼저 측정한 뒤 재생성 가능한 정확한 경로만 정리하는 것이다.

용량 측정 결과:

| 경로 | 용량 |
| --- | ---: |
| `/` 파일시스템 여유 | 163MiB (146GiB 중 140GiB 사용) |
| `target/task3930-stage2` | 1.3GiB |
| 이번 전수 raster | 97MiB |

실행 중인 Cargo/Rust process는 없었다. `target/task3930-stage2`는 이번 stage의 정확한 전용
build directory이고 중간 build 결과만 포함하므로, 다음 재실행 전에 이 경로만 정리할 수 있다.
전수 raster는 110쪽 불일치 근거이므로 유지한다.

정리 결과: `find target/task3930-stage2 -depth -delete`로 정확한 전용 directory만 제거했다.
공유 target, 기존 review target, HWP/PDF/raster output은 삭제하지 않았다. 파일시스템 여유는
163MiB에서 1.5GiB로 증가했다.

재실행은 test 동작을 바꾸지 않는 `CARGO_PROFILE_DEV_DEBUG=0`을 추가해 전용 target의 debug info
용량을 줄인다. 이 값은 assertion·parser·serializer 실행 경로에는 영향을 주지 않는다.

### 실제 편람 저장 회귀 재실행

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
  cargo test --test issue_3930_hwpx_hwp_save_layout -- --nocapture
```

결과: exit code `0`.

```text
test issue_3930_preserves_page_count_and_inherited_even_master_page ... ok
test result: ok. 1 passed; 0 failed; finished in 25.11s
```

전용 target은 761MiB의 파일시스템 여유를 남겼다. 단일 Odd HWP5 flag, 희소 HWPX master slot,
PICTURE bit 28, 그림 raw extra, 편람 native 저장·재열기 계약은 모두 regression으로 고정됐다.
MCP 전 페이지 비교에서 남은 110쪽 시각 불일치는 이 내부 회귀 통과와 별개의 다음 Stage 3 분석 대상이다.

### PICTURE 일반 개체 및 포맷 검사

명령:

```bash
CARGO_TARGET_DIR=target/task3930-stage2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
  cargo test --lib test_parse_rect_ratio_as_round_rate
cargo fmt --check
git diff --check
```

결과: 모두 exit code `0`.

- `test_parse_rect_ratio_as_round_rate`: `1 passed; 0 failed`. `numberingType="PICTURE"` 일반
  개체가 HWP5 common attr bit 28을 설정하는 경로를 포함한다.
- `cargo fmt --check`, `git diff --check`: formatting 및 공백 오류 없음.

## Stage 2 결론

HWP 2020 MCP 표준으로 실제 출력 성공과 383쪽/용지/PrintToPDFEx 계약을 확인했고, 110쪽 시각
불일치도 전수 raster 수치로 확정했다. 단일 Odd master·PICTURE bit 28·그림 raw extra 보정은 local
regression으로 통과했으나, 110쪽 차이를 해결하지 못했으므로 Stage 2는 부분 완료다. 이 코드와
문서를 함께 커밋한 뒤 Stage 3에서 raw record 차이를 계속 조사한다.
