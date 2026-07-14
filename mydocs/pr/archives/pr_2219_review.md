# PR #2219 검토 — HWPML(HML) 문서 열기 + 의미 보존 저장 (#1157)

- 작성자: cskwork (과거 5건 미머지 CLOSED — 본 건이 실질 첫 머지 후보) / 검토일: 2026-07-12
- base: devel / 70파일 +8,994/−368 / MERGEABLE, CI 12항목 green
- 판정: **B안 — 문서 수정 2건 요청(CHANGES_REQUESTED) 후 merge 수용**, 렌더
  차이 2건은 머지 후 후속 이슈 분리 (작업지시자 옵션 결정)

## 검증 (결합 로컬 게이트)

- Rust 전수 3,121/0 (HML 신규 61건: CLI 12/파서 26/직렬화 23) / fmt·clippy 0
- Studio npm 225/0 + production build / OVR 3샘플 0건
- 구조 격리: parser/hml·serializer/hml 분리, 렌더러·공통 모듈 비접촉
  (document_core에 HmlImportMetadata Option 1개), 파서 구조 규칙 정합
- 저장 계약: 손실 시 구조화 거부(차단 코드+XML 경로), TAIL/SCRIPTCODE 원본
  조각 보존, atomic write + 하드링크 거부, Studio 첫 저장 '다른 이름으로' 강제
- 샘플: ohah-hwpjs(MIT)/osk_filter(UNLICENSE) 라이선스 동봉 — 재배포 적법성 처리

## 실측 차이 2건 (후속 이슈 대상)

1. formatting_table.hml 표 폭 — IR 단계부터 41956HU(148mm 전체폭) 파싱,
   뷰어 참고(약 절반 폭)와 상이.
2. "abc[표]efg" 인라인 병합 — 뷰어는 abc/표/efg 세로 분리.

## 수정 요청 2건 (문서 수준, 반영 후 approve)

1. "정답지" 용어 → 보조 대조 자료 (한컴 뷰어 macOS 출력은 권위 등급 미달).
2. 지원 범위와 제한에 표 폭·표 배치 해석 차이 명시.

---

## 재검토 (v2, 2026-07-13) — 수정 반영 + 추가 커밋 2건

head `1051e108` (검토 시점 대비 70→116파일, +8,994→+12,386).

### 요청 2건 처리 확인

1. **정답지 용어**: 반영 확인 — `samples/hml/README.md` "보조 대조 자료" 재분류.
2. **표 폭 148mm**: **원 소견 오판으로 정정**. 실측 — HML 원문 선언 41956 HU
   = 419.6pt = 148.0mm 이 IR(dump `148.0×4.5mm`)·SVG(559.4px)·뷰어 대조
   자료(~420pt)와 전부 일치. 파싱 결함 아님 → 제한 명시 요청은 소멸.

### 추가 커밋 평가

- **9083e9d1 (TAC 중간 앵커)**: 원 검토의 "렌더러·공통 모듈 비접촉" 전제를
  벗어나 공통 렌더러 6파일 수정. 다만 위험 관리가 검증됨 —
  ①3중 가드(TAC 표 + 텍스트 중간 앵커 + 양쪽 alphanumeric, #842 판정 재사용)
  ②측정(height_measurer)·렌더(paragraph_layout) 두 경로 대칭 교체
  ③반례(issue_2020 복학원서 U+F081C 필러)를 자체 발견·수정
  ④공개 표면은 `is_tac_table_inline_in_para` 추가만(기존 시그니처 불변).
  abc/표/efg 시각 실측: y=207→231→249 세로 순서 정합(보고 수치 일치).
- **1051e108 (수식 import/export)**: 리뷰 후 scope 확장이나, HML 격리 밖
  접촉은 `intrinsic_size_hwp` 헬퍼 추출(기존 중복 제거)과 수식 삽입 시
  intrinsic 크기 기록(0,0 → 계산값)으로 건전. studio embed 프로토콜 연동은
  #2187 merge 위 rebase 완료 상태.

### 결합 게이트 재실증 (PR head, 로컬)

| 게이트 | 결과 |
|--------|------|
| `cargo test --profile release-test --tests --no-fail-fast` | **3,154 / 0** |
| fmt --check / clippy(all-targets) | 통과 / 0 |
| OVR 5샘플 (±2px, 분리 폴더 `output/poc/pr2219v2/`) | **회귀 0건** — 공통 렌더러 TAC 변경의 비-HML 시각 파급 없음 |
| studio npm ci + tsc + test + build | 클린 / **270 / 0** / 성공 |

### 판단

approve → merge 수용 권고. 렌더러 공통 변경은 전제 이탈이지만 가드·대칭·
반례·OVR 4중 근거로 위험이 통제됐고, 표 폭 반박은 우리 오판의 정정이다.
