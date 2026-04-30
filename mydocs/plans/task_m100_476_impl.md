# Task M100 #476 구현 계획서

연결: [수행계획서](task_m100_476.md), [Issue #476](https://github.com/edwardkim/rhwp/issues/476)

## 단계 구성 (4단계)

| 단계 | 내용 | 산출물 |
|------|------|--------|
| Stage 1 | 코드 추적 + D 임시 차단 구현 | `task_m100_476_stage1.md` |
| Stage 2 | D 단계 검증 (시각 + 회귀) | `task_m100_476_stage2.md` |
| Stage 3 | A 본질 수정 (paginator 라우팅) | `task_m100_476_stage3.md` |
| Stage 4 | A 단계 검증 + 임시 차단 정리 + 최종 보고 | `task_m100_476_stage4.md`, `task_m100_476_report.md` |

---

## Stage 1: 코드 추적 + D 임시 차단

### 1-1. 코드 추적 (수정 없음)

조사 대상:

1. **paginator의 인라인 Shape 페이지 라우팅 코드**
   - `src/renderer/pagination/` 디렉터리 전수 조사
   - PartialParagraph + treat_as_char Shape 처리 분기
   - PageItem::Shape를 페이지 N에 등록하는 의사결정 위치 파악
2. **paragraph_layout의 partial 처리 진입점**
   - `layout_partial_paragraph` (페이지 11 line 0..3 처리 시점)
   - 시작 line이 0인지 확인 가능한 경로 (start_line 인자)
3. **PageRenderTree 페이지 단위 생성 위치**
   - `render_tree.rs:730` 호출 caller 식별

### 1-2. D 임시 차단 구현

**전략**: `shape_layout.rs:218` 분기에서 다음 조건 모두 만족 시 박스 렌더 스킵.

조건:
- `common.treat_as_char == true`
- `tree.get_inline_shape_position(...) == None`
- 해당 paragraph가 PartialParagraph로 이전 페이지에서 시작됨 (즉, 박스의 line이 현재 페이지에 없음)

이전 페이지 시작 여부 판단:
- paragraph의 `line_segs[0].vertical_pos`와 현재 페이지 라우팅된 시작 line 비교
- 또는 paragraph의 첫 line이 현재 페이지 PageItem 범위에 포함되는지 검사

복잡도가 크면 더 단순한 시그널 사용:
- `treat_as_char Shape이고 inline_pos=None이며 paragraph가 매우 길어 분할되었음을 시사하는 경우(line_segs.len() ≥ 5)` 같은 휴리스틱 — 단, false positive 위험.

**최종 조건 결정은 1-1 조사 결과에 따라 확정**.

### 1-3. 회귀 안전장치

스킵된 박스에 대한 telemetry 로그(env var `RHWP_DEBUG_LAYOUT=1` 시 stderr 출력) 추가하여 다른 케이스에서 의도치 않은 스킵이 발생하면 빠르게 감지.

### 산출물

`mydocs/working/task_m100_476_stage1.md`:
- 코드 추적 결과(파일/라인/호출 경로)
- 적용한 D 차단 조건과 코드 diff 요약
- 빌드 통과 확인

---

## Stage 2: D 단계 검증

### 2-1. 핵심 시각 검증

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 11 -o output/svg/21_p12_d/
./target/release/rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 11
```

확인 사항:
- 페이지 12 단 0의 y≈742 위치에 박스/"배너지와 뒤플로" 텍스트가 **없어야 함**
- 단 0 사용량(used)이 한컴 기준(1219px)에 가까워야 함 (현재 1012px → 개선)
- 23번 문제 본문 라인에 박스가 들어가야 함 (이건 별도 버그일 가능성 — Stage 3에서 다룸)
- Chrome headless로 SVG → PNG 캡처 후 PDF와 시각 비교

### 2-2. 페이지 11 정상성 확인

페이지 11에서 paragraph 238의 박스 위치가 한컴/PDF와 일치하는지 확인:

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 10 -o output/svg/21_p11_d/
```

페이지 11에서 박스가 정상 위치에 그려지고 있어야 함 (set_inline_shape_position이 등록된 페이지).

### 2-3. 광범위 회귀 검증

```bash
cargo test 2>&1 | tail -50          # 전체 단위 테스트
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

골든 SVG 회귀:
- `tests/golden_svg/` 하위 모든 샘플 비교 (특히 `issue-*` 디렉터리 — 페이지 분할 + 도형 케이스 다수 포함 추정)
- 차이 발생 시 차이 내역을 stage 보고서에 기록

### 2-4. 추가 샘플 점검

`samples/` 하위에서 인라인 도형 포함 + 페이지 분할 가능성 있는 파일 식별 후 시각 확인:
- 메모리 항목 [본질 정정 회귀 위험] 가이드 — 다단/단일 단/표분할 상호작용 광범위 확인 필요

### 산출물

`mydocs/working/task_m100_476_stage2.md`:
- 시각 비교 결과 (캡처 또는 영역별 좌표)
- 회귀 테스트 결과 (통과/실패 건수, 실패 시 원인)
- 한컴 PDF와의 비교 (피드백 메모리: PDF는 보조 ref)

---

## Stage 3: A 본질 수정 (paginator 라우팅)

### 3-1. 변경 범위

paginator에서 PageItem::Shape를 등록할 때, 인라인(treat_as_char) Shape는 **박스가 위치한 line이 그려지는 페이지에만** 등록하도록 수정.

식별 알고리즘:
- paragraph의 char index에서 박스 위치 → `control_text_positions(para)[ci]` 사용
- 그 char index가 속한 line 번호 결정 → `line_segs`의 `text_start` 비교
- 그 line이 현재 paginator가 처리 중인 페이지에 라우팅되었는지 확인 → 라우팅된 경우에만 PageItem::Shape 등록

### 3-2. inline_pos 등록 정합성

paragraph_layout이 line 0을 처리할 때 set_inline_shape_position을 호출하므로, line 0이 페이지 11에 라우팅되면 페이지 11 tree에 등록됨. PageItem::Shape도 페이지 11에 등록하면 같은 tree에서 일관되게 그려짐.

### 3-3. 회귀 방지

A 수정으로도 Stage 1의 D 차단이 우연히 활성화되지 않도록 D 차단 조건의 부정적 영향을 회피:
- D는 fallback 위치 출력을 막는 안전장치 — A 수정 후에도 잠시 유지 후 Stage 4에서 정리

### 산출물

`mydocs/working/task_m100_476_stage3.md`:
- 변경된 paginator 코드 위치 + diff 요약
- 페이지 12 시각 검증 (박스/텍스트 정상 위치)
- 단 0 사용량 회복 확인

---

## Stage 4: A 단계 검증 + 정리 + 최종 보고

### 4-1. A 단계 광범위 회귀 검증

- `cargo test` 전체 통과
- `cargo clippy --all-targets -- -D warnings`
- 골든 SVG 회귀 — Stage 2 대비 추가 차이 없는지 확인
- 인라인 도형 포함 + 페이지 분할 샘플 시각 점검

### 4-2. D 차단 코드 정리

A 수정으로 근본 원인이 제거되었으므로 D 차단은 다음 중 하나로 정리:

(a) **제거**: 깔끔한 구조 유지. 다만 향후 paginator 회귀 시 fallback 노출 위험.
(b) **유지 + 경고 로그**: D 차단이 발동되면 경고 stderr — 회귀 검출 안전장치.

권장: **(b) 유지 + 경고 로그**. 안전장치 비용이 작고, 회귀 시 빠른 감지 가능.

### 4-3. 최종 보고서

`mydocs/report/task_m100_476_report.md`:
- 증상/원인/수정/검증 요약
- 변경 파일 목록
- 회귀 테스트 결과
- 시각 비교 (페이지 12 before/after)
- 향후 작업 후보 (옵션 C — 인라인 Shape를 paragraph_layout 직접 렌더)

`mydocs/orders/yyyymmdd.md` 갱신 (해당 일자 파일 있을 시).

### 4-4. 커밋 및 머지 준비

- 모든 변경사항을 `local/task476` 브랜치에 커밋
- 단계별 보고서 + 최종 보고서 + 계획서 모두 커밋 후 git status 확인
- `local/devel`로 머지는 작업지시자 승인 후 별도 단계

---

## 리스크 및 완화

| 리스크 | 완화책 |
|-------|--------|
| paginator 수정으로 다른 PartialParagraph 케이스 회귀 | 골든 SVG 광범위 비교, 단계별 검증 |
| D 차단이 정상 케이스 박스를 누락시킴 | telemetry 로그로 감지, Stage 2에서 광범위 검증 |
| 23번 문제 박스 위치(별도 버그)도 수정 범위에 포함 여부 | Stage 2/3 진행 중 별도 이슈 분리 판단 |
| 한컴 정답지 부재로 PDF만으로 검증 | 메모리 가이드 — PDF는 보조 ref. 한컴 2010/2020 정답지 환경 차이 가능성 인식 |

## 승인 요청

본 구현 계획에 대한 승인을 요청드립니다. 승인 시 Stage 1부터 진행하겠습니다.
