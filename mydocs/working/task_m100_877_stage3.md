# Task #877 Stage 3 완료 보고서 — 시각 차이 4건 진단 + 로마숫자 매핑 추가

**관련 계획서**: [task_m100_877_impl.md](../plans/task_m100_877_impl.md)
**참조 spec**: [한글문서파일구조3.0.md](../tech/한글문서파일구조3.0.md)
**브랜치**: `local/task877_v2` (분기: `local/task873`)

## 배경

Stage 1+2 적용 후 rhwp-studio 에서 sample16 로드 시 panic 없음 + 65 페이지 인식. 그러나 한컴오피스와 시각 차이 4건 발견:

1. **표지 박스/외곽선 누락**: "서버통합 및 원격지 재해복구센터 구축 / 제안요청서(RFP)" 박스 미표시
2. **빈 페이지 2 (목차 페이지 어긋남)**: 한컴은 페이지 1 표지 → 페이지 2 목차. rhwp 는 페이지 1 표지 → 빈 페이지 2 → 페이지 3 목차
3. **로마숫자 prefix (Ⅰ, Ⅱ, Ⅲ ...) 누락**: 목차 본문 ". 사업개요", ". 제안 일반사항" — Ⅰ, Ⅱ 가 빠짐
4. **16페이지 다이어그램 미표시**: 주전산센터 목표시스템 구성안 다이어그램 안 보임

## 진단 결과

### 1. 표지 박스 (HWP3 ParaShape.border 비활성)
- probe 측정: `para[4]` (표지 제목) `border=0, border_conn=0, shade_ratio=0` → **HWP3 raw 에 박스 정보 없음**
- HWP5 변환본은 `border_fill_id=1` — 한컴 변환기가 picture (외곽선 그림) 를 paragraph 의 border_fill 로 변환한 결과
- 표지 박스의 실체는 paragraph 5 의 `picture (ch=11, pic_type=2 Embedded)` 인 외곽선 그림
- task873 base 에 이미 `pic_type==0` 조건 적용 → 외부 fetch 시도 차단됨 (E$$0001E.gif 404 차단)
- **본 task 범위 밖**: embedded image binary 가 IR `Picture.bin_data_id` 에서 base64 로 등록 + 렌더러에서 사용까지 별도 검증 필요

### 2. 빈 페이지 2 (pagination 차이)
- `paragraph 24` (페이지 2 의 유일한 paragraph) `vpos=72360` — paragraph stream 의 자체 LineSeg.vpos 값
- 한컴 viewer 의 페이지 break 알고리즘과 rhwp 의 vpos-based pagination 알고리즘이 sample16 의 표지 영역에서 다르게 동작
- HWP5 변환본은 정상 (paragraph 25 "목차" 가 page 2 시작) — 한컴 변환 시 paragraph stream 의 vpos/line_height 가 재조정됨
- **본 task 범위 밖**: HWP3 pagination 알고리즘은 sample16 외 다른 sample (sample10 26767 문단 등) 의 정합성도 함께 고려한 별도 task 가 필요

### 3. 로마숫자 (Ⅰ, Ⅱ, Ⅲ ...) 누락 **— 본 stage 에서 수정 완료**
- probe 측정: paragraph 26 첫 hchar = **0x3590** (HWP3 사적 인코딩)
- `decode_johab(0x3590)`:
  - 0x3590 < 0x8000 → 조합형 한글 아님
  - 0x3590 ≥ 0x0080 → `decode_hwp3_extra(0x3590)` 호출 → 매핑 부재 → `None`
  - 최종 '?' 반환 → ch ≥ 0x80 이므로 `continue` (skip) → IR text 에서 누락
- 다른 paragraph 의 첫 chars cross-ref:
  - para 26 = 0x3590 → Ⅰ (U+2160)
  - para 31 = 0x3591 → Ⅱ (U+2161)
  - para 36 = 0x3592 → Ⅲ (U+2162)
  - para 44 = 0x3593 → Ⅳ (U+2163)
- **패턴**: HWP3 사적 인코딩 `0x3590 + n` = Unicode `0x2160 + n` (Ⅰ~Ⅹ)

### 4. 16페이지 다이어그램 (drawing tree 렉더링)
- 사용자 확인: **HWP5 변환본 (`hwp3-sample16-hwp5.hwp`) 에서도 동일 증상** — drawing object tree (ch=11 pic_type=3 group of shapes) 가 rhwp 의 렉더러에서 일부만 표시
- 즉 HWP3/HWP5 파서 모두에서 IR 은 동일 표현이고, **렉더러 측 작업** 영역
- **본 task 범위 밖**: 별도 drawing tree 렉더링 task 로 분리

## 작업 내용 (수정 적용)

### decode_hwp3_extra 매핑 확장 ([src/parser/hwp3/johab.rs:64-71](../../src/parser/hwp3/johab.rs#L64-L71))

```rust
fn decode_hwp3_extra(ch: u16) -> Option<char> {
    // [Task #877 Stage 3] 로마숫자 대문자 Ⅰ~Ⅹ: 0x3590~0x3599 → U+2160~U+2169.
    if (0x3590..=0x3599).contains(&ch) {
        return char::from_u32(0x2160 + (ch - 0x3590) as u32);
    }
    // ... 기존 매핑 ...
}
```

## 검증 결과

### sample16 로마숫자 표시 (Stage 3 후)
| paragraph | Stage 2 만 | **Stage 3 후** |
|-----------|----------|--------------|
| 0.26 | ". 사업개요" | **"Ⅰ. 사업개요"** ✓ |
| 0.31 | ". 제안 일반사항" | **"Ⅱ. 제안 일반사항"** ✓ |
| 0.36 | ". 제안 요구사항" | **"Ⅲ. 제안 요구사항"** ✓ |
| 0.44 | ". 프로젝트 과업범위" | **"Ⅳ. 프로젝트 과업범위"** ✓ |
| 0.51 | ". 도입장비 내역서" | **"Ⅴ. 도입장비 내역서"** ✓ |
| 0.55 | ". 공사 정보화 현황" | **"Ⅵ. 공사 정보화 현황"** ✓ |
| 0.62 | ". 한국수자원공사 일반현황" | **"Ⅶ. 한국수자원공사 일반현황"** ✓ |

### 회귀 검증 (다른 HWP3 sample)
- `samples/hwp3-sample.hwp`: 195 문단 (변동 없음)
- `samples/hwp3-sample10.hwp`: 26767 문단 (변동 없음)
- `samples/hwp3-sample13.hwp`: 71 문단 (변동 없음)
- `samples/hwp3-sample14.hwp`: 256 문단 (변동 없음)
- `samples/hwp3-sample4.hwp`: 1273 문단 (변동 없음)
- `samples/hwp3-sample5.hwp`: 1931 문단 (변동 없음)

### cargo test
```
test result: ok. 1234 passed; 0 failed; 2 ignored (lib)
+ integration tests 36개 묶음 전부 ok
```

## 후속 별도 이슈 등록 (예정)

본 Stage 3 범위 밖이므로 별도 이슈로 분리:

1. **HWP3 표지 picture (외곽선 그림) embedded image 렉더링**
   - sample16 처럼 picture pic_type=2 (Embedded Image) 가 paragraph 의 외곽선/배경으로 사용되는 case
   - additional_info_blocks id=1 의 이미지 binary 가 IR Picture.bin_data 로 정확히 등록되는지 검증
   - 우선순위: 중

2. **HWP3 sample16 표지 다음 빈 페이지 2 — pagination 정합**
   - paragraph 24 vpos=72360 처리 — 한컴 viewer 의 page break 알고리즘 매칭
   - 우선순위: 중

3. **HWP3/HWP5 drawing object tree (ch=11 pic_type=3) 렌더링 정합**
   - sample16 페이지 16 의 다이어그램 — HWP5 변환본도 동일 미표시
   - 우선순위: 낮 (rhwp 전체 렉더링 영역)

4. **HWP3 글머리 ○ / paragraph bullet 처리**
   - sample16 본문의 "○ 업무특성 ..." prefix — HWP3 paragraph style/outline 처리
   - 우선순위: 낮

## 변경 파일

- `src/parser/hwp3/johab.rs` — `decode_hwp3_extra` 에 로마숫자 매핑 추가 (Ⅰ~Ⅹ)

## 다음 단계

- Stage 3 보고서 승인 → 최종 결과 보고서 작성 → task #877 완료
- 후속 별도 이슈 4건 등록 (별도 단계로 진행)
