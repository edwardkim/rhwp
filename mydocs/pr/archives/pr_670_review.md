# PR #670 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #670 |
| 제목 | samples: 한글2022 PDF 변환본 추가 (pdf/) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — PR #629/#620/#578 직후 동일 컨트리뷰터, 4번째 사이클 PR |
| base / head | `devel` ← `planet6897:local/task-pdf-samples` |
| state / mergeable | OPEN / MERGEABLE / **BEHIND** |
| 변경 | 199 files, +346 / -0 (PDF 196 binary + 스크립트/로그 3) |
| commits | 1 (`8764b1b`) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **없음** (PR 본문에 closes 키워드 부재) |
| 작성일 / 갱신 | 2026-05-07 09:19 / 09:33 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- CodeQL ✅
- WASM Build SKIPPED

### 댓글 영역
- 5/7 09:22 (컨트리뷰터): **한글 2010 / 2020 추가 변환 제안** — 버전 간 비교 자료 확보 + 폴더 구조 제안 (`pdf-2010/` / `pdf-2020/` / `pdf/` 분리 또는 동일 폴더 접미어 분리)
- 5/7 09:32 (작업지시자): "와우! 고생하셨습니다."
- 5/7 09:33 (컨트리뷰터): **한글 2022 PDF 자동 변환 방법 상세** (pyhwpx + FilePathCheckerModule.dll + FileSaveAs_S 액션 + 크래시 복구 + watchdog hang 감지 + 본 PR 변환 결과 통계)

---

## 2. 본 PR 의 본질 영역

### 본질
`samples/` 폴더의 HWP/HWPX 파일 207개를 한글 2022 OCX 자동화로 PDF 변환 → 199 성공 (96.1%) → `pdf/` 폴더에 영구 추가.

### 추가 영역
- **PDF 196개** (한글 2022 변환본, 명명 규약 `{원본 stem}-2022.pdf`)
- `pdf/_convert.ps1` (+106 LOC) — PowerShell 변환 스크립트 (크래시 복구 포함)
- `pdf/_watchdog.ps1` (+19 LOC) — Hang 감지 watchdog
- `pdf/_convert.log` (+221 LOC) — 본 변환 실행 전체 로그

### 변환 결과 통계 (컨트리뷰터 댓글)
| 항목 | 값 |
|------|-----|
| 총 입력 (samples 재귀) | 207개 (.hwp + .hwpx) |
| 성공 | 199 (96.1%) |
| 실패 | 8 (3.9%) |
| 크래시 자동 복구 | 3건 |
| 평균 변환 시간 | ~3.5초/파일 |
| 총 소요 시간 | ~15분 |

### 실패 8개 영역
| 파일 | 추정 원인 |
|------|-----------|
| `20250130-hongbo_saved.hwp` | RPC 0x800706BE |
| `20250130-hongbo-no.hwp` | SaveAs False |
| `basic\calendar_monthly.hwp` | 8분 hang |
| `honbo-save.hwp` | 한글 크래시 |
| `hwp_table_test_saved.hwp` | 한글 크래시 |
| `hwp-3.0-HWPML.hwp` | HWPML 형식, SaveAs 실패 |
| `hwpers_test4_complex_table.hwp` | 복잡한 테이블, 크래시 |
| `hwpspec.hwp` | 한컴 스펙 문서, 크래시 |

---

## 3. 본 환경 정합 상태 점검

### 본 환경 기존 PDF 영역 (samples/ 직속 git tracked)
21 PDF 영구 보존 영역, 명명 규약 혼재:

| 패턴 | 예시 |
|------|------|
| `{stem}.pdf` (버전 표기 없음) | `복학원서.pdf` / `equation-lim.pdf` / `pua-test.pdf` |
| `{stem}-2010.pdf` | `21_언어_기출_편집가능본-2010.pdf` / `exam_eng-2010.pdf` |
| `{stem}-2020.pdf` | `aift-2020.pdf` / `exam_eng-2020.pdf` / `21_언어_기출_편집가능본-2020.pdf` |
| `{prefix}-{stem}.pdf` (provider 접두어) | `2010-exam_kor.pdf` / `2020-exam_kor.pdf` / `hancomdocs-exam_kor.pdf` |
| `{stem}.pdf` (HWP 직접 비교용) | `2022년 국립국어원 업무계획.pdf` (samples/ 직속, hwp 와 동일명) |

분포:
- `samples/` 직속: 17 PDF
- `samples/hwpx/`: 4 PDF (`issue_241.pdf` / `aift-2020.pdf` / `form-002.pdf` / `hwpx-h-02.pdf`)

### 본 환경 .gitignore 영역
```
# Output (렌더링 결과물)
/output/
```
PDF 영역 미명시 — 영구 보존 영역은 `samples/` 직속 git tracked 정합, 본 PR 의 `pdf/` 폴더는 신규 영역.

### 본 환경 메모리 룰 정합 영역
- `reference_authoritative_hancom` — **한컴 2010 + 한컴 2022 의 편집기 출력만 권위 정답지**. 한컴뷰어 / HWP5 v2024 변환본 / macOS 인쇄 / 외부 변환은 정답지 아님.
- `feedback_pdf_not_authoritative` — **PDF 출력은 환경별 차이로 정답지 미입증**. 본 PR 의 한글 2022 PDF 도 동일 영역 정합.
- 본 PR 의 PDF 199 영역은 **권위 정답지 영역 아님** (메모리 룰 명시 영역) — 다만 시각 비교 자료 / 픽셀 차이 측정 영역의 보조 자료 영역으로 활용 가능.

---

## 4. PR 의 주요 정정 영역

### 4.1 PDF 변환 자동화 본질 영역
- **pyhwpx + FilePathCheckerModule.dll** 보안 다이얼로그 차단
- **FileSaveAs_S 액션 + Attributes=0** 모아찍기 차단 (1 page/sheet 강제)
- **크래시 자동 복구** (RPC_S_SERVER_UNAVAILABLE / RPC_S_CALL_FAILED 패턴 감지 → Hwp.exe 강제 종료 → 새 COM 인스턴스 + RegisterModule 재호출)
- **Watchdog hang 감지** (10초마다 _convert.log mtime 감시 → 60초 무변동 시 Hwp.exe 강제 종료)

### 4.2 한글 2010 / 2020 추가 변환 제안 영역 (컨트리뷰터 댓글)
- COM ProgID 동일 (`HWPFrame.HwpObject`) — 한 시스템 마지막 등록 1개만 활성화 → 가상 머신 / 별도 사용자 계정 분리 필요
- FilePathCheckerModule.dll 등록은 한글 버전 무관
- FileSaveAs_S 의 Attributes 매개변수 호환, 단 일부 옵션 (PDF/A 등) 버전 차이 — 첫 1~2개 파일 시각 확인 권장

---

## 5. 본 환경 옵션 분류

### 본 PR 의 영역 본질
1. **이슈 미연결** — `closes` 키워드 부재 영역. 외부 컨트리뷰터의 자기 등록 영역 (`feedback_assign_issue_before_work` 영역에서 외부 컨트리뷰터 일차 방어선 부재 케이스)
2. **권위 자료 영역의 권위 등급** — `reference_authoritative_hancom` 메모리 룰 영역에서 PDF 는 권위 정답지 아님 영역 명시 → 본 PR 의 199 PDF 는 보조 자료 영역
3. **48MB 영구 추가** — `pdf/` 폴더 신규 + 196 PDF binary + 3 스크립트/로그
4. **폴더 구조 영역의 결정 영역** — `pdf/` (본 PR 제안) vs `samples/` 직속 (기존 21 PDF 패턴) vs 다른 영역

### 옵션 A — 전체 cherry-pick 머지 (`pdf/` 폴더 + 196 PDF + 3 스크립트)

**진행 영역**:
```bash
git checkout devel && git checkout -b local/pr670
git cherry-pick 8764b1b
git checkout devel && git merge --no-ff local/pr670 \
  -m "Merge: PR #670 cherry-pick (samples 한글 2022 PDF 변환본 199 + 변환 자동화 스크립트)"
```

**장점**:
- 한글 2022 변환본 영역 영구 보존 — 향후 한글 편집기 시각 판정 시 보조 자료 영역
- 변환 자동화 스크립트 영역 영구 보존 (`_convert.ps1` / `_watchdog.ps1`) — 향후 한글 2010 / 2020 추가 변환 시 재사용 영역
- `_convert.log` 영역 — 변환 실행 영역의 reproducibility 영역 보존
- 컨트리뷰터의 변환 자동화 노력 (~15분 + 자동 복구 3 건 + watchdog) 영역의 결과물 영구 인정

**잠재 위험**:
- **48MB 영구 추가** — 본 환경 repo 영역의 binary 비용 영역
- **권위 정답지 영역 아님** (`reference_authoritative_hancom` + `feedback_pdf_not_authoritative`) — 본 PDF 영역의 권위 등급 영역의 명시 필요 (혼동 영역 회피)
- **폴더 구조 영역의 본 환경 정합성 미정합** — 기존 21 PDF 가 `samples/` 직속에 영구 보존된 패턴과 다른 영역 (`pdf/` 신규 폴더 영역). 본 환경 통합성 영역 결정 필요
- **명명 규약 차이** — 기존 영역은 `{stem}-2010.pdf` / `-2020.pdf` 등 혼재, 본 PR 영역은 `{stem}-2022.pdf`. 향후 한글 2010/2020 추가 시 일관 규약 영역 결정 영역
- **이슈 미연결** — closes 키워드 부재로 이슈 추적 영역 부재

### 옵션 B — 부분 cherry-pick (PDF 199만, 스크립트 제외)

**진행 영역**:
```bash
git checkout devel && git checkout -b local/pr670_partial
git checkout local/pr670 -- pdf/*.pdf "pdf/basic/" "pdf/hwpx/" "pdf/hwpx/ref/"
git add pdf/
git commit --author="Jaeook Ryu <jaeook.ryu@gmail.com>" -m "samples: 한글 2022 PDF 변환본 199개 (PR #670 부분, 스크립트 제외)"
```

**장점**:
- PDF 영역만 영구 보존 — 변환 자동화 스크립트 영역은 별도 영역 분리 (rhwp repo 본질 영역 외)
- repo 비용 영역 일부 좁힘 (스크립트 ~ 350 LOC 영역 제외)

**잠재 위험**:
- 변환 자동화 스크립트 영역 손실 — 향후 한글 2010 / 2020 추가 변환 영역에서 재현성 영역 부재
- 컨트리뷰터의 작업 영역 일부 손실
- 이미 PDF binary 가 48MB 영역 — 스크립트 350 LOC 영역 제외로 비용 절감 영역 미세

### 옵션 C — close + 폴더 구조 / 권위 등급 / 명명 규약 영역 결정 후 재제출 권유

**진행 영역**:
컨트리뷰터에게 PR close + 본 환경 정합 영역 결정 후 재제출 권유:
1. 폴더 구조 영역: `pdf/` (신규) vs `samples/pdf/` vs `samples/` 직속 (기존 패턴 정합)
2. 권위 등급 영역: 메모리 룰 영역 명시 (`reference_authoritative_hancom` 영역에서 PDF 는 정답지 아님)
3. 명명 규약 영역: `{stem}-2022.pdf` vs 다른 패턴
4. 한글 2010 / 2020 추가 변환 영역과의 통합 영역 (한 PR 묶음 vs 분리)

**장점**:
- 본 환경 정합 영역의 결정 영역 정리 후 진행 — 향후 영역 정합 영역 부담 영역 회피
- 한글 2010 / 2020 추가 변환 영역 영역과의 통합 영역 결정 가능

**잠재 위험**:
- 컨트리뷰터의 ~15분 변환 작업 영역 + 작성한 commit 영역의 close 부담
- 작업지시자의 "와우! 고생하셨습니다." 댓글 영역의 모호 영역 (이미 환영 영역) — close 영역의 정합 영역 미정합

### 옵션 D — 작업지시자 결정 영역 (메모리 룰 영역 명시 후 진행)

**진행 영역**:
1. 본 PR 의 PDF 영역 권위 등급 영역 명시 — README 또는 `samples/pdf/README.md` 등에 "한글 2022 PDF 출력 영역 — 권위 정답지 아님 (메모리 룰 `reference_authoritative_hancom` 정합)" 명시
2. 폴더 구조 영역 결정 — 본 환경 통합 영역
3. 옵션 A 또는 옵션 B 진행

**장점**:
- 메모리 룰 영역의 명시 영역 + 본 환경 정합 영역 결정 영역 모두 정합
- 컨트리뷰터의 작업 결과 영역 보존 + 향후 한글 2010 / 2020 영역 통합 영역

**잠재 위험**:
- 작업지시자 결정 영역 부담 (폴더 구조 + 권위 등급 + 명명 규약 + README)
- 본 PR 처리 영역의 사이클 단축 영역 trade-off

---

## 6. 결정 영역 (잠정)

### 핵심 결정 영역
1. **권위 등급 영역** — 본 PR 의 199 PDF 영역의 권위 등급 영역 명시 가/부 (메모리 룰 `reference_authoritative_hancom` 정합 영역)
2. **폴더 구조 영역** — `pdf/` (PR 본문 제안) vs `samples/pdf/` vs `samples/` 직속 (기존 패턴) vs 다른 영역
3. **명명 규약 영역** — `{stem}-2022.pdf` 일관 영역 정합 가/부 (향후 한글 2010 / 2020 추가 변환 영역 정합 영역)
4. **변환 자동화 스크립트 영역의 영구 보존 영역** — `_convert.ps1` / `_watchdog.ps1` / `_convert.log` 의 rhwp repo 영역 정합 영역 가/부
5. **이슈 등록 영역** — 본 PR 의 closes 영역 부재 — 후속 이슈 (예: "한컴 PDF 권위 자료 영역 영구 보존 영역") 등록 영역 가/부
6. **48MB 영구 추가 영역의 비용 영역** — 본 환경 git repo 영역의 binary 비용 영역 평가

### 권장 영역
**옵션 D — 메모리 룰 영역 명시 + 폴더 구조 영역 결정 + 옵션 A 또는 B 진행** 권장. 사유:
1. 본 PR 의 권위 등급 영역 정확 표기 영역 — 메모리 룰 `reference_authoritative_hancom` + `feedback_pdf_not_authoritative` 영역의 명시 영역 (혼동 영역 회피)
2. 폴더 구조 영역의 본 환경 통합 영역 결정 — 기존 21 PDF 영역 (`samples/` 직속) 과의 통합 영역
3. 명명 규약 영역의 일관 영역 — 향후 한글 2010 / 2020 추가 변환 영역의 정합 영역
4. 컨트리뷰터의 작업 결과 영역 영구 보존 + 향후 한글 2010 / 2020 영역 추가 영역

### 검증 영역 (옵션 A 또는 B 진행 시)
PDF binary 영역만 추가되고 src 영역 변경 없음 → 결정적 검증 영역 거의 영향 없음:
1. `cargo test --lib --release` 1155 passed 정합 (회귀 0)
2. `cargo clippy` 0 warning 정합
3. `cargo build --release` 정합
4. WASM 빌드 정합 (영향 0)
5. `rhwp-studio npm run build` 정합 (영향 0)
6. PDF 영역의 권위 등급 영역 명시 (README 또는 `samples/pdf/README.md`)

---

## 7. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `reference_authoritative_hancom` — 한컴 2010 + 한컴 2022 의 **편집기** 출력만 권위 정답지. 본 PR 의 PDF 출력 영역은 정답지 아님 영역 (보조 자료 영역).
- `feedback_pdf_not_authoritative` — PDF 출력은 환경별 차이로 정답지 미입증 영역. 본 PR 의 PDF 영역도 동일 영역 정합 — 결정적 검증 영역의 1차 기준 수용 금지.
- `feedback_assign_issue_before_work` — 본 PR 의 closes 키워드 부재 + 이슈 미연결 영역 (외부 컨트리뷰터 자기 등록 영역, 일차 방어선 부재 사례).
- `feedback_pr_comment_tone` — 작업지시자의 "와우! 고생하셨습니다." 영역의 표현 영역 (차분 + 사실 중심 영역과 비교 영역).
- `feedback_visual_regression_grows` — PDF 영역은 시각 비교 자료 영역 (작업지시자 시각 판정 영역의 보조 자료 영역).

---

## 8. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_670_review.md` 작성 → 승인 요청
3. (필요 시) `pr_670_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정) + 판단 → `pr_670_report.md` 작성

### 작업지시자 결정 요청
1. **권위 등급 영역 명시 가/부** — 본 PR 의 PDF 199 영역의 권위 등급 영역 (메모리 룰 정합 영역)
2. **폴더 구조 영역** — `pdf/` (PR 제안) vs `samples/pdf/` vs `samples/` 직속 vs 다른 영역
3. **명명 규약 영역** — `{stem}-2022.pdf` 일관 영역 정합 가/부 + 한글 2010 / 2020 추가 변환 시 영역
4. **변환 자동화 스크립트 영역** — `_convert.ps1` / `_watchdog.ps1` / `_convert.log` 영역 영구 보존 가/부
5. **이슈 등록 영역** — 본 PR 의 후속 이슈 등록 가/부 (한컴 PDF 권위 자료 영역 / 한글 2010 / 2020 추가 변환 영역)
6. **옵션 결정** — 옵션 A (전체) / 옵션 B (PDF 만) / 옵션 C (close + 재제출) / 옵션 D (결정 + 진행)

결정 후 본 환경 결정적 검증 (회귀 0 영역 + 영향 없음) 진행 + `pr_670_report.md` 작성.
