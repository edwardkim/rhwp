---
issue: 929
stage: 1
status: 완료 — 승인 대기
---

# Task #929 Stage 1 완료 보고서 — 실패 지점 특정

## 작업 내용

`src/parser/hwp3/mod.rs` 의 `parse_paragraph_list` 안 `?`-전파 read 지점에 임시 진단 출력(`eprintln!` with `[diag929]` 태그)을 추가하고 `samples/hwp3-sample19.hwp` 를 dump 실행하여 EOF 발생 위치를 좁혔다.

진단 출력 추가 지점 (Stage 3 에서 제거 예정):
- `parse_paragraph_list` loop 시작 (para_idx, pos, body_len)
- `Hwp3ParaInfo::read` 결과 (char_count, line_count, include_char_shape, flags, special_char_flags)
- `Hwp3LineInfo::read` 실패 시 (li_idx, pos_before, cur_pos)
- 본문 `read_u16` 실패 시 (char_idx, ch_pos) + 컨트롤 ch 트레이스
- `ch == 17` (각주/미주) 진입 시 cursor + 주변 64 bytes hex peek

## 실패 흐름 (재현)

```
[diag929] loop iter para_idx=21 pos=7709 body_len=12903
[diag929]   ParaInfo ok char_count=24 line_count=1 include_char_shape=0 ...
[diag929]   ctrl ch=17 pos=7776 para_char_idx=5/24
[diag929] ENTER ch=17 ch_pos=7776 cursor=7784 info_buf_size=14 peek(7776..7848): ...
[diag929]   ch=17 after info_buf cursor=7798 (consumed 14, recurse below)
[diag929] loop iter para_idx=0 pos=7798 body_len=12903
[diag929]   ParaInfo ok char_count=57762 line_count=33196 include_char_shape=183 ...   ← GARBAGE
[diag929] FAIL LineInfo::read para_idx=0 li_idx=361/33196 pos_before=12895 cur_pos=12903
        err=Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
```

## 실패 지점

- **함수**: `parse_paragraph_list` 의 **재귀 호출 (sub-paragraph list)**, ch=17 (각주/미주) 컨트롤 처리에서 들어감.
- **위치**: `Hwp3LineInfo::read` 에서 33,196 개의 LineInfo 를 읽으려다 361 번째에서 EOF.
- **근본 원인**: 재귀 진입 시 첫 `Hwp3ParaInfo::read` 가 **garbage 를 정상 ParaInfo 로 잘못 해석** → 비현실적인 line_count=33,196 으로 진행 → EOF.

## Cursor 정합성 분석

ch_pos=7776 (ch=17 의 ch 위치) 직후의 raw bytes (압축 해제된 body_data 기준):

```
offset:  7776                     7782          7784 (info_buf start)
bytes:   11 00 | 00 00 09 00     | 09 00      | a9 01 00 00 09 00 09 00 a9 01 00 00 09 00
         ↑ ch=17  ↑ header_val1   ↑ ch2         ↑ info_buf (14 bytes)

offset:  7798 (recursive parse_paragraph_list 시작 위치)
bytes:   85 a2 e1 ac 81 b7 | 20 00 55 00 52 00 4c 00 | 0d 00
         ↑ johab(?) 6 bytes  ↑ ' ' 'U' 'R' 'L' (8 bytes)  ↑ CR (HWP3 문단 종결)

offset:  7814 (그 다음, 정상 ParaInfo 패턴)
bytes:   01 19 00 01 00 00 00 00 02 00 00 00 ed 00 ...
         ↑ follow=1 char_count=0x0019=25 line_count=0x0001 include_char_shape=0 flags=0
           special_char_flags=0x00000200 style_index=0  → 명백한 정상 ParaInfo 헤더
```

## 가설

**7798~7813 사이 15 bytes 는 ParaInfo 가 아니라 "각주/미주 자체의 인라인 텍스트 또는 추가 헤더"** 이다.

- `e1 ac 81 b7`: 한글(johab) 추정 — 각주 번호 표식 또는 라벨
- ` URL` + CR(0x000d): 각주 인라인 텍스트 (각주 내용으로 "URL" 이 등장)
- 그리고 7814 부터가 실제 nested paragraph list 의 첫 ParaInfo (follow=1, char_count=25, line_count=1).

추정 결함:
- (a) ch=17 의 `info_buf` 14 bytes 가 실제 한컴 사양과 다름 — 헤더 크기가 더 크거나, 가변 길이일 가능성
- (b) ch=17 처리 시 인라인 텍스트(각주 본문)를 별도로 소비해야 하는데 누락
- (c) HWP3 spec 에서 각주/미주는 nested paragraph list 자체가 없고, ch=17 직후 인라인 hchar 로 본문이 직접 들어 있을 가능성 (→ `parse_paragraph_list` 재귀 호출 자체가 잘못)

가장 유력: **(c)** — ch=17 의 14 byte info_buf 다음에 0x000d (CR) 종결까지 hchar 시퀀스로 각주 본문이 인라인되고, 그 후엔 outer paragraph 의 다음 char 로 돌아가야 함. 현재 코드는 정확히 sub-paragraph_list 재귀 진입 — 이건 ch=15/16 (숨은설명/머리말꼬리말) 패턴과 동일하게 처리 중. 각주/미주는 다른 구조일 수 있음.

cf. ch=15 (숨은설명), ch=16 (머리말/꼬리말) 도 동일하게 `parse_paragraph_list` 재귀 호출. 그 경우들은 다른 hwp3 샘플(sample17 등)에서 정상 동작했을 가능성이 큰데, ch=17 만 잘못 모방한 가능성이 있다.

## 다음 단계 (Stage 2 제안)

1. **한컴 HWP3 spec 또는 pyhwp / hwp.js 등 reference parser** 에서 각주/미주(footnote/endnote) 의 byte layout 확인.
2. 위 가설 (a)/(b)/(c) 중 어느 것이 맞는지 확정 + 수정 설계.
3. 회귀 가드 단위 테스트 (의도된 실패 상태) 추가:
   - hwp3-sample19 의 `parse_hwp3` 가 Ok 를 반환해야 함
   - 기존 hwp3 샘플 (sample, sample10, sample11, sample13, sample14, sample16) 회귀 없음
4. Stage 1 의 임시 진단 출력은 **Stage 3 마무리 시 모두 제거** — Stage 1 은 임시 커밋 1개 (`Task #929 Stage 1: 진단 출력 추가`) 로 남김.

## 변경 파일

- `src/parser/hwp3/mod.rs` — 진단 출력 추가 (4 지점, Stage 3 에서 제거)

## 검증 (재현 가능 명령)

```bash
cargo build --release --bin rhwp
./target/release/rhwp dump samples/hwp3-sample19.hwp 2> /tmp/diag929.log
tail -25 /tmp/diag929.log
```

---

## 승인 요청

Stage 1 완료 보고드립니다. 다음 사항 승인 부탁드립니다:

1. **Stage 1 진단 출력의 임시 커밋** 진행 (브랜치 히스토리에 1개 커밋 남음, Stage 3 에서 진단 코드 제거 커밋으로 정리).
2. **Stage 2 진행** — 한컴 사양/reference parser 조사 + 수정 설계 + 단위 테스트 작성.

특히 Stage 2 의 reference parser 조사 범위에 대해 의견이 있으시면 `피드백: …` 으로 지시 부탁드립니다.
