# Task M100-852 구현 계획서

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852` (base: `local/devel = 7ec2e25f`)
- 수행 계획서: `mydocs/plans/task_m100_852.md` (승인)
- Stage 1 보고서: `mydocs/working/task_m100_852_stage1.md` (승인)

## 1. Stage 1 확정 root cause

`src/parser/hwpx/mod.rs:185` 의 `extra_streams: Vec::new()` 빈 초기화 — HWPX ZIP 내 13 파일 (Scripts/settings/Preview/content.hpf 등) 미처리. 결과 `cfb_writer.rs:155` 가 작성할 contract 스트림이 없어 한컴 손상 판정.

**Form 컨트롤 무관 — 모든 HWPX→HWP 변환 공통 결함**. troubleshootings/hwpx2hwp-rule.md 5.A (Container/Stream Contract) 위반.

## 2. 구현 전략 — 옵션 C 하이브리드

### Stage 2.1 — HWPX 컨테이너 → extra_streams 보존 (정공법)

HWPX ZIP 의 다음 4 파일을 HWP OLE 스트림으로 매핑:

| HWPX 파일 | HWP OLE 스트림 | 변환 |
|----------|----------------|------|
| `Preview/PrvText.txt` (UTF-8) | `/PrvText` | UTF-8 → UTF-16 LE 변환 (HWP5 spec) |
| `Preview/PrvImage.png` | `/PrvImage` | 그대로 (PNG passthrough) |
| `Scripts/sourceScripts` | `/Scripts/DefaultJScript` | zlib deflate (HWP5 표준 압축) |
| `Scripts/headerScripts` | `/Scripts/JScriptVersion` | 13 bytes 헤더 — fallback (Stage 2.2 정적) |

**구현 위치**:
- `src/parser/hwpx/reader.rs` — ZIP 추가 파일 읽기 헬퍼
- `src/parser/hwpx/mod.rs:185` — `extra_streams` 채우기

### Stage 2.2 — `saved/blank2010.hwp` fallback (실용)

HWPX 컨테이너에 동등 데이터가 없는 스트림은 정적 템플릿에서 추출:

| HWP OLE 스트림 | 출처 |
|----------------|------|
| `/HwpSummaryInformation` | `blank2010.hwp` 추출 (461 bytes) — title/creator 빈값 |
| `/DocOptions/_LinkDoc` | `blank2010.hwp` 추출 (524 bytes) — UTF-16 LE 임시 경로 |
| `/Scripts/JScriptVersion` | `blank2010.hwp` 추출 (13 bytes) — 버전 헤더 |

**구현 위치**:
- `src/parser/hwpx/contract_streams.rs` (신규) — `include_bytes!` 정적 자산 + 추출 헬퍼
- 가능 시 HWPX content.hpf opf:metadata 의 title/creator/date 를 HwpSummary 에 패치

### Stage 2.3 — BodyText/Section0 1202 bytes 잔존 누락 분석

이슈 본문 Stage 40 작업의 BodyText 영역 보강 후 잔존. 본 task 의 우선순위는 contract 스트림 (Stage 2.1+2.2). BodyText 잔존이 한컴 손상에 추가로 영향을 미치는지 작업지시자 시각 판정 후 Stage 2.4 로 분리 또는 본 task 범위 외.

## 3. Stage 분해

### Stage 2.1 — HWPX 컨테이너 → extra_streams (예상 2 시간)

1. `src/parser/hwpx/reader.rs` 에 ZIP 추가 파일 (Preview, Scripts) 읽기 헬퍼 추가
2. `src/parser/hwpx/mod.rs:185` 의 `extra_streams: Vec::new()` 를 채우기로 변경
3. Preview/Scripts 4 파일 → HWP OLE 스트림 매핑 + 형식 변환 (UTF-8→UTF-16 LE, PNG passthrough, zlib)
4. 단위 검증 (cargo test + clippy + fmt)
5. commit

### Stage 2.2 — `saved/blank2010.hwp` fallback (예상 1 시간)

1. `saved/blank2010.hwp` 에서 contract 3 스트림 (HwpSummary / DocOptions/_LinkDoc / Scripts/JScriptVersion) 정적 추출
2. `src/parser/hwpx/contract_streams.rs` 신규 모듈 — `include_bytes!` 정적 자산
3. HWPX content.hpf opf:metadata → HwpSummary 패치 (가능 시)
4. Stage 2.1 의 `extra_streams` 채우기에 fallback 통합
5. 단위 검증 + commit

### Stage 2.3 — 검증 (예상 1 시간)

1. **CFB 스트림 보존 검증**:
   - `rhwp convert samples/hwpx/form-01.hwpx output/.../form-01.hwp` 결과의 9 스트림 (정답지) 존재 확인
   - 각 스트림 크기 정답지 ±10% (Preview/Scripts) 또는 동일 (정적 fallback)
2. **HWP roundtrip 회귀 부재**:
   - `rhwp convert samples/hwpx/hancom-hwp/hy-001.hwp` 결과 스트림 보존 (기존 정상 동작 유지)
3. **광범위 sweep**:
   - HWPX (form-01/02 외 hy-001.hwpx, sample16-hwp5.hwpx 등) 변환 결과 CFB 스트림 9개 모두 보유
4. **한컴 호환 검증** (작업지시자 시각 판정):
   - form-01.hwp / form-02.hwp 변환 결과를 한컴 에디터에서 열어 손상 미판정 확인 (`feedback_self_verification_not_hancom` 게이트)
5. **CI 패턴 검증** (`feedback_push_full_test_required`):
   - `cargo test --release --lib` + `--tests` + `clippy -D warnings` (전체) + `fmt --all -- --check`

### Stage 3 — 회귀 가드 + 최종 보고서 (예상 30 분)

1. **회귀 가드 `tests/issue_852_hwpx_to_hwp_contract_streams.rs`**:
   - `form-01.hwpx` 변환 결과의 9 CFB 스트림 모두 존재 단언
   - 각 스트림 최소 크기 단언 (HwpSummary>=256, DocOptions>=256, Scripts>=10 등)
   - 일반 HWPX (hy-001.hwpx) 변환도 동일 contract 단언
2. 최종 보고서 `mydocs/report/task_m100_852_report.md`
3. orders/20260520.md 갱신
4. no-ff merge + devel push + 이슈 #852 close
5. 임시 브랜치/산출물 정리

## 4. 회귀 위험 평가

| 영역 | 위험도 | 근거 |
|------|--------|------|
| HWP→HWP roundtrip (기존 정상) | **낮음** | 본 PR 은 HWPX parser 만 변경 |
| HWPX→HWP 일반 (Form 무관) | **의도된 변경** | 모든 HWPX 변환에 contract 스트림 추가 |
| Document IR `extra_streams` 필드 | **영향 없음** | 기존 필드 활용만, 구조 변경 없음 |
| `saved/blank2010.hwp` 정적 자산 | **영향 없음** | 신규 모듈, 기존 BLANK_TEMPLATE 사용 패턴과 동일 |
| Stage 40 BodyText 보강 | **본 task 범위 외** | 잔존 1202 bytes 는 Stage 2.4 또는 후속 task |

## 5. 메모리 룰 정합

- `feedback_self_verification_not_hancom` — **핵심**: rhwp 재로드 OK ≠ 한컴 호환. Stage 2.3 작업지시자 한컴 에디터 판정 게이트 필수
- `feedback_visual_judgment_authority` — 한컴 에디터 손상 판정 최종 게이트
- `feedback_diagnosis_layer_attribution` — Stage 1 정확 진단 (HWPX parser 단일 누락)
- `feedback_hancom_compat_specific_over_general` — 옵션 C 하이브리드 (정공법 + fallback 케이스별)
- `feedback_image_renderer_paths_separate` — N/A (parser/serializer 영역)
- `feedback_fix_scope_check_two_paths` — HWP roundtrip (정상) vs HWPX 변환 (누락) 두 경로 차이 식별 + HWPX 만 정정 (HWP 무변경)
- `feedback_push_full_test_required` — Stage 2.3 cargo test --tests + fmt --all + clippy 전체 (CI 패턴)
- `reference_authoritative_hancom` — `samples/form-01.hwp` / `samples/form-02.hwp` 한컴 정답지 baseline
- `feedback_search_troubleshootings_first` — Stage 1 사전 검색 완료 (`hwpx2hwp-rule.md` 5.A 위반 확정)
- `project_output_folder_structure` — 산출물 `output/poc/task852/`

## 6. 잔존 / 후속 권고

- **BodyText/Section0 1202 bytes 잔존 누락** — Stage 2.3 검증에서 한컴 호환 영향 평가 후 별도 task 또는 본 task 범위 확장 결정
- **이슈 본문 명시 `form-002.hwpx`** (10 페이지 대형) 추가 검증 — form-01/02 정합 후 자동 정합 기대, Stage 2.3 sweep 에 포함
- **Scripts/DefaultJScript 압축 정확도** — Stage 2.1 zlib 변환이 한컴 압축과 정확히 일치하는지 byte-level 비교 (`feedback_self_verification_not_hancom` 정합)
- HWPX content.hpf opf:metadata 의 풍부한 정보 (title/creator/CreatedDate 등) 를 HwpSummaryInformation 으로 정밀 매핑 — 후속 정합 task

## 7. 참고 자산

- `saved/blank2010.hwp` — 한컴 호환 최소 HWP 템플릿 (`document.rs:515` BLANK_TEMPLATE 사용 중)
- `src/model/document.rs::Document::extra_streams` — 라운드트립 보존 필드 (HWP 정상 동작 입증)
- `src/serializer/cfb_writer.rs:155` — extra_streams 작성 로직
- `samples/hwpx/form-01.hwpx` / `form-02.hwpx` / `form-01.hwp` / `form-02.hwp` — 검증 fixture (commit `7ec2e25f`)
