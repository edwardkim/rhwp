# 구현계획서 — Task M100 #1199

**이슈**: [#1199](https://github.com/edwardkim/rhwp/issues/1199) HWPX 미주/각주 마커 접두문자(prefixChar) 미파싱
**브랜치**: `local/task1199`
**수행계획서**: `task_m100_1199.md`
**작성일**: 2026-06-01

---

## 단계 분할 (3단계)

### Stage 1 — 파서 수정 (prefixChar → before_decoration_letter)

`src/parser/hwpx/section.rs`:
- `parse_ctrl_endnote()` (3920~): `suffixChar` 분기와 대칭으로 `b"prefixChar"` 분기 추가 → u16 파싱하여 `note.before_decoration_letter` 설정.
- `parse_ctrl_footnote()` (3884~): 동일하게 `b"prefixChar"` 분기 추가 → `note.before_decoration_letter`.

기본값: `before_decoration_letter`는 `default()`의 0 유지(접두 없음). prefixChar 속성이 있을 때만 설정.

산출물: 소스 수정 + 빌드 성공.

### Stage 2 — 회귀 테스트

`src/parser/hwpx/section.rs` 테스트 모듈에 단위 테스트 추가:
- `<hp:endNote prefixChar="47928" suffixChar="65289">` 파싱 → `before_decoration_letter == 47928`, `after_decoration_letter == 65289` 검증.
- `<hp:footNote ...>` 동일 검증.
- prefixChar 없는 경우 `before_decoration_letter == 0` 유지 검증(회귀 방지).

산출물: `cargo test` 통과.

### Stage 3 — 시각 검증

1. `rhwp export-svg samples/3-09월_교육_통합_2022.hwpx -o output/svg/t1199/ -p 8`,`-p 9`,`-p 10`
2. 미주 마커가 `"문N）"`로 출력되는지 SVG 텍스트 grep 확인.
3. 한글 2022 PDF(`pdf/3-09월_교육_통합_2022.pdf`) 9~11쪽과 시각 대조 → 작업지시자 시각 판정.
4. `cargo fmt`(변경 파일 한정) → 최종 보고서 작성.

산출물: 검증 결과 + 최종 보고서.

---

## 영향 범위

- 수정 파일: `src/parser/hwpx/section.rs` 1개 (+테스트).
- 모델/렌더러/HWP3/공통 모듈 무변경.
- 회귀 리스크: 낮음 (입력 보강, 기존 경로 불변).
