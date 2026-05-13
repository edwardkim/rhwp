# Task #877 Stage 2 완료 보고서 — HWP3 special char alignment 정합 (ch=5/6/7/8)

**관련 계획서**: [task_m100_877_impl.md](../plans/task_m100_877_impl.md)
**참조 spec**: [한글문서파일구조3.0.md](../tech/한글문서파일구조3.0.md) §10.1~§10.4
**브랜치**: `local/task877`

## 진단 — 28737 페이지 폭주 원인

### 1차 단서 (Stage 1 이후)
Stage 1 가드 적용 후 sample16 panic 은 사라졌으나, dump 시 **1 구역 / 77 문단**, dump-pages **28737 페이지** 로 폭주 인식. paragraph 71 부근부터 garbage cc/lc (cc=2560, lc=2602 등) 출현.

### probe 분석
`/tmp/pic_probe/` 외부 probe binary 로 sample16 의 decompressed body byte stream 정밀 추적:
- paragraph 70 (`@31904`, raw cc=20) text body 에 `ch=6` (HWP3 책갈피) control 포함
- 현재 파서는 `ch=6` 을 미지 제어로 처리하여 **8 byte** 만 소비 (ch + dword + ch close)
- paragraph 70 끝 위치 32188 에서 paragraph 71 헤더 시도 → fp=73, cc=2560 (garbage)
- char_shape pattern `0906 0201 0101 6400 6464 6464 6464` (size + font_indices + ratios) 가 body offset **32236** 부터 등장 → para[71] 진짜 시작 = 32236 - 12 = **32224**
- 32224 - 32188 = **36 byte 추가 소비 필요** 확인

### spec 대조 (한글문서파일구조3.0 §10)

| ch | 의미 | spec 총 byte | 현재 파서 | 차이 |
|----|-----|------------|---------|------|
| 5 | 필드 코드 | 가변 (8+n) | 8 | n bytes 누락 |
| **6** | **책갈피** | **42** | 8 | **+34 누락** |
| **7** | **날짜 형식** | **84** | 8 | **+76 누락** |
| **8** | **날짜 코드** | **96** | 8 | **+88 누락** |
| 18~21 | 번호코드 등 | 8 | 8 | ✓ |
| 22 | 메일머지 | 24 | 24 | ✓ |
| 23 | 글자겹침 | 10 | 10 | ✓ |
| 24, 25 | 하이픈, 차례 | 6 | 6 | ✓ |
| 26 | 찾아보기 | 246 | 246 | ✓ |
| 28 | 개요 | 64 | 64 | ✓ |
| 29 | 상호참조 | 가변 | 가변 (가드) | ✓ |
| 30, 31 | 묶음/고정폭빈칸 | 4 | 4 | ✓ |

`ch=6/7/8/5` 만 spec 비정합. **sample16 의 핵심 트리거는 `ch=6 책갈피` (= "1. 추진목적" 등 본문 제목들에 부착된 책갈피)**.

### 검증
para[70] cc=20 의 hchar 구성 (spec 적용 후):
- 7 hchars text "1. 추진목적" → 14 bytes (32148-32161)
- 1 ch=6 책갈피 control: cc count += 4, 실제 42 bytes 소비 (32162-32203)
- 1 ch=19 새 번호: cc += 4, 8 bytes (32204-32211)
- 1 ch=20 쪽번호달기: cc += 4, 8 bytes (32212-32219)
- 1 ch=13 CR: cc += 1, 2 bytes (32220-32221)

합: 7+4+4+4+1 = 20 cc ✓, byte 32148→32222. para[71] @ **32222** valid header (fp=0, cc=5, lc=1) ✓

## 작업 내용

### 1. 책갈피 (ch=6) 처리 추가 ([src/parser/hwp3/mod.rs](../../src/parser/hwp3/mod.rs#L1149-L1169))

기존 `_ =>` else 분기에 책갈피 명세에 따른 34 byte 추가 소비 + `Control::Field("Bookmark:이름:type=종류")` 형식으로 IR 등록.

```rust
} else if ch == 6 {
    // 책갈피 spec §10.2 표 36: 42 bytes total
    let mut bookmark_extra = [0u8; 34];
    if let Err(_) = body_cursor.read_exact(&mut bookmark_extra) { break; }
    let name = decode_hwp3_string(&bookmark_extra[0..32]).trim_end_matches('\0').to_string();
    let bookmark_type = (&bookmark_extra[32..34]).read_u16::<LittleEndian>().unwrap_or(0);
    // → Control::Field { command: "Bookmark:{name}:type={type}", ... }
}
```

### 2. 날짜 형식/코드 (ch=7, ch=8) ([mod.rs:1183-1206](../../src/parser/hwp3/mod.rs#L1183))

spec 정합 byte 소비. sample16 은 사용 안 함이나 다른 HWP3 sample 회귀 대비.

### 3. 필드 코드 (ch=5) ([mod.rs:1140-1148](../../src/parser/hwp3/mod.rs#L1140))

가변 길이 (8 + n bytes). header_val1 = n. 추가 n bytes 소비. Stage 1 의 `alloc_record_buf` 가드 통해.

### 4. 단위 테스트 강화

`test_hwp3_sample16_load_alignment` — sample16 의 paragraph count >= 1000 검증 (Stage 1 만 적용 시 77, Stage 2 적용 후 1058).

## 검증 결과

### sample16 (이슈 #877 대상)
| 항목 | Stage 1 만 | Stage 2 적용 후 | 한컴 viewer | 한컴 HWP5 변환본 |
|------|----------|--------------|----------|----------------|
| 문단 수 | 77 | **1058** | - | 1058 |
| 페이지 수 | 28737 | **65** | 64 | 62 |

문단 수가 한컴 HWP5 변환본과 동일. 페이지 수 차이 (65 vs 64) 는 layout 미세 차이로 본 task 범위 밖.

### 회귀 검증 (다른 HWP3 sample)
| 샘플 | 문단 수 | 페이지 수 |
|------|--------|----------|
| hwp3-sample.hwp | 195 (변동 없음) | 16 |
| hwp3-sample10.hwp | 26767 (변동 없음) | 763 |
| hwp3-sample13.hwp | 71 (변동 없음) | 3 |
| hwp3-sample14.hwp | 256 (변동 없음) | 11 |
| hwp3-sample4.hwp | 1273 (변동 없음) | 36 |
| hwp3-sample5.hwp | 1931 (변동 없음) | 64 |

전부 Stage 1 직후 값과 동일. **회귀 없음** ✓

### cargo test
```
test result: ok. 1234 passed; 0 failed; 2 ignored (lib)
+ integration tests 36개 묶음 전부 ok
+ test_hwp3_sample16_load_alignment ✓
```

## 신규 파일 git 추가

작업지시자 요청 — sample16 관련 자료를 저장소에 영구 보존:
- `samples/hwp3-sample16.hwp` (2.94 MB) — HWP3 원본
- `samples/hwp3-sample16-hwp5.hwp` (3.04 MB) — 한컴 HWP5 변환본 (정합 기준)
- `samples/hwp3-sample16-hwp5.hwpx` (3.06 MB) — 한컴 HWPX 변환본
- `pdf/hwp3-sample16-hwp5-2022.pdf` (2.23 MB) — 한컴 2022 편집기 PDF 변환본 (시각 정답지)

모두 50 MB 미만이라 일반 git 영역에 보존 (LFS 불필요).

## 변경 파일

- `src/parser/hwp3/mod.rs`:
  - `ch=5` (필드 코드) 가변 길이 처리 추가
  - `ch=6` (책갈피) 42 byte 처리 추가 + Control::Field 로 IR 등록
  - `ch=7` (날짜 형식) 84 byte 처리 추가
  - `ch=8` (날짜 코드) 96 byte 처리 추가
  - `test_hwp3_sample16_load_alignment` 추가 (paragraph 수 1000 이상 검증)
- `samples/hwp3-sample16*.hwp{,x}` git 추가 (3 파일)
- `pdf/hwp3-sample16-hwp5-2022.pdf` git 추가

## 잔여 / 향후 작업

- **외부 image 404 (rhwp-studio 콘솔)**: sample16 의 pic_type=2 (Embedded Image) 가 `external_path` 로 잘못 설정되어 studio 가 `/samples/E$$0001E.gif` 등을 fetch 시도 → 404. 본 이슈는 PR #869 (task864) 의 `pic_type == 0` 조건 fix 가 들어와야 해결. **본 task 범위 밖**.
- **페이지 수 65 vs 64**: 1쪽 차이는 layout 의 미세한 줄 높이/줄간격 차이. **이슈 #877 의 1차 목표는 panic 차단 + 합리적 페이지 수 인식이며 이미 달성됨.**
- ch=2/3/4/12/27 등 spec 미정의 control 의 정확한 처리: 추후 발견 시 별도 수정.

## 다음 단계

Stage 3 (WASM panic hook + 통합 회귀 테스트) 진행.
