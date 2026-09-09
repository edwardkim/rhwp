# 최종 결과 보고서 — Task M100 #2844

## 이슈

HWP3 파서(`src/parser/hwp3/mod.rs`)의 문자 스캔 루프가 특수 문자 코드 **7(날짜 형식,
한글문서파일구조3.0 §10.3)** 과 **8(날짜 코드, §10.4)** 를, 총 길이가 8바이트인 다른
"단순 컨트롤" 코드들(9, 22, 23, 24, 25, 26, 28, 30, 31)과 같은 디스패치 버킷
(`parse_simple_control_char`)으로 잘못 라우팅. 실제로는 각각 84/96바이트짜리 구조체인데
6바이트만 읽고 스킵하여 76/88바이트가 다음 컨트롤·본문 텍스트로 잘못 흡수됨. 문단 전체가
바이트 오프셋 디싱크에 빠짐.

- Issue: https://github.com/edwardkim/rhwp/issues/2844
- PR: (본 보고서 하단 참조)

## 결론

문자 스캔 루프 디스패치 매치(`parse_paragraph_list` 내부, 舊 2089행)에서 `7 | 8` 을
제거하여 `_` 캐치올(`parse_object_control_char`)로 되돌렸다. `parse_object_control_char`에는
Task #877 에서 작성된 76/88바이트 스킵 로직이 이미 정확하게 존재했으나, 회귀 이후
dispatch가 도달시키지 않는 죽은 코드였다. 라우팅만 복원하면 되므로, `parse_simple_control_char`
안의 잘못된 `7 | 8 =>` (6바이트만 소비) arm은 이제 도달 불가능해져 삭제했다.

이는 추론이 아니라 **git blame 으로 확인된 회귀**다. `git log -L1777,1795:src/parser/hwp3/mod.rs`
로 추적한 결과, 커밋 `f1995532`("Task #2001: 추출 2 — 소형 컨트롤 코드 arm 헬퍼 2개
(field/simple, 동작 불변)")가 `1 | 7 | 8 | 9 | 22 | ...` 매치로 ch=7/8을
`parse_simple_control_char`로 우회시키면서 동시에 6바이트짜리 새 `7 | 8` arm을 만들었다.
그 결과 "동작 불변"을 표방한 리팩터가 실제로는 회귀를 도입했고, 원래 존재하던 76/88바이트
로직은 그대로 남았지만 죽은 코드가 되었다.

## 결함 상세

### 결함 위치 — `src/parser/hwp3/mod.rs`

디스패치 매치(`parse_paragraph_list` 내부):

```rust
1 | 7 | 8 | 9 | 22 | 23 | 24 | 25 | 26 | 28 | 30 | 31 => {
    let (next_i, next_utf16_len, break_char_loop) = parse_simple_control_char(...)
```

`parse_simple_control_char` 내부:

```rust
7 | 8 => {
    let mut buf = [0u8; 6];          // ← 84/96바이트 구조체인데 6바이트만 소비
    ...
    i += 3;
    ...
}
```

스펙(`mydocs/tech/한글문서파일구조3.0.md` §10.3/§10.4):

- ch=7(날짜 형식): 전체 84바이트 = 여는 코드(2, outer read 완료) + hchar array[40]
  날짜 형식 문자열(80) + 닫는 코드(2). 즉 outer read 이후 **82바이트** 추가 소비 필요.
- ch=8(날짜 코드): 전체 96바이트 = 여는 코드(2) + 날짜형식(80) + 날짜(8) + 시각(4) +
  닫는 코드(2). outer read 이후 **94바이트** 추가 소비 필요.

`parse_object_control_char`(舊 1257~1279행, Task #877)는 이미 정답을 갖고 있었다:

```rust
} else if ch == 7 {
    let mut date_fmt = [0u8; 76];   // header_val1(4)+ch2(2) 이미 소비 → 8+76=84 일치
    ...
} else if ch == 8 {
    let mut date_code = [0u8; 88];  // 8+88=96 일치
    ...
}
```

## 수정 내용

1. `parse_paragraph_list` 디스패치 매치에서 `1 | 7 | 8 | 9 | ...` → `1 | 9 | ...` (7, 8 제거).
   이제 ch=7/8은 `_` 캐치올을 통해 `parse_object_control_char`로 라우팅되어 Task #877의
   정확한 76/88바이트 스킵 로직이 실행된다.
2. `parse_simple_control_char`의 버그 있는 `7 | 8 => { ... 6바이트만 소비 ... }` arm 삭제
   (라우팅 수정으로 도달 불가능해진 죽은 코드).

## 테스트 (red → green)

`src/parser/hwp3/mod.rs` `mod tests`에 통합 테스트 1건 추가:
`task2844_hwp3_date_format_ctrl_does_not_swallow_following_text`

- 문단 1개: ch=7 날짜 형식 컨트롤(84바이트, 날짜 형식 문자열은 전부 0) 뒤에 실제 본문
  `"AAA"`(3 hchar)가 오는 최소 HWP3 문단 바이트열을 직접 구성.
- `parse_paragraph_list()` 호출 후 `paragraphs[0].text`가 `"AAA"`를 포함하는지 단언.

**Red (수정 전, 임시로 舊 로직 복원 후 실행):**
```
thread '...' panicked at src\parser\hwp3\mod.rs:4247:9:
날짜 형식(ch=7) 컨트롤 뒤의 "AAA" 본문이 유실됨 (바이트 언더리드로 흡수): "￼"
test result: FAILED. 0 passed; 1 failed
```
(84바이트 레코드 중 6바이트만 소비되어 커서가 78바이트 어긋나고, "AAA" 텍스트에
도달하지 못한 채 남은 0-바이트들을 종료 문단으로 오인해 파싱이 조기 종료됨.)

**Green (수정 후):**
```
test parser::hwp3::tests::task2844_hwp3_date_format_ctrl_does_not_swallow_following_text ... ok
test result: ok. 1 passed; 0 failed
```

## 검증

- `cargo build --lib`: 통과
- `cargo test --lib task2844_hwp3_date_format_ctrl_does_not_swallow_following_text`: 통과 (green)
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs`: 실질적 diff 없음 (개행 방식 경고만,
  실제 내용 변경 없음 — `git diff --stat`로 확인)

## 영향 범위

- 변경 파일: `src/parser/hwp3/mod.rs` 만 (스코프 최소화).
- ch=1, 9, 22, 23, 24, 25, 26, 28, 30, 31 은 실제로 총 8/6/10/24/246/64/4바이트로
  스펙과 정확히 일치함을 개별 재검산으로 확인, 손대지 않았다.
- 그 밖에 검토했으나 문제가 없었던 영역(스펙 대조로 balancing 확인, 수정 불필요):
  - `records.rs` `Hwp3InfoBlock`/`Hwp3AdditionalInfoBlock` (§3.4/§3.8 정보 블록)
  - 그림 정보 블록(§10.7, 348+n바이트) 및 하이퍼텍스트 확장(TagID 3) 관련 n_ext 처리
  - 표/텍스트박스/수식/버튼 (§10.6) 셀 정보 27바이트 × 셀개수 반복
  - 각주/미주(§10.11, 14바이트), 머리말/꼬리말(§10.10, 10바이트), 상호참조(§10.22,
    46+n바이트 가변 구조)
