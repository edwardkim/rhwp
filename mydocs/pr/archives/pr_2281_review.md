# PR #2281 검토 — svg2pdf 벤더 패치로 PDF 채번 결정화 (planet6897, #2269 (a)안)

- 검토일: 2026-07-15 / base: devel / 3파일 +48/−2 (Cargo.toml patch 지정 +
  Cargo.lock rev 고정 + tech 문서) / rhwp .rs 무변경
- 경위: 업스트림 기여 지시(1안) → typst/svg2pdf **2026-07-10 아카이브**
  확인(기여 불가, 공식 대체 krilla/krilla-svg) → 작업지시자 (a)안(벤더
  패치) 직접 승인(#2269 코멘트) → 이행.

## 공급망 감사 (fork 전량 검수)

- `planet6897/svg2pdf@2caeb0a` = **v0.13.0 태그 + 단일 커밋** "Make PDF
  output byte-for-byte reproducible".
- diff 전량: `util/context.rs`(+13/−4) `util/resources.rs`(+5/−1) —
  **외과적 정렬 2건뿐** (폰트를 Font.reference 로 정렬 후 방출, 리소스
  딕셔너리 항목 reference 정렬). 다른 파일·의존성·빌드 스크립트 무변경.
- 기준 무결성: fork 의 v0.13.0 태그 소스 == crates.io 0.13.0 배포본
  (context.rs·resources.rs 직접 대조 동일).
- Cargo.lock 이 rev `2caeb0a` 해시 고정 — 브랜치 후속 이동에도 불변.
- 진단 정련 가치: Font.reference/foN 이름은 원래 결정적(문서 순회 순)이고
  실체는 write 시점 부가 객체 채번 + 딕셔너리 순서 2지점이라는 보완 규명
  — reference 정렬이 자연 정준 순서라는 근거 명확.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| **결정성**: 같은 문서 2회 export-pdf | **바이트 동일** (무패치 시 466KB diff 였던 문서) |
| 무패치(어제 산출)↔패치 정규형 비교 | **동일** — 시각/구조 불변 실증 |
| 전수 `--tests --no-fail-fast` / fmt / clippy | **3,161/0** / 통과 / 0 |
| cargo tree | svg2pdf v0.13.0 이 fork rev 로 정확히 대체 확인 |

## 유지보수 계약

tech 문서에 채택 기록 + krilla-svg 마이그레이션 시 패치 제거 노트.
#2269 close 여부는 작업지시자 결정 대기 — 장기 krilla-svg 축을 본 이슈에
남길지 / 별도 이슈 분리할지.

## 판단

**approve → merge 수용 권고.** 공급망 우려는 rev 해시 고정 + 전량 감사로
해소. 권고: fork 브랜치가 개인 계정 소유이므로 장기적으로는 조직/rhwp
산하 미러 또는 krilla-svg 마이그레이션(#2264 계열)이 정착지.
