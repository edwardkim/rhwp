# Task #528 Stage 2 — 매핑 표 확보 + 변환 함수 구현

**작성일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528)
**브랜치**: `local/task528`

## 1. 결론

> **KTUG HanyangPuaTableProject** (Public Domain) 데이터를 채택. 5,660 매핑 / exam_kor p17 BMP PUA 25/25 (100%) 커버. `src/renderer/pua_oldhangul.rs` 자동 생성 모듈 + 5개 단위 테스트 통과.

## 2. 자료원 조사

### 2-1. 검토 대상

| 자료원 | 결과 |
|--------|------|
| `pyhwp` (mete0r) | `decode_utf16le_with_hypua` 함수명만 존재, 실제 PUA 변환 로직 없음 (0.1b12 에서 `hypua2jamo` 의존성 제거됨) |
| `hwp.js` (hahnlee) | PUA 옛한글 매핑 미발견 |
| **`hypua2jamo`** (mete0r) | **★ 발견** — Hanyang-PUA → Hangul Jamo 변환 패키지 (LGPL-3.0). 데이터 파일 자체는 Public Domain |

### 2-2. hypua2jamo 데이터 출처

`hypua2jamo/data/hypua2jamocomposed.txt` 헤더:

```
%%% Result of http://faq.ktug.or.kr/mywiki/HanyangPuaTableProject
%%% by Korean TeX Users Group (KTUG).
%%% Many thanks to many voluntary participants in the project.
%%% We do not believe that simple factual data can be copyrighted.
%%% This file is in Public Domain.
```

→ **KTUG (Korean TeX Users Group) HanyangPuaTableProject** — 다년 커뮤니티 검증. **Public Domain** 명시.

### 2-3. 데이터 통계

| 항목 | 값 |
|------|-----|
| 매핑 수 | **5,660** |
| 입력 영역 | U+E0BC ~ U+F8F7 (BMP PUA) |
| 출력 자모 | Hangul Jamo (U+1100-11FF: 256 cp) + Ext-A (U+A960-A97F: 29 cp) + Ext-B (U+D7B0-D7FF: 72 cp) |
| 출력 길이 | 1자모: 361 / 2자모: 1,410 / 3자모: 3,884 / 4자모: 1 / 6자모: 4 |
| 검증 이력 | 2004 ~ 2011 (KTUG, 함초롬바탕 정합) |

자세한 내용: [mydocs/tech/pua_oldhangul_mapping_sources.md](../tech/pua_oldhangul_mapping_sources.md)

## 3. exam_kor p17 커버리지 검증

### 3-1. 측정 코드

```python
# Stage 1 측정 결과 — 25 unique BMP PUA codepoints
exam_kor_pua = [0xE17A, 0xE1A7, ..., 0xF537]

# KTUG 매핑 적용
for cp in exam_kor_pua:
    result = mapping.get(cp)
    print(f"U+{cp:04X} => {result}")
```

### 3-2. 결과

```
✓ U+E17A => U+1100 U+1173 U+11DF (그ᇟ)
✓ U+E1A7 => U+1100 U+119E       (ᄀᆞ)
✓ U+E1C2 => U+1100 U+119E U+11EB (ᄀᆞᇫ)
✓ U+E288 => U+1102 U+119E U+11AF (ᄂᆞᆯ)
✓ U+E38A => U+1103 U+119E       (ᄃᆞ)
✓ U+E40A => U+1105 U+1161 U+11EB (라ᇫ)
✓ U+E474 => U+1105 U+119E U+11D7 (ᄅᆞᇗ)
✓ U+E560 => U+1106 U+119E       (ᄆᆞ)
✓ U+E566 => U+1106 U+119E U+11AF (ᄆᆞᆯ)
✓ U+E79C => U+1122 U+1166       (ᄢᅦ)
✓ U+E8A7 => U+112B U+119E       (ᄫᆞ)
✓ U+E8B2 => U+112B U+11A1       (ᄫᆡ)
✓ U+E95B => U+1109 U+1173 U+11F0 (스ᇰ)
✓ U+EB66 => U+1132 U+116E       (ᄲᅮ)
✓ U+EB68 => U+1132 U+116E U+11AB (ᄲᅮᆫ)
✓ U+EBD4 => U+110A U+119E       (ᄊᆞ)
✓ U+ECF0 => U+1140 U+1161       (ᅀᅡ)
✓ U+ECFB => U+1140 U+1162       (ᅀᅢ)
✓ U+ED41 => U+1140 U+116E       (ᅀᅮ)
✓ U+ED98 => U+1140 U+119E U+11B7 (ᅀᆞᆷ)
✓ U+ED9A => U+1140 U+119E U+11B8 (ᅀᆞᆸ)
✓ U+F152 => U+114C U+1174       (ᅌᅴ)
✓ U+F154 => U+114C U+1175       (ᅌᅵ)
✓ U+F1C4 => U+110C U+1172 U+11F0 (쥬ᇰ)
✓ U+F537 => U+1112 U+119E       (ᄒᆞ)
```

→ **25/25 (100%) 커버리지** ★

### 3-3. 비-옛한글 Supp PUA-A (별도 처리)

| 코드 | 빈도 | 추정 용도 | 처리 |
|------|------|----------|------|
| U+F0854 | 33 | 책괄호 시작 (`󰡔` → `《` 추정) | 본 task 미커버 (별도 issue) |
| U+F0855 | 33 | 책괄호 끝 (`󰡕` → `》` 추정) | 본 task 미커버 (별도 issue) |
| U+F00DA | 2 | 괄호류 (`(<U+F00DA> 단풍 철`) | 본 task 미커버 (별도 issue) |

→ 후속 task 분리 권장.

## 4. 코드 산출물

### 4-1. 자동 생성 스크립트

`scripts/gen_pua_oldhangul_rs.py` — KTUG 데이터 → Rust 모듈 변환.

```bash
python3 scripts/gen_pua_oldhangul_rs.py /path/to/hypua2jamocomposed.txt > src/renderer/pua_oldhangul.rs
```

### 4-2. 생성된 모듈

`src/renderer/pua_oldhangul.rs` (5,773 라인 — 5,660 매핑 + 헤더 + 테스트):

```rust
pub fn map_pua_old_hangul(ch: char) -> Option<&'static [char]> {
    let cp = ch as u32;
    PUA_OLDHANGUL_MAP
        .binary_search_by_key(&cp, |&(k, _)| k)
        .ok()
        .map(|idx| PUA_OLDHANGUL_MAP[idx].1)
}

pub fn is_pua_old_hangul(ch: char) -> bool {
    map_pua_old_hangul(ch).is_some()
}
```

### 4-3. 모듈 등록

`src/renderer/mod.rs:21`:

```rust
pub mod pua_oldhangul;
```

## 5. 단위 테스트

```
running 5 tests
test renderer::pua_oldhangul::tests::test_map_size ... ok
test renderer::pua_oldhangul::tests::test_known_mapping_sample ... ok
test renderer::pua_oldhangul::tests::test_exam_kor_p17_coverage ... ok
test renderer::pua_oldhangul::tests::test_no_collision_with_pua_bullet_range ... ok
test renderer::pua_oldhangul::tests::test_map_sorted ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out
```

검증 항목:
1. **test_map_size** — 5,660 entries 확인
2. **test_map_sorted** — 이진 검색용 정렬 확인
3. **test_exam_kor_p17_coverage** — exam_kor 25 코드 100% 커버
4. **test_known_mapping_sample** — U+E1A7 => [U+1100, U+119E] 정합
5. **test_no_collision_with_pua_bullet_range** — Task #509 bullet 영역 충돌 없음

## 6. 산출물 요약

| 산출물 | 위치 | 용도 |
|--------|------|------|
| 자동 생성 스크립트 | `scripts/gen_pua_oldhangul_rs.py` | KTUG 데이터 → Rust 변환 |
| Rust 모듈 | `src/renderer/pua_oldhangul.rs` | 5,660 매핑 + API |
| 모듈 등록 | `src/renderer/mod.rs:21` | `pub mod pua_oldhangul;` |
| 자료원 정리 | `mydocs/tech/pua_oldhangul_mapping_sources.md` | 라이선스 + 검증 + 갱신 절차 |
| 본 보고서 | `mydocs/working/task_m100_528_stage2.md` | (현재) |

## 7. Stage 2 미수행 영역

구현계획서 Stage 2 의 일부 항목은 본 단계에서 불필요하다고 판단:

- **`gen-pua` 도구 보강 (gen-pua-oldhangul)** — KTUG 매핑이 이미 다년 커뮤니티 검증을 거쳤고 exam_kor 25/25 커버 → 추가 검증 도구 불필요. 한컴 PDF 와의 시각 비교는 Stage 5 (시각 판정) 에서 일괄 수행
- **KS X 1026-1:2007 부속서 직접 확인** — KTUG 데이터가 한/글 함초롬바탕 (HCR) 글리프 정합으로 검증된 점이 더 실용적

## 8. 다음 단계

작업지시자 승인 후 Stage 3 (Composer 단계 변환 적용):

1. `src/renderer/composer.rs` 의 `compose_paragraph` 에 `convert_pua_old_hangul` 호출 추가
2. 1:N 변환의 LINE_SEG 영향 분석 (옵션 A 원본 보존 vs B IR 치환)
3. Task #122 자모 클러스터 폭 계산 인프라 활용
4. 단위 + integration 테스트
5. 7 샘플 byte 비교 회귀

## 9. 회귀 영향 (Stage 2 자체)

| 영역 | 영향 |
|------|------|
| 코드 동작 | **0** — 모듈 추가만 (호출처 없음) |
| 빌드 | 정상 (cargo test --lib pua_oldhangul 5/5 통과) |
| 단위 테스트 전체 | 1112 filtered out 외 통과 (lib test 영향 없음) |

## 10. 승인 게이트

- [x] 매핑 표 자료원 확인 (KTUG, Public Domain)
- [x] 라이선스 정합성 확인
- [x] exam_kor p17 100% 커버 검증
- [x] Task #509 영역 충돌 없음 검증
- [x] 단위 테스트 통과
- [x] 자동 생성 스크립트 (재생성 가능)
- [x] 자료원 / 출처 문서화
