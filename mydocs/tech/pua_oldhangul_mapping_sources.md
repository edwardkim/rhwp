# Hanyang-PUA 옛한글 매핑 표 — 자료원 정리

**작성일**: 2026-05-02
**관련 task**: [#528](https://github.com/edwardkim/rhwp/issues/528)

## 1. 본질

한컴 (한/글) 의 옛한글 인코딩은 두 단계 진화를 거쳤다:

| 한컴 버전 | 옛한글 인코딩 | 영역 |
|----------|---------------|------|
| 한/글 2010 이전 | **Hanyang-PUA** (한양 PUA) | U+E0BC ~ U+F8F7 (BMP PUA) |
| 한/글 2010 이후 | **KS X 1026-1:2007** (첫가끝 코드) | U+1100-11FF + U+A960-A97F + U+D7B0-D7FF |

기존 HWP 문서는 PUA 영역을 보존하므로 변환 처리가 필요. 한컴 자체 변환 도구 `HncPUAConverter.exe` 가 이 역할을 수행.

## 2. 자료원 비교

| 자료원 | 라이선스 | 매핑 수 | 검증 이력 | 채택 |
|--------|---------|--------|----------|------|
| **KTUG HanyangPuaTableProject** | **Public Domain** | 5,660 | 2004 ~ 2011 (다년 커뮤니티 검증, 함초롬바탕 기반 fix) | ✅ |
| KS X 1026-1:2007 부속서 | 한국산업표준 (유료) | (표준 매핑) | 표준 정의 | (보강 ref) |
| HncPUAConverter.exe | 한컴 EULA | (전체) | 한컴 공식 | (라이선스 부적합) |
| 함초롬바탕 LVT cmap | 한컴 폰트 라이선스 | (PUA 글리프) | (라이선스 부적합) |
| 기타 OSS (`pyhwp` 등) | 다양 | 부분 / 미구현 | 한정적 | (참고만) |

## 3. KTUG HanyangPuaTableProject 상세

### 3-1. 출처

- 주소: http://faq.ktug.or.kr/mywiki/HanyangPuaTableProject
- 단체: Korean TeX Users Group (KTUG)
- 작업: 다년 자원봉사자 협업으로 작성/검증
- 라이선스: 파일 헤더 명시 — *"We do not believe that simple factual data can be copyrighted. This file is in Public Domain."*

### 3-2. 데이터 형식 (`hypua2jamocomposed.txt`)

```
U+E0BC => U+115F U+1161 U+11AE
U+E0BD => U+115F U+1161 U+D7CD
U+E0BE => U+115F U+1161 U+11AF
...
```

- 5,660 entries (U+E0BC ~ U+F8F7 범위)
- 각 PUA 코드 → 1-6 자모 시퀀스 (대부분 3자모 = 초성+중성+종성)
- "composed" 변형: Unicode 5.2 의 Hangul Jamo Extended-B (U+D7B0-D7FF) 활용
- "decomposed" 변형: 기본 Hangul Jamo (U+1100-11FF) 만 사용 (구버전 호환)

### 3-3. 매핑 통계

| 출력 영역 | 활용 코드포인트 수 |
|----------|------|
| Hangul Jamo (U+1100-11FF) | 256 |
| Hangul Jamo Extended-A (U+A960-A97F) | 29 |
| Hangul Jamo Extended-B (U+D7B0-D7FF) | 72 |

| 출력 자모 길이 | 매핑 수 |
|---------------|--------|
| 1 자모 | 361 |
| 2 자모 | 1,410 |
| **3 자모** | **3,884** (대부분) |
| 4 자모 | 1 |
| 6 자모 | 4 |

### 3-4. 변경 이력 (원본 헤더 발췌)

```
2004/09/17  initial release
2010/02/06  fix according to unicode 5.2
2010/02/08  fix U+E945 U+E95C U+EA60 U+F4C7 according to HCR font
2010/02/11  fix years-old bug U+E379
2010/02/21  fix U+E230 U+E231 U+E232 according to HCR font
2010-2011   ... (다수 수정, 한컴 PDF 검증 영역)
```

→ **함초롬바탕 (HCR) 글리프 정합** 으로 검증되어 한컴 출력과의 정합도 높음.

## 4. exam_kor.hwp 검증

### 4-1. exam_kor p17 측정 결과 (Stage 1 보고서 §2-3)

25 unique BMP PUA codepoints 발견:

```
0xE17A, 0xE1A7, 0xE1C2, 0xE288, 0xE38A, 0xE40A, 0xE474,
0xE560, 0xE566, 0xE79C, 0xE8A7, 0xE8B2, 0xE95B,
0xEB66, 0xEB68, 0xEBD4, 0xECF0, 0xECFB,
0xED41, 0xED98, 0xED9A, 0xF152, 0xF154, 0xF1C4, 0xF537
```

### 4-2. KTUG 매핑 적용 결과

```
U+E17A => U+1100 U+1173 U+11DF (그ᇟ)
U+E1A7 => U+1100 U+119E       (ᄀᆞ)
U+E1C2 => U+1100 U+119E U+11EB (ᄀᆞᇫ)
U+E288 => U+1102 U+119E U+11AF (ᄂᆞᆯ)
U+E38A => U+1103 U+119E       (ᄃᆞ)
... (전체 25개 모두 매핑 존재)
```

→ **25/25 (100%) 커버리지**.

## 5. 비-옛한글 PUA 영역 (별도 처리 필요)

exam_kor p17 에서 발견된 Supplementary PUA-A 코드 (3종) 는 옛한글이 **아닌** 별도 기호:

| 코드 | 빈도 | 추정 용도 | 처리 영역 |
|------|------|----------|---------|
| U+F0854 | 33 | 책괄호 시작 (`󰡔`) — `《` 또는 `〔` | 본 task 미커버 (별도 issue) |
| U+F0855 | 33 | 책괄호 끝 (`󰡕`) — `》` 또는 `〕` | 본 task 미커버 (별도 issue) |
| U+F00DA | 2 | 괄호류 시작 (`(<U+F00DA> 단풍 철`) | 본 task 미커버 (별도 issue) |

→ Task #509 의 PUA 기호 매핑 영역 정합. 본 task 의 옛한글 변환과 분리하여 후속 task 에서 처리.

## 6. 본 프로젝트 적용

### 6-1. 코드 위치

- 데이터 + 변환 함수: `src/renderer/pua_oldhangul.rs` (자동 생성)
- 생성 스크립트: `scripts/gen_pua_oldhangul_rs.py`
- 데이터 갱신 절차:
  ```bash
  curl -O https://raw.githubusercontent.com/mete0r/hypua2jamo/master/data/hypua2jamocomposed.txt
  python3 scripts/gen_pua_oldhangul_rs.py hypua2jamocomposed.txt > src/renderer/pua_oldhangul.rs
  cargo test --lib pua_oldhangul
  ```

### 6-2. API

```rust
pub fn map_pua_old_hangul(ch: char) -> Option<&'static [char]>
pub fn is_pua_old_hangul(ch: char) -> bool
```

### 6-3. 검증

- 단위 테스트 5건 (자체 정합 + exam_kor p17 커버리지 + Task #509 영역 충돌 없음)
- Composer 단계 통합 시 광범위 회귀 검증 필수 (Task #528 Stage 3-5)

## 7. 라이선스 / 어트리뷰션

본 프로젝트는 KTUG 데이터의 Public Domain 선언을 신뢰하여 매핑 표를 코드화. 어트리뷰션은 다음 위치에 명시:

- `src/renderer/pua_oldhangul.rs` 헤더 주석
- 본 문서 (`mydocs/tech/pua_oldhangul_mapping_sources.md`)
- 향후 README 의 Acknowledgements 섹션 (선택)

## 8. 참고 자료

- KTUG HanyangPuaTableProject: http://faq.ktug.or.kr/mywiki/HanyangPuaTableProject
- hypua2jamo 패키지 (LGPL, 데이터는 PD): https://github.com/mete0r/hypua2jamo
- 한컴 매뉴얼 hncpuaconverter.htm: `mydocs/manual/hwp/Help/extracted/hwpbase/hncpuaconverter.htm`
- KS X 1026-1:2007 (한국산업표준 — 유료)
- Unicode Hangul Jamo (U+1100-11FF) 표준
- Unicode 5.2 Hangul Jamo Extended-A/B (U+A960-A97F, U+D7B0-D7FF) 표준
