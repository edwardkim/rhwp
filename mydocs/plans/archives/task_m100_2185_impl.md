# 구현 계획서 — Task M100 #2185

## 1. 구현 원칙

이번 수정은 저장 포맷의 값을 바꾸는 작업이 아니라, 저장값을 소비하는 공통 한글
라인브레이커의 의미를 바로잡는 작업이다.

| 경로 | 현재 저장·내부 계약 | 한컴 통제 실측 의미 | 처리 |
|------|--------------------|--------------------|------|
| HWP5 `ParaShape.attr1 bit7` | `0` 또는 `1` 원형 보존 | `0=어절`, `1=글자` | 파서·직렬화 유지 |
| HWPX `KEEP_WORD` | bit7=`1` | 글자 단위 채움 | 매핑 유지 |
| HWPX `BREAK_WORD` | bit7=`0` | 어절 유지 | 매핑 유지 |
| 모델·Studio UI | `0=어절`, `1=글자` | 통제 실측과 일치 | 유지 |
| composer 토큰화·줄 채움 | 현재 `1=어절`, `0=글자` | 통제 실측과 반대 | 두 소비 조건 정정 |

HWPX 값의 이름만으로 의미를 재해석하지 않고 #2185의 한컴 통제 실측을 권위로 삼는다.
`0da18bbc`에는 PUA와 공백 압축 변경도 함께 있으므로 커밋 전체를 되돌리지 않고, 의미를
뒤집은 두 조건만 복구한다.

## 2. 예상 수정 파일

- `src/renderer/composer/line_breaking.rs`
  - 한글 어절 토큰 분기를 `korean_break_unit == 0`으로 복구한다.
  - 단일 한글 글자의 줄바꿈 가능 조건을 `korean_break_unit == 1`로 복구한다.
  - HWPX 이름과 실제 renderer 의미를 혼동하는 주석을 통제 실측 기준으로 정리한다.
- `src/renderer/style_resolver.rs`
  - 실제 raw bit 추출은 유지하고 잘못된 `0=글자, 1=어절` 주석만 정정한다.
- `src/renderer/composer/tests.rs`
  - 기존 반대 의미를 고정한 토크나이저 테스트 인자를 바로잡는다.
  - bit0 어절 유지와 bit1 글자 채움을 실제 줄 경계로 구분하는 reflow 테스트를 보강한다.
- `tests/issue_2185_korean_break_unit.rs` (신규)
  - 공개 거대 셀 샘플의 첫 입력과 저장·재로드까지 고정하는 전용 통합 회귀 테스트를 둔다.

다음 파일의 값 매핑과 기능 코드는 수정하지 않는다.

- `src/parser/doc_info.rs`, `src/serializer/doc_info.rs`
- `src/parser/hwpx/header.rs`, `src/serializer/hwpx/header.rs`
- `src/model/style.rs`, `src/document_core/helpers.rs`
- `rhwp-studio/src/core/types.ts`, `rhwp-studio/src/ui/para-shape-dialog.ts`

## 3. 구현 단계

### Stage 1. 공통 의미 복구와 단위 회귀 테스트

1. `test_tokenize_korean_eojeol`을 bit0 어절 단위 계약으로, 기존
   `test_tokenize_korean_break_word_chars`를 bit1 글자 단위 계약으로 수정한다.
2. 공백 없는 한글과 다음 어절 일부가 들어갈 수 있는 폭을 사용해 bit0과 bit1의 줄 시작점이
   실제로 달라지는 reflow 테스트를 추가한다. 기존 `test_reflow_korean_eojeol_wrap`처럼 양쪽
   모드가 우연히 같은 결과를 내는 입력은 결정적 게이트로 사용하지 않는다.
3. `line_breaking.rs`의 토큰화·줄 채움 조건 두 곳만 복구하고 `style_resolver.rs`의 계약 주석을
   정정한다. 기본값 `0`과 raw bit 추출은 변경하지 않는다.
4. 표적 단위 테스트를 실행하고 `mydocs/working/task_m100_2185_stage1.md`에 변경과 결과를
   기록한 뒤 승인을 요청한다.

검증 명령:

```bash
cargo test --lib renderer::composer::tests::test_tokenize_korean -- --nocapture
cargo test --lib renderer::composer::tests::test_reflow_korean -- --nocapture
cargo test --lib parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit -- --exact
cargo test --lib serializer::hwpx::header::tests::write_para_pr_emits_align_and_break_from_preserved_bits -- --exact
```

### Stage 2. 실제 편집·저장 통합 회귀 핀

1. 별도 `tests/issue_2185_korean_break_unit.rs`를 만들고 기존 #1949 성능 테스트와 정확성
   책임을 분리한다.
2. HWP/HWPX 공개 샘플의 대상 경로
   `(section=0, parent=0, control=2, cell=2, cell_para=5)`에서 다음을 확인한다.
   - ParaShape bit7=`1`
   - 입력 전 `LINE_SEG.text_start == [0, 44, 84, 122]`, 네 줄
   - offset 130에 `1`을 지연 페이지네이션 경로로 입력한 뒤 같은 시작점과 네 줄 유지
   - 마지막 텍스트가 `하여 적용한다.1`로 끝나고 다음 문단 첫 `vpos`가 불변
3. 전체 페이지네이션 후 115쪽을 확인하고, 원본 포맷으로 저장·재로드한 뒤 시작점·줄 수·
   `vpos`·페이지 수를 다시 확인한다. 셀 접근 및 저장 재로드 패턴은
   `tests/issue_2164_cell_enter_overlap.rs`를 따른다.
4. `mydocs/working/task_m100_2185_stage2.md`에 HWP/HWPX 결과와 실행 시간을 기록한 뒤
   승인을 요청한다.

검증 명령:

```bash
cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture
cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture
cargo test --profile release-test --test issue_2164_cell_enter_overlap
```

### Stage 3. 포맷·WASM·인접 회귀 검증

1. HWPX `KEEP_WORD→bit1`, `BREAK_WORD→bit0` 파서 테스트와 직렬화 역매핑 테스트가 수정 없이
   통과하는지 재확인한다.
2. HWP5/HWPX roundtrip, #937 PUA, #1949 거대 셀 렌더를 실행해 `0da18bbc`의 다른 변경을
   훼손하지 않았음을 확인한다.
3. WASM을 빌드하고 Studio에서 공개 샘플 끝에 `1`을 입력해 앞 세 줄이 유지되고 마지막 줄만
   바뀌는지 확인한다. Console에서 전체 pagination fallback이 발생하지 않는지도 확인한다.
4. `mydocs/working/task_m100_2185_stage3.md`에 native/WASM/브라우저 결과를 기록한 뒤
   승인을 요청한다.

검증 명령:

```bash
cargo test --profile release-test --test issue_937
cargo test --profile release-test --test hwp5_roundtrip_baseline
cargo test --profile release-test --test hwpx_roundtrip_baseline
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm test && npm run build
```

### Stage 4. 광역 게이트와 결과 보고

1. 전체 Rust 테스트, clippy, 포맷 검사를 실행한다.
2. 확보 가능한 #2169 kbu/kbu2 통제 자료와 장문 줄바꿈 코퍼스를 대조한다. 현재 HEAD에는
   해당 픽스처가 없으므로 경로와 권위 자료를 먼저 확보하고, 없으면 통과로 간주하지 않는다.
3. 기존 기준선에 대한 359문서 recount와 사용 가능한 대량 서베이에서 `REGRESSED=0`을
   확인한다. 코퍼스가 현재 환경에 없으면 외부 실행을 요청하고 완료를 선언하지 않는다.
4. 회귀가 나오면 다른 줄바꿈 휴리스틱이나 폰트 메트릭을 즉석 보정하지 않는다. 재현 문서와
   원인을 분리해 작업지시자에게 보고하고 범위 확장 승인을 받는다.
5. 모든 게이트 통과 후 `mydocs/working/task_m100_2185_stage4.md`와
   `mydocs/report/task_m100_2185_report.md`를 작성하고 최종 승인을 요청한다.

검증 명령:

```bash
cargo fmt --check
cargo test --profile release-test --tests
cargo clippy --all-targets --all-features -- -D warnings
```

## 4. 단계별 커밋 원칙

- 각 Stage의 소스·테스트 변경과 `task_m100_2185_stageN.md`를 같은 커밋에 포함한다.
- 커밋 메시지는 `Task #2185: ...` 형식을 사용한다.
- 각 Stage 완료 후 작업지시자 승인 없이 다음 Stage로 넘어가지 않는다.
- 기능 변경에 무관한 포맷, 생성물, `scripts/frontend-metrics/`는 커밋하지 않는다.
- 최종 보고서와 오늘할일 상태 갱신도 최종 승인 전 타스크 브랜치에 남긴다.

## 5. 중단 기준과 비목표

- HWP/HWPX 파서·직렬화 매핑 변경이 필요해지는 경우 구현을 멈추고 별도 승인을 받는다.
- 기존 공식 PDF pin 또는 359 recount에서 새 페이지 수 회귀가 한 건이라도 나오면 해당
  Stage를 완료 처리하지 않는다.
- 증분 `LINE_SEG` reflow, `column_start`·줄별 tag 보존, 폰트 메트릭, 거대 셀 입력 성능은
  이번 구현 범위에 포함하지 않는다.
- 이슈 close, 브랜치 통합과 원격 push는 별도 승인 없이는 수행하지 않는다.

본 구현계획 승인 전에는 소스 코드와 테스트를 수정하지 않는다.
