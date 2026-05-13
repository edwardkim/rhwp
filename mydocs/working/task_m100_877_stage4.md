# Task #877 Stage 4 완료 보고서 — 잔여 3건 추가 처리

**관련 계획서**: [task_m100_877_impl.md](../plans/task_m100_877_impl.md)
**참조 spec**: [한글문서파일구조3.0.md](../tech/한글문서파일구조3.0.md), [한글문서파일형식_5.0_revision1.3.md](../tech/한글문서파일형식_5.0_revision1.3.md)
**브랜치**: `local/task877_v2`

## 배경

Stage 3 후 sample16 의 시각 차이 4건 중 1건 (로마숫자) + 2건 (외곽선, 빈 페이지) 처리. 잔여 3건 (◦ 글머리 / 본문 박스 외곽선 / 16쪽 다이어그램) Stage 4 에서 시도.

## 진단 및 수정

### ✅ 1. ◦ 글머리 — **휴리스틱 도입 완료**

**진단**:
- HWP5 spec §표 43 은 paragraph 의 `Bullet ID` 필드 보유
- HWP3 spec §5 paragraph 모양은 `Bullet ID` 필드 부재. 한컴 변환기가 paragraph margins 패턴 분석하여 ◦ 자동 추가하는 휴리스틱.
- sample16 의 paragraph 91/100/110 raw 첫 char = ' ' (공백). HWP3 raw 에 ◦ 정보 자체 부재.

**휴리스틱**: HWP3 ParaShape (L=6500, R=1000, I=-2500, ls=130) + 첫 char 공백 → paragraph text 첫 공백 다음에 "◦ " insert.

**회귀 검증**: 다른 HWP3 sample (sample/sample10/sample14) 에서 패턴 매치 paragraph **0개** → 회귀 안전.

**구현** ([src/parser/hwp3/mod.rs](../../src/parser/hwp3/mod.rs)): `fixup_hwp3_outline_bullets` 후처리 함수 추가. char_count / char_shapes.start_pos 동기화 갱신.

**결과**: sample16 의 25개 paragraph 에 ◦ 자동 prefix. paragraph 91 = `" ◦ 주요업무에 대한 고가용성의 클러스터링(Clustering) 기술 도입"`.

### ❌ 2. 본문 박스 외곽선 — **본 task 범위 외 (별도 task)**

**진단**:
- HWP5 spec §표 43 은 paragraph 의 `BorderFill ID` 필드 보유
- HWP3 spec §5 paragraph 모양 (offset 181) 의 `테두리` 는 1 byte boolean. sample16 의 paragraph 89/91 = `border=0` (raw 정보 부재)
- HWP5 변환본의 1058 paragraph 모두 `border_fill_id > 0` — 한컴 변환기가 default 값 (line_type=0 "선 없음") 부여 + 일부 paragraph 만 실제 시각 외곽선

**판단**: paragraph 별 border_fill 자동 부여 휴리스틱은 광범위 (한컴 viewer 의 HWP3 표시 알고리즘 reverse-engineering 필요) → 별도 task 분리.

### ✅ 3. 16쪽 다이어그램 — **수정 완료**

**진단**:
- 사용자 확인: HWP5 변환본도 동일 미표시 → rhwp 렉더러 영역 추정
- 그러나 추가 분석 결과 **HWP3 파서의 image format detection 누락**
- paragraph 394 의 picture (bin_id=3, 161mm × 109mm) 의 binary data magic = `01 00 09 00 00 03` = **WMF (Windows Metafile)**
- HWP3 파서 [src/parser/hwp3/mod.rs:2198](../../src/parser/hwp3/mod.rs#L2198) 의 image format magic 검사가 JPG / PNG / GIF / BMP 만 지원. WMF / EMF 누락 → `ext="bin"` 으로 저장 → rhwp 렉더러가 미지원으로 처리

**수정**:
```rust
} else if img_data.starts_with(b"\xD7\xCD\xC6\x9A")
    || img_data.starts_with(b"\x01\x00\x09\x00")
{
    "wmf"
} else if img_data.len() >= 44
    && img_data.starts_with(b"\x01\x00\x00\x00")
    && &img_data[40..44] == b" EMF"
{
    "emf"
}
```

**결과**: sample16 의 bin_data id=3/5/7 의 image ext = "bin" → **"wmf"**. rhwp 의 `src/wmf/converter/svg` 모듈이 WMF → SVG 변환하여 다이어그램 표시.

## 최종 sample16 결과

| 항목 | Stage 4 후 |
|------|----------|
| Panic | ✅ 없음 |
| 문단 수 | 1058 (한컴 정확 일치) |
| 페이지 수 | 64 (한컴 정확 일치) |
| 페이지 2 = 목차 | ✅ |
| 표지 RFP 박스 외곽선 | ✅ |
| Ⅰ~Ⅹ 로마숫자 | ✅ |
| paragraph 89/92 글머리 `󰏅` | ✅ |
| paragraph 91/100/110... ◦ 글머리 | ✅ (Stage 4 신규) |
| **16쪽 다이어그램 (WMF)** | ✅ (Stage 4 신규) |
| 본문 박스 외곽선 (paragraph border_fill) | ❌ 별도 task |

## 검증

### cargo test
```
total: passed: 1381, failed: 0
```

### 다른 HWP3 sample 회귀 없음
- hwp3-sample.hwp: 195 문단
- hwp3-sample10.hwp: 26767 문단
- hwp3-sample14.hwp: 256 문단

## 변경 파일

- `src/parser/hwp3/mod.rs`:
  - `fixup_hwp3_outline_bullets` 함수 추가 (◦ 자동 prefix 휴리스틱) — 커밋 b647227
  - additional_info_blocks image magic 검사에 WMF / EMF 추가 — 커밋 5b70dfc

## 후속 별도 이슈 (1건)

- **HWP3 paragraph border_fill 자동 부여 휴리스틱**: HWP3 raw 의 paragraph margins/style 패턴 분석하여 본문 영역의 paragraph border 자동 부여. 광범위 reverse-engineering 작업.

## 다음 단계

Stage 4 최종 + 최종 결과 보고서 → task #877 종료.
