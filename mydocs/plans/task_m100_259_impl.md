# Task #259 구현 계획서 — HY 계열 한글 폰트명 alias 매핑

- 이슈: [#259](https://github.com/edwardkim/rhwp/issues/259)
- 수행계획서: [`task_m100_259.md`](task_m100_259.md)
- 브랜치: `local/task259`
- 작성일: 2026-04-23

## 개요

수행계획서 5단계를 각각 **작업 항목·변경 파일·검증 방법**으로 세분한다.

---

## Stage 1 — HY 매핑 테이블 확정

### 목표
`style_resolver.rs` 가 사용하는 HY 한글 정규명 7종 → `FONT_METRICS` 엔트리의 영문명 1:1 확정.

### 작업
1. `ttfs/` 또는 시스템에 존재하는 한컴 HY TTF 수집 확인:
   ```
   find ttfs/ -iname 'HY*' -type f
   fc-list | grep -i 'HY\|한양\|Myeong\|Gothic-Medium'
   ```
2. 각 TTF 의 PostScript name·Full name 추출 (Python fonttools 또는 `otfinfo -p`)
3. `FONT_METRICS` 내 HY 엔트리 35종과 대조하여 정규명 7종 매핑 확정
4. 실측 검증: DB 엔트리가 있는 ASCII/한글 대표 글자 폭 5~10 개를 TTF 에서 직접 추출한 폭과 ±1 units 비교
5. 실측 불가·PS name 미일치 시 **"없음"** 으로 기록하고 폴백 전략 결정 (가까운 weight 엔트리로 수동 매핑)

### 산출물
- `mydocs/tech/task_259_hy_mapping.md` — 7종 최종 매핑표 + 근거

### 검증
- 7종 중 N종 매핑 성공, (7-N)종 폴백 전략 명시

### 리스크
- TTF 가 ttfs/ 에 없는 경우: 시스템 한컴 오피스 설치 경로(`~/.fonts` 또는 한글 설치 폴더) 점검. 그래도 없으면 PS name 추정만으로 진행하고 회귀 테스트로 검증.

---

## Stage 2 — `resolve_metric_alias` 매핑 추가 + 단위 테스트

### 작업
1. `src/renderer/font_metrics_data.rs::resolve_metric_alias` 에 Stage 1 확정 매핑 추가:
   ```rust
   "HY중고딕" => "HYGothic-Medium",
   "HY견고딕" => "HYGothic-Extra",
   "HY헤드라인M" => "HYHeadLine-Medium",
   "HY견명조" => "HYMyeongJo-Extra",
   "HY신명조" => "HYSinMyeongJo-Medium",
   "HY그래픽" => "HYGraphic-Medium",
   "HY궁서" => "HYGungSo-Bold",
   ```
   *(실제 매핑값은 Stage 1 결과로 교체)*
2. 모듈 하단에 `#[cfg(test)] mod tests` 추가:
   ```rust
   #[test]
   fn hy_normalized_names_resolve_to_db_entries() {
       for name in ["HY중고딕","HY견고딕","HY헤드라인M","HY견명조",
                    "HY신명조","HY그래픽","HY궁서"] {
           assert!(find_metric(name, false, false).is_some(),
                   "HY alias failed: {}", name);
       }
   }
   ```

### 변경 파일
- `src/renderer/font_metrics_data.rs` (+~10 lines source, +~15 lines test)

### 검증
- `cargo test --lib renderer::font_metrics_data::tests` 그린
- `cargo clippy --lib -- -D warnings` 그린

### 커밋
- `Task #259: HY 한글 정규명 → 메트릭 DB 영문명 alias 7종 추가`

---

## Stage 3 — `text-align.hwp` 회귀 검증

### 작업
1. 네이티브 빌드: `cargo build --release`
2. SVG 재생성:
   ```
   ./target/release/rhwp export-svg samples/text-align.hwp -o output/svg/text-align-task259/
   ./target/release/rhwp export-svg samples/text-align.hwp --debug-overlay -o output/debug/text-align-task259/
   ```
3. 수정 전/후 s0:pi=4 문단 글자 폭 비교 (브라우저 혹은 `dump` 명령)
4. 전체 스냅샷: `cargo test --test svg_snapshot` — 변경 시 golden 갱신 필요 여부 판단
5. 웹 에디터 시각 확인:
   - `rhwp-studio` 실행 → `text-align.hwp` 열기 → s0:pi=4 겹침 해소 확인

### 성공 기준
- `cargo test` 전부 그린 (또는 HY 관련 golden 의도적 갱신)
- s0:pi=4 문단 웹 에디터에서 겹침 없음

### 산출물
- `mydocs/working/task_m100_259_stage3.md` — 검증 결과 + 스크린샷 경로

---

## Stage 4 — 타 HY 계열 스모크 검증

### 작업
1. 기존 `samples/` 중 HY 계열 7종을 사용하는 샘플 그레핑:
   ```
   for f in samples/*.hwp samples/*.hwpx; do
     ./target/release/rhwp dump "$f" 2>/dev/null | grep -oE 'HY[가-힣M]+' | sort -u \
       | sed "s|^|$f: |"
   done
   ```
2. 각 HY 정규명이 등장하는 샘플 1종 이상 확인. 없는 경우 스킵 (신규 샘플 생성은 범위 외 — 별도 이슈로 이관)
3. 각 샘플 SVG 렌더 + 해당 문단 시각 확인

### 성공 기준
- 등장하는 HY 폰트 모두 겹침/폭 이상 없음
- 없는 HY 폰트는 "해당 샘플 부재 — 후속 이슈" 로 기록

### 산출물
- `mydocs/working/task_m100_259_stage4.md`

---

## Stage 5 — 문서화 + 메모리 등록

### 작업
1. `mydocs/working/task_m100_259_stage1.md` ~ `_stage4.md` (각 스테이지 완료 후 작성)
2. 최종 보고서 `mydocs/report/task_m100_259_report.md` (수행 요약 + 매핑표 + 검증 결과 + 후속 이슈)
3. `mydocs/orders/20260423.md` 의 #259 항목 "완료" 로 갱신
4. 메모리 등록:
   - **feedback**: "HY/한컴 계열 폰트 메트릭 DB 에 엔트리를 추가할 때 `font_metrics_data.rs::resolve_metric_alias` 도 반드시 갱신해야 함. 정규명(`HY중고딕`)이 DB 영문명(`HYGothic-Medium`)과 다르기 때문. 누락 시 기본 폭 적용으로 글자 겹침 발생 (#259 재발 방지)."
   - **project**: Task #259 마무리 (2026-04-23 완료) 간단 기록 — 폰트 메트릭 경로 2단 정규화 구조

### 브랜치 마무리
1. 모든 변경 커밋 (최종 보고서, orders 갱신 포함)
2. `git status` 로 미커밋 확인
3. `local/task259` → `local/devel` 또는 `devel` merge (작업지시자 지시에 따름)

---

## 전체 커밋 계획

| 커밋 | 단계 | 내용 |
|------|------|------|
| 1 | Stage 1 | `Task #259: HY 폰트 매핑 테이블 조사` (`mydocs/tech/task_259_hy_mapping.md`, `_stage1.md`) |
| 2 | Stage 2 | `Task #259: HY 한글 정규명 → 메트릭 DB alias 7종 추가` (source + test + `_stage2.md`) |
| 3 | Stage 3 | `Task #259: text-align.hwp 회귀 검증` (필요 시 golden 갱신 + `_stage3.md`) |
| 4 | Stage 4 | `Task #259: HY 계열 스모크 검증` (`_stage4.md`) |
| 5 | Stage 5 | `Task #259: 최종 보고서 + orders 갱신 + 수행계획서 archive 이동` |

## 승인 요청

본 구현계획 승인 시 **Stage 1** (매핑 조사) 부터 착수합니다.
