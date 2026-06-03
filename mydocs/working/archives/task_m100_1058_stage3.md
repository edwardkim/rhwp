# Task #1058 Stage 3 보고서 — 회귀 가드 + 광범위 sweep + WASM

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058)
- 단계: Stage 3
- 일시: 2026-05-21

## 1. 결과 요약

회귀 가드 4 추가 (4/4 통과) + Task #1050 회귀 가드 7/7 양립 + 14 fixture 광범위 sweep diff=0 (회귀 부재) + WASM Docker 빌드 4.91 MB + rhwp-studio 동기화.

## 2. 회귀 가드

`tests/issue_1058_textbox_list_header.rs` (4 tests):

| 테스트 | 검증 |
|--------|------|
| `issue_1058_textbox_list_header_size_33` | HWPX → HWP 저장 시 글상자 LIST_HEADER size=33 |
| `issue_1058_hwp_textbox_roundtrip` | HWP 출처 (table-in-tbox) 글상자 LIST_HEADER 33 유지 (회귀 부재) |
| `issue_1058_footnote_list_header_size_16_preserved` | Task #1050 의 footnote LIST_HEADER size=16 양립 |
| `issue_1058_textbox_list_header_byte_contract` | byte-by-byte 정합 (offset 20..28 zero, 28..32 editable=0, 32 flag=0) |

→ **4/4 통과**.

## 3. Task #1050 회귀 가드 양립

```bash
cargo test --release --test issue_1050_footnote_serialize --test issue_1058_textbox_list_header
# 7 + 4 = 11 tests passed
```

## 4. CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1323 passed** (Task #1050 1319 + 회귀 4) |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

## 5. 광범위 sweep (14 fixtures, 1143 SVG)

작업지시자 선택 B (변환본 포함 광범위):

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|-------|
| samples/hwpx/footnote-tbox-01.hwpx | 1 | 0 |
| samples/footnote-tbox-01.hwp | 1 | 0 |
| samples/hwpx/footnote-01.hwpx | 6 | 0 |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/hwp3-sample-hwp5.hwp | 16 | 0 |
| samples/hwp3-sample10-hwp5.hwp | 763 | 0 |
| samples/hwp3-sample11-hwp5.hwp | 151 | 0 |
| samples/hwp3-sample16-hwp5.hwp | 64 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

```
diff -rq output/poc/issue_1058/sweep-before/ output/poc/issue_1058/sweep-after/ = 0
```

→ **전체 1143 SVG 완전 동일** (직접 export 회귀 부재 정량 입증). Task #1058 본질은 serializer 영역, 직접 export 무관.

## 6. WASM 빌드 + 동기화

| 항목 | 결과 |
|------|------|
| Docker WASM 빌드 | OK (`pkg/rhwp_bg.wasm` **4.91 MB**) |
| rhwp-studio 동기화 | OK (`public/rhwp_bg.wasm` + `rhwp.js`) |

## 7. 한컴 시각 판정용 산출물

```
output/poc/issue_1058/footnote-tbox-01-final.hwp  (14 KB)
output/poc/issue_1058/footnote-01-final.hwp       (60 KB)
```

판정 항목:
1. 한컴 한글 2020 으로 열기 + 신규 각주 추가
2. "1.1.1.1.1.1." 본문 다단계 목록 부여 **안 됨** 확인
3. Task #1050 통과 영역 (각주 영역 조판) 회귀 부재

## 8. 다음 단계

Stage 4 — 작업지시자 시각 판정 + merge + close + orders + archives.
