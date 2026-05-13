# Task #877 구현 계획서

**관련 수행 계획서**: [task_m100_877.md](task_m100_877.md)

## 단계 구성 (3 stage)

---

## Stage 1 — 방어성 가드 (allocation sanity check)

### 목적
HWP3 파서에서 외부 입력으로 받은 `length` 값으로 직접 `vec![0u8; length as usize]` / `Vec::with_capacity(length)` 를 하는 모든 지점에 sanity check 도입.
- 32-bit WASM 의 `RawVec` capacity overflow panic → graceful `Hwp3Error::IoError` 변환
- 64-bit 네이티브의 거대 가상메모리 할당 시도 차단 → 빠른 graceful fail

### 작업 내용

1. **취약 지점 식별** (`src/parser/hwp3/` grep):
   - [records.rs:413](../../src/parser/hwp3/records.rs#L413) `Hwp3AdditionalInfoBlock::read` — `length: u32`
   - [records.rs:392](../../src/parser/hwp3/records.rs#L392) `Hwp3InfoBlock::read` — `length: u16` (16-bit 이므로 ≤64KB, panic 위험 낮음)
   - [mod.rs:932](../../src/parser/hwp3/mod.rs#L932) picture `ext_buf` — `n_ext: u32`
   - [mod.rs:1120](../../src/parser/hwp3/mod.rs#L1120) `ch=29` (`< 1000000` 검증 기존 존재 — 동일 패턴 표준화)
   - drawing.rs, ole.rs 등 다른 `read_exact` 호출 위치 정밀 grep
   
2. **공통 helper 도입** (예: `src/parser/hwp3/util.rs` 또는 `mod.rs` 내부):
   ```rust
   /// stream 의 남은 길이를 초과하는 length 요청 시 Err.
   /// HWP3 binary 파싱에서 garbage length 로 인한 거대 vec! 할당을 방지.
   fn read_sized_buf<R: Read + Seek>(
       reader: &mut R,
       length: usize,
       max_allowed: usize,
   ) -> Result<Vec<u8>, io::Error> {
       if length > max_allowed {
           return Err(io::Error::new(
               io::ErrorKind::InvalidData,
               format!("HWP3 size overflow: requested={length}, max_allowed={max_allowed}"),
           ));
       }
       let mut buf = vec![0u8; length];
       reader.read_exact(&mut buf)?;
       Ok(buf)
   }
   ```
   - `max_allowed` 는 호출 측에서 stream 남은 길이 또는 spec 상 최댓값 전달
   - `Cursor<&[u8]>` 사용처에서는 `get_ref().len() - position()` 으로 산출
   - 일반 `R: Read` 사용처 (drawing.rs 등) 에서는 호출 측에서 적정 상한 전달

3. **호출 측 수정**:
   - `Hwp3AdditionalInfoBlock::read`: `max_allowed = body_data.len() - cursor.position()` (호출자가 전달)
   - picture `ext_buf`: `max_allowed` = body 잔여 또는 spec 권장 (예: 100 MB 상한)
   - `ch=29` 의 기존 `< 1000000` 검증 → 새 helper 로 통합

4. **단위 테스트**: garbage length 입력 시 panic 없이 `Err` 반환 검증.

### 산출물
- 코드: `src/parser/hwp3/` 내 가드 적용
- 보고서: `mydocs/working/task_m100_877_stage1.md`

### 검증
- `cargo test --release` 통과
- sample16 dump 시 panic 없이 graceful Err (또는 부분 파싱 후 Err) 반환
- 다른 HWP3 sample 회귀 없음

---

## Stage 2 — HWP3 picture (ch=11) byte alignment 정합 (근본 원인)

### 목적
sample16 가 1개 paragraph 만 인식되고 나머지 16.18 MB 가 잘못 해석되는 alignment 버그 수정 → 64쪽 전체 파싱 성공.

### 작업 내용

1. **현 picture (ch=11) 처리 흐름 정리** ([mod.rs:852-1053](../../src/parser/hwp3/mod.rs#L852-L1053)):
   - 6 byte 헤더 (u32 + u16) 소비
   - 348 byte `info_buf`
   - `n_ext` (info_buf[0..4]) 만큼 `ext_buf` 추가 소비
   - `pic_type == 3` 일 때 `parse_drawing_object_tree(ext_buf)` (drawing.rs 진입)
   - `caption_paras = parse_paragraph_list(body_cursor, ...)` **재귀 호출**
   - 호출자 (text body 루프) 는 `i += 3` (헤더 4 hchar 소비)

2. **sample16 picture 실제 byte 구조 분석**:
   - decompressed body offset 15078~ 의 bytes 정밀 dump (probe binary 작성)
   - `info_buf` 348 byte 의 모든 필드 추출 + 의미 매핑 (HWP3 spec hwp30_spec.pdf 또는 한컴 변환본 `hwp3-sample16-hwp5.hwp` IR 비교)
   - 정확한 picture record 끝 위치 = ?
   - 같은 sample16 가 처음 1개 paragraph 만 cc=5 인 이유 검증 (실제로 표지가 매우 작은 paragraph + 큰 picture 인 경우 vs. parser 가 일부 byte 를 빠뜨리는 경우)

3. **HWP5 변환본 (`samples/hwp3-sample16-hwp5.hwp`) IR 비교**:
   - `rhwp dump samples/hwp3-sample16-hwp5.hwp` 출력 → 한컴이 변환한 paragraph 구조
   - 첫 paragraph 의 picture record 가 HWP5 에서 어떻게 표현되는지 확인
   - HWP3 → HWP5 변환 spec offset 매핑 추정

4. **alignment 수정**:
   - Picture record 끝나는 정확한 offset 산출 로직 수정
   - `caption_paras` 재귀 호출 시 진입 시점이 caption 영역인지 명확화
   - 가능한 원인 후보:
     - (a) `info_buf` 크기 (348) 가 sample16 에서는 다를 가능성 (variant 1.7 vs 1.5 등 HWP3 minor version)
     - (b) `ext_buf` 무조건 read 대신 `pic_type == 3` 일 때만 read
     - (c) `caption_paras` 재귀 호출 진입 조건 (caption 이 있을 때만)
     - (d) 다른 미처리 sub-record (예: pic_type 별 별도 데이터 블록)

5. **회귀 방지**: 기존 HWP3 sample (sample01/10/11/13/14) 의 picture 처리 동일성 유지.

### 산출물
- 코드: `src/parser/hwp3/mod.rs` picture (`ch=11`) 부분
- 분석 문서: `mydocs/tech/hwp3_picture_record_alignment.md` (byte 구조 정리)
- 보고서: `mydocs/working/task_m100_877_stage2.md`

### 검증
- `cargo run --bin rhwp -- dump samples/hwp3-sample16.hwp` 성공 (64쪽 인식)
- `cargo run --bin rhwp -- dump-pages samples/hwp3-sample16.hwp -p 0` 정상
- 다른 HWP3 sample 회귀 없음 (sample01/10/11/13/14 dump 비교)

---

## Stage 3 — WASM panic hook + 통합 회귀 테스트

### 목적
- WASM 환경에서 향후 미식별 panic 발생 시 진단 가능하도록 panic hook 정비
- sample16 회귀 방지를 위한 통합 테스트 추가

### 작업 내용

1. **WASM panic hook 점검**:
   - 현재 console 로그에 `panicked at library/alloc/src/raw_vec/mod.rs:28:5: capacity overflow` 메시지는 노출 — `console_error_panic_hook` 또는 유사 hook 이 이미 동작 중인지 확인
   - `src/wasm_api.rs` 초기화 부에서 panic hook 설정 코드 확인 + 누락 시 추가
   - 빌드 옵션 (debug-info, source map) 검토 — wasm function index 만 노출되는 stack 을 source 라인으로 매핑하는 방법 조사 (debug build 비용 vs. release 성능)
   - 결정: 매핑 비용이 크면 panic hook 만 강화하고 source map 은 별도 task 로 미룸

2. **try_reserve 패턴 검토**:
   - Stage 1 의 helper 외에 `Vec::try_reserve` / `Vec::try_with_capacity_in` 같은 alloc API 적용 가치 검토
   - 현재 stable Rust 에서 `Vec::with_capacity` 의 panic 가능성 vs. `try_reserve_exact` 의 fallible 패턴
   - 보수적 결정: 본 task 에서는 helper 함수 + length 검증만 하고 `try_reserve` 도입은 별도 RFC 로

3. **통합 회귀 테스트 추가**:
   - `tests/issue_877.rs` (또는 `tests/hwp3_sample16.rs`) 신설
   - sample16 로딩 → `DocumentCore::from_bytes()` panic 없이 성공
   - paragraph 개수 / 페이지 수가 1 보다 큰지 (Stage 2 후 64쪽 인식 검증)
   - 다른 HWP3 sample (sample14 등) 의 회귀 없음 sanity check

4. **rhwp-studio 시각 확인**:
   - Docker 로 WASM 빌드 (`docker compose --env-file .env.docker run --rm wasm`)
   - rhwp-studio 에서 sample16 로드 → 페이지 표시 확인
   - 스크린샷 캡처 → 보고서 첨부

### 산출물
- 코드: `src/wasm_api.rs` panic hook (필요 시), `tests/issue_877.rs`
- 보고서: `mydocs/working/task_m100_877_stage3.md`

### 검증
- `cargo test --release tests::issue_877` 통과
- `cargo test --release` 전체 통과
- WASM 빌드 → rhwp-studio sample16 로드 시각 확인

---

## 최종 산출물

- 코드 수정: `src/parser/hwp3/` (records.rs, mod.rs), `src/wasm_api.rs`, `tests/issue_877.rs`
- 문서:
  - 수행 계획서: `mydocs/plans/task_m100_877.md`
  - 구현 계획서: `mydocs/plans/task_m100_877_impl.md`
  - Stage 1/2/3 보고서: `mydocs/working/task_m100_877_stage{1,2,3}.md`
  - 분석 문서: `mydocs/tech/hwp3_picture_record_alignment.md`
  - 최종 보고서: `mydocs/report/task_m100_877_report.md`
- 신규 sample git 등록: `samples/hwp3-sample16.hwp` (및 `-hwp5.hwp` / `-hwp5.hwpx` 변환본)

## 위험 / 미해결 가능성

- **Stage 2 가 가장 불확실**: HWP3 spec 의 picture record 정확한 layout 이 sample16 의 variant 와 일치하지 않을 가능성. 분석 후 본 task 범위 내에서 해결 불가 판단 시 Stage 1 (방어성) 만 머지하고 alignment 는 별도 task 로 분기.
- HWP3 spec 문서 부재 시 한컴 변환본 (`hwp3-sample16-hwp5.hwp`) IR 비교 + 다른 HWP3 sample 실측으로 reverse-engineer.

## 진행 순서

1. Stage 1 시작 → 완료 후 Stage 1 보고서 + 승인 요청
2. Stage 2 시작 → 분석 후 (alignment 가능 / 불가능) 결정 → Stage 2 보고서 + 승인 요청
3. Stage 3 시작 → 완료 후 Stage 3 보고서 + 승인 요청
4. 최종 결과 보고서 + orders 갱신 → 승인 요청 → 머지
