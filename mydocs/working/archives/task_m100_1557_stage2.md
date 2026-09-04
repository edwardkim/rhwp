# Task #1557 Stage 2 완료보고서 — serializer 수정 (secCnt)

## 변경
`src/serializer/hwpx/header.rs:37`
```rust
- let sec_cnt = doc.doc_properties.section_count.max(1).to_string();
+ let sec_cnt = doc.sections.len().max(1).to_string();
```
`secCnt` 를 stale 가능한 `doc_properties.section_count` 대신 실제 직렬화 섹션 수
(`doc.sections.len()`, 섹션 파일 수와 항상 일치)로 산출. 주석으로 사유 명시.

## 검증 (fixed 바이너리)
| 파일 | header secCnt | 게이트 | 한글 원본→fixed_rt |
|------|----:|--------|----|
| 36382669 | 1 → **3** | PASS diff=0 (불변) | 8 → **8 완전 복원** ✅ |
| 36384160 | 1 → **3** | IR_DIFF d10 (불변) | 1 → **3** (3구역 로드, 잔여는 본문 내 별도 요인) |

- IR diff **불변**(36382669 PASS 유지) — 수정이 IR 의미를 바꾸지 않음(직렬화 메타만 교정).
- 순수 secCnt 붕괴(36382669) **완전 해소**. 36384160 은 secCnt 외 추가 요인(표 내 pic 드롭 등, V2-B 계열)으로 29 미달 — 본 타스크 범위 외.

## 빌드
`cargo build --release` 성공.

## 다음
Stage 3 — 코드 레벨 회귀 가드(secCnt == 섹션 수 단언) + baseline 회귀 없음 + hwpdocs 광역 재측정(붕괴율 감소) + 최종 보고.
