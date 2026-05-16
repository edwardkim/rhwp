---
issue: 929
stage: 2
status: 완료 — 승인 대기
spec_ref: mydocs/tech/한글문서파일구조3.0.md
---

# Task #929 Stage 2 완료 보고서 — 원인 확정 + 수정 설계

## 사용 권위 자료

`mydocs/tech/한글문서파일구조3.0.md` (HWP3 공식 spec 정리본). 작업지시자 지시.

## 추가 진단 (Stage 1 진단 보강)

para 21 한정 char 단위 trace 추가:

```rust
if paragraphs.len() == 21 {
    eprintln!("[diag929]     para21 i={} ch_pos={} ch=0x{:04x}", i, ch_pos, ch);
}
```

진단 결과:

```
[diag929] loop iter para_idx=21 pos=7709 body_len=12903
[diag929]   ParaInfo ok follow=1 char_count=24 line_count=1 ...
[diag929]   para21 body peek(7752..7832):
  00 00 00 00 ed 00 f9 21 00 00 00 00 8a 24      ← LineInfo×1 (14B)
  68 00 72 00 65 00 66 00                        ← "href" (4 hchars, 8B)
  09 00 11 00 00 00 09 00                        ← tab1: ch=9 + hunit=0x0011 + word=0 + close=9 (8B, 4 hchars)
  09 00 a9 01 00 00 09 00                        ← tab2 (8B, 4 hchars)
  09 00 a9 01 00 00 09 00                        ← tab3 (8B, 4 hchars)
  85 a2 e1 ac 81 b7                              ← johab 3글자 (6B, 3 hchars)
  20 00 55 00 52 00 4c 00                        ← " URL" (8B, 4 hchars)
  0d 00                                          ← CR 문단 종결 (1 hchar)
```

각 영역의 hchar 카운트 합: 4 + 4 + 4 + 4 + 3 + 4 + 1 = **24 char_count** ✓

따라서 **para 21 의 실제 내용은 `"href[tab][tab][tab]<한글3>·URL"`** 이며, char_count=24 가 spec 과 정확히 일치한다.

## 원인 (확정)

`src/parser/hwp3/mod.rs` 의 본문 char loop 에서 **ch=9 (탭) 처리에 결함**:

```rust
9 => {
    char_offsets.push(utf16_len);
    utf16_len += 1;
    text_string.push('\t');
}
```

- HWP3 spec §10.5 표 39 의 탭 구조 = **8 bytes** (`hchar(=9) + hunit(탭 폭) + word(점끌기) + hchar(=9, 닫기)`). 
- 코드는 `read_u16` 으로 ch (2 bytes) 만 소비, 추가 6 bytes 미소비.
- char_count 단위는 hchar(2 byte) — 8 bytes = **4 hchar** 차지지만 코드는 `i += 1` 만 (1 hchar 만 카운트).

결과:
1. cursor 가 6 bytes 일찍 진행 → tab 내부 hunit(=0x0011=17 hunit) 을 다음 hchar 로 잘못 read.
2. 0x0011 이 ch=17 (각주/미주) 컨트롤로 해석되어 각주 14B 식별 정보 + `parse_paragraph_list` 재귀 호출.
3. 재귀 진입 위치가 paragraph 시작이 아니라 tab2 의 hunit 값 → garbage `ParaInfo` (char_count=57762, line_count=33196) → `LineInfo::read` EOF.

## 다른 hwp3 샘플과의 정합성

기존 hwp3 샘플 (sample, sample10, sample11, sample13, sample14, sample16) 은 정상 동작 — 즉 tab 이 없거나, tab 이 있어도 우연히 cursor 어긋남이 garbage 분기로 흘러가도 외형상 종료된 경우일 수 있다 (텍스트가 부정확할 수는 있음).

Stage 3 회귀 검증에서 다음 사항을 같이 확인한다:
- 기존 6 샘플의 `parse_hwp3` 가 여전히 Ok 를 반환하는가
- 기존 6 샘플의 dump 결과 (텍스트 추출) 가 수정 전/후 변하지 않는가, 또는 더 정확해지는가

## 수정 설계

### 코드 변경

`src/parser/hwp3/mod.rs` 의 char loop 안 `9 =>` 분기 (현재 line 368) 를 spec §10.5 표 39 정합 처리로 변경:

```rust
9 => {
    // [#929] HWP3 spec §10.5 표 39: 탭 = 8 bytes
    //   hchar(=9) + hunit(탭 폭) + word(점끌기) + hchar(=9, 닫기)
    // char_count 단위는 hchar(2B), 8 bytes = 4 hchar 차지.
    // 코드 패턴: 7|8 (날짜) 와 동일 — 추가 6 bytes 읽고 i+=3 (이미 +1 됨, 총 4 hchar).
    let mut buf = [0u8; 6];
    if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
    for k in 0..3usize {
        if i + k < hwp3_char_to_utf16_pos.len() {
            hwp3_char_to_utf16_pos[i + k] = utf16_len;
        }
    }
    i += 3;
    char_offsets.push(utf16_len);
    utf16_len += 1;
    text_string.push('\t');
}
```

변경 범위: **단 1 분기**, 약 7 lines 추가/수정.

### 회귀 가드 테스트

Stage 2 단계에서 **의도된 실패 상태** 로 추가 (Stage 3 코드 수정 시 통과 전환):

위치 후보: `tests/hwp3_sample_parse.rs` (신규) 또는 기존 통합 테스트 모듈 안.

테스트 내용:

```rust
#[test]
fn task929_hwp3_sample19_parses_ok() {
    let data = std::fs::read("samples/hwp3-sample19.hwp").unwrap();
    let doc = rhwp::parser::hwp3::parse_hwp3(&data)
        .expect("hwp3-sample19.hwp must parse without error");
    // 기본 정합: 최소 1 섹션, 일부 문단 존재
    assert!(!doc.sections.is_empty(), "section count must be > 0");
    let para_count: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
    assert!(para_count > 20, "expected > 20 paragraphs, got {}", para_count);
}

#[test]
fn task929_existing_hwp3_samples_no_regression() {
    for sample in &[
        "samples/hwp3-sample.hwp",
        "samples/hwp3-sample10.hwp",
        "samples/hwp3-sample11.hwp",
        "samples/hwp3-sample13.hwp",
        "samples/hwp3-sample14.hwp",
        "samples/hwp3-sample16.hwp",
    ] {
        let data = std::fs::read(sample).expect(sample);
        rhwp::parser::hwp3::parse_hwp3(&data)
            .unwrap_or_else(|e| panic!("{}: {:?}", sample, e));
    }
}
```

기존 6 샘플 회귀 가드는 본 task 의 직접 fixture 가 아니지만, ch=9 처리 변경이 광범위하므로 필수.

### 진단 코드 제거 (Stage 3)

`src/parser/hwp3/mod.rs` 의 `[diag929]` 태그 eprintln! 모두 제거:
- `loop iter`, `ParaInfo ok`, `LineInfo::read FAIL`, `char read_u16 FAIL`, `ctrl ch`, `para21 body peek`, `para21 i= ch_pos=`, `ENTER ch=17 …`, `ch=17 after info_buf …`.

Stage 1 + Stage 2 보강 진단 합쳐 약 40 라인 모두 제거.

## 변경 파일

본 Stage 2 commit 에 포함:
- `src/parser/hwp3/mod.rs` — para21 char trace + ParaInfo follow trace 진단 추가분만 (Stage 1 commit 위에 누적)
- `tests/hwp3_sample_parse.rs` — 신규 회귀 가드 테스트 (의도된 실패 상태)
- `mydocs/working/task_m100_929_stage2.md` — 본 보고서

## 검증 결과 예고 (Stage 2 시점)

- `cargo test --release --test hwp3_sample_parse task929_hwp3_sample19_parses_ok` → **현재 실패 (의도)**
- `cargo test --release --test hwp3_sample_parse task929_existing_hwp3_samples_no_regression` → **현재 통과 예상** (회귀 가드는 수정 전에도 통과해야 함)

Stage 3 에서 코드 수정 후 양쪽 모두 통과 예정.

## 리스크 검토

| 리스크 | 평가 | 대응 |
|--------|------|------|
| 기존 6 hwp3 샘플 회귀 | 중 — tab 처리 변경이 광범위 | 회귀 가드 테스트 필수, ir-diff 로 정량 비교 |
| 한컴 일부 버전이 tab 을 2-byte 로 저장한 변형 존재 | 낮음 — spec §10.5 명시적 | 회귀 발견 시 별 이슈 분리 |
| 다른 컨트롤 (ch=7,8 등) 도 유사 결함 가능 | 낮음 — 코드상 7,8 은 이미 6-byte 추가 처리됨 | Stage 3 검증 시 다른 ctrl trace 도 같이 확인 |

---

## 승인 요청

Stage 2 완료 보고드립니다. 다음 사항 승인 부탁드립니다:

1. **원인 진단 (ch=9 탭 spec §10.5 8-byte 미준수)** 의 정확성 확인.
2. **수정 설계 (코드 변경 + 회귀 가드 테스트 의도된 실패 상태 추가)** 의 적정성 확인.
3. **Stage 2 commit + Stage 3 진행** (실제 코드 수정 + 진단 제거 + 검증).

특별히 의견 없으시면 `승인` — 위 수정안 그대로 적용 + Stage 3 진행.
