# Task #902 구현 계획서 v2 — WMF renderer 근본 개선

**이슈**: [edwardkim/rhwp#902](https://github.com/edwardkim/rhwp/issues/902)
**수행 계획서**: [task_m100_902_v2.md](task_m100_902_v2.md)
**v1 → v2 전환**: WMF renderer 근본 개선으로 scope 확장 (multi-stage, open-ended)

## Stage 3 — fix3 (DX byte-aware indexing) commit

### 3.1 변경 내용 요약

`src/wmf/converter/svg/mod.rs:915~935`:
- 기존: tspan `dx` (relative) + `excess_dx` 감산 + grapheme index 로 DX 접근
- v2: tspan `x` (absolute) + `unicode_width::s.width()` 기반 byte-aware DX 접근

### 3.2 ROOT CAUSE

WMF EXTTEXTOUT 의 DX 배열은 **MBCS byte index** — Korean wide char (2 byte) 는 DX 2 entry 차지 (실제 advance + 0). 기존 grapheme index 접근은 wide char 마다 매 둘째 dx=0 산출 → 글자 위치 잘못 누적.

### 3.3 검증

- [x] `cargo build --release` 성공
- [x] `cargo test --release --all-targets` — 1412 passed / 0 failed
- [x] SVG 출력: tspan x 값 monotonic 정합 (예: 174 → 291 → 408 → 525 → 641, +117 advance)
- [x] 시각 비교: text 위치 호전 — 한컴 참조와 구조 정합 (잔존 차이 = 폰트/viewport 갭, Stage 4~7 영역)

### 3.4 산출물

- 소스 수정: `src/wmf/converter/svg/mod.rs`
- Stage 보고서: `mydocs/working/task_m100_902_stage3.md`

## Stage 4 — META_SETVIEWPORTEXT/ORG 구현 + MM_ANISOTROPIC ratio 정합

### 4.1 진단 절차

- [ ] `META_SETVIEWPORTEXT` / `META_SETVIEWPORTORG` 의 현 `not implemented` 영역 식별
- [ ] WMF spec 의 MM_ANISOTROPIC 변환 공식 확인:
  ```
  device_x = (logical_x - WindowOrg.x) × (ViewportExt.x / WindowExt.x) + ViewportOrg.x
  device_y = (logical_y - WindowOrg.y) × (ViewportExt.y / WindowExt.y) + ViewportOrg.y
  ```
- [ ] sample16 (ViewportExt 미호출) vs ViewportExt 호출 sample 비교
- [ ] 한컴 사적 default ViewportExt 추정 (현재 viewBox 자동 확장 결과 vs 한컴 출력)

### 4.2 구현

- [ ] `set_viewport_ext` / `set_viewport_origin` 의 context 갱신 추가
- [ ] context 의 viewport 정보 반영하여 device 좌표 계산
- [ ] ViewportExt 미설정 시 default 처리 정의 (현재 Task #860 자동 확장과 호환성 유지)

### 4.3 검증

- [ ] 회귀: sample14 (Task #860 fixture), sample16, sample17/18 의 WMF SVG
- [ ] `cargo test --release --all-targets` 통과

### 4.4 산출물

- `mydocs/working/task_m100_902_stage4.md`

## Stage 5 — EXTTEXTOUT options flags 처리

### 5.1 대상 플래그

| 플래그 | 의미 | 처리 우선순위 |
|--------|------|--------------|
| `ETO_OPAQUE` (0x0002) | 배경색 채우기 | 중 |
| `ETO_CLIPPED` (0x0004) | clip rect 적용 | 중 |
| `ETO_GLYPH_INDEX` (0x0010) | char code = glyph index | 낮 |
| `ETO_RTLREADING` (0x0080) | 오른→왼 방향 | 낮 |
| `ETO_PDY` (0x2000) | per-char Y offset (DX 가 dx,dy 쌍) | 중 (회귀 위험) |

### 5.2 구현

- [ ] flag bit 검사 후 처리 분기
- [ ] `ETO_OPAQUE` + `ETO_CLIPPED` 의 background rect 처리
- [ ] `ETO_PDY` 의 dx/dy 쌍 처리 (현 코드는 dx 만 처리)

### 5.3 검증 + 산출물

- [ ] 회귀 점검
- [ ] `mydocs/working/task_m100_902_stage5.md`

## Stage 6 — 미구현 WMF records 완성

### 6.1 식별

`src/wmf/converter/svg/mod.rs` 의 `not implemented` 영역 grep:
- `META_SETMAPPERFLAGS`
- `META_SETPALENTRIES`
- `META_ESCAPE`
- 기타 무시되는 records

### 6.2 구현 우선순위

본 sample 들이 실제 사용하는 record 우선:
- WMF record histogram 분석 (sample 별 record type 분포)
- 빈번 사용 record 우선 구현

### 6.3 검증 + 산출물

- `mydocs/working/task_m100_902_stage6.md`

## Stage 7 — 폰트 metric 정합

### 7.1 옵션

| 옵션 | 처리 | 효과 |
|------|------|------|
| 7-A | CSS font-family 체인 우선순위 조정 (Apple SD Gothic Neo 우선) | 미세 — fallback 임 |
| **7-B** | 나눔고딕/D2Coding 등 오픈 한국어 폰트 substitute (font-family 매핑) | 중 — metric 비교적 정합 |
| 7-C | 굴림체 호환 폰트 임베딩 (`--embed-fonts` 기존 옵션 활용) | 큼 — 정합 우수, 파일 크기 증가 |
| 7-D | per-tspan `textLength` 강제 (B'' from analysis) | 미세 왜곡 가능, scope 최소 |

권장: **7-B 시도 후 효과 부족 시 7-C 또는 7-D**.

### 7.2 검증

- [ ] 한국 폰트 substitute 매핑 (굴림체 → 나눔고딕)
- [ ] 시각 비교: sample16 page 18

### 7.3 산출물

- `mydocs/working/task_m100_902_stage7.md`

## Stage 8 — 광범위 회귀 검증

### 8.1 회귀 점검

- [ ] 모든 WMF 사용 sample SVG 회귀
- [ ] golden SVG 회귀 (tests/golden_svg/)
- [ ] `cargo test --release --all-targets`
- [ ] sample14 (Task #860 fixture) 정합 유지

### 8.2 산출물

- `mydocs/working/task_m100_902_stage8.md`

## Stage 9 — 통합 + 최종 보고서 + PR

- [ ] 최종 보고서: `mydocs/report/task_m100_902_report.md`
- [ ] orders 갱신
- [ ] PR 생성 (작업지시자 명시 승인 후)
- [ ] issue #902 회신

### 산출물

- `mydocs/report/task_m100_902_report.md`

## 위험 평가

| Stage | 위험 | 완화 |
|-------|------|------|
| 4 | viewport 처리 변경 → 다른 WMF 회귀 | 매 stage 후 다중 sample 회귀 |
| 5 | ETO_PDY 처리 → 일부 sample 의 dx 매핑 변경 | 회귀 점검 |
| 6 | 미구현 records 활성화 → 예측 못 한 부작용 | record 별 conservative 구현 |
| 7 | 폰트 임베딩 → SVG 크기 폭증 (수 MB) | font-subset only (CharShape 사용 글자만) |
| 8 | golden SVG 변경 — 다수 fixture 갱신 필요 | sample 별 시각 점검 + 의도된 변경 confirm |

## 의사결정 요청

본 구현 계획서 v2 자체 승인. 승인 시:
1. Stage 3 (fix3 commit) → 단계별 진행
2. 매 stage 보고/승인
