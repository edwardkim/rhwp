# 기술 분석 — #1472 ParaShape indent 2배 어긋남 근본 원인

**이슈**: upstream #1472 / 브랜치 `local/task1472` / 2026-06-25
**결론 요약**: 이슈 본문 진단은 오진. 실제 원인은 `is_hwp3_variant`(#1042) indent/spacing 절반화 + `detect_hwp3_variant` 오탐. **옵션 A(탐지 강화)는 파일 내부 신호만으로는 원천적으로 불가능**함을 확인.

---

## 1. 조사로 확정된 사실

### 1.1 HWP5 바이너리는 indent 를 full HWPUNIT 로 저장 (절반 아님)
- `parse_para_shape` raw 바이트 디버그: 3-11월 paraPr(parse idx 12) raw indent = **-3216** (HWPX `default` 와 동일).
- HWP5 바이너리 전체에 `-1608` 은 **0건**. 파서는 indent 를 올바르게 읽는다.

### 1.2 `-1608` 의 출처 = `parser/mod.rs:327` 절반화
```rust
if doc.is_hwp3_variant {  // Task #1001/#1042
    for ps in &mut doc.doc_info.para_shapes {
        ps.indent /= 2; ps.spacing_before /= 2; ps.spacing_after /= 2;
    }
}
```
- `ir-diff`(`parse_document`) 경로엔 이 보정 적용 → -1608. `dump`/raw 디버그는 파싱 직후 → -3216. 두 경로 차이가 최초 혼선의 원인.

### 1.3 오탐 메커니즘
- `detect_hwp3_variant`: HwpSummary 텍스트에 "1990~2003년" 문자열 존재 시 HWP3 시대로 판정(내용 휴리스틱).
- 3-11월 summary: `"1997년 5월 25일 일요일, 3시 40분"` (요일 실제 일치 → 진짜 생성 타임스탬프, 1997 템플릿 유래) + producer `Windows_10`.
- ps_r=0.067, cs_r=0.044 → 비율 가드 통과 → `is_hwp3_variant=true` → 절반화.

### 1.4 정답 = full -3216 (시각 권위)
- jangster77(collaborator): `.pdf`(한글2022)·`.hwpx` 가 정답지.
- 렌더 확인: PDF·한컴2024 hwp·한컴2024 hwpx 모두 연속줄 **깊은 내어쓰기**(full). rhwp-hwp(절반 -1608)만 얕음.

## 2. 옵션 A(탐지 강화) 불가 증명

3-11월(모던, full 정답)과 hwp3-sample16-hwp5(진짜 HWP3 변환본, 절반 보정 대상)를 비교:

| 신호 | 3-11월 | hwp3-sample16-hwp5 |
|------|--------|---------------------|
| summary HWP3-era 날짜 | 1997 (요일 일치=진짜) | 1998 (요일 일치=진짜) |
| producer/OS | WIN32LE Windows_10 | WIN32LE Windows_10 |
| app major | 13 | 13 |
| 파일 버전 | 5.1.1.0 | 5.1.1.0 |
| ps_r/cs_r | 0.067/0.044 | 0.174/0.154 (둘 다 가드 통과) |
| HWP5 raw indent | full (= HWPX twin) | full (-5000, = HWPX twin) |
| indent 짝수성 | 모두 짝수 | 모두 짝수 |

→ **모든 파일 내부 신호가 동일.** 유일한 차이는 *어느 외부 권위가 정답인가*(3-11월=HWPX/PDF, sample16=HWP3 원본)인데, 이는 파일에 인코딩돼 있지 않다. 따라서 파서가 둘을 구분할 방법이 없다.

## 3. 옵션 B(절반화 재검증) 부분 관찰

- sample16-hwp5-2022 본문(불릿 리스트) 페이지에서 `ps.indent` 절반화의 **수평 영향 0** — 불릿(□/◦)은 numbering indent 로 위치가 정해짐. halved/full 렌더의 x좌표 완전 동일(90.0/110.0/117.3), Y(간격)만 차이.
- 즉 sample16 에서 절반화가 실제 작용하는 곳은 주로 **세로 간격(sb/sa)** 및 비-번호 내어쓰기 문단. 한글2022 PDF(`pdf/hwp3-sample16-hwp5-2022.pdf`) 와의 세로 밀도 정밀 비교는 미완(절반화가 한컴 PDF 와 일치하면 sample16 은 절반화 필요 → B 제거 시 회귀).

## 4. 함의

- **A 불가**: 메타/구조 신호로 모던 vs 변환본 구분 불가.
- **B 위험**: 절반화 전면 제거 시 진짜 변환본(sample16 등) 회귀 가능(한컴 PDF 와의 정밀 대조 필요).
- 두 클래스가 파일 신호상 동일하므로 단순 파서 휴리스틱으로는 양립 불가 가능성.

## 5. 권고 (작업지시자 결정 대기)

1. **B-검증 선행**: `pdf/hwp3-sample16-hwp5-2022.pdf` 등 권위 PDF 로 절반화가 진짜 변환본에 *시각적으로 필요한지* 확정. 불필요(한컴이 full 렌더)면 #1042 절반화 축소/제거로 양쪽 해결(클린). 필요면 양립 불가 → upstream 협의.
2. **upstream 코멘트**: #1472 에 교정 진단 공유, 수정 방향 합의.

## 6.5 결정적 추가 발견 — 절반화는 페이지네이션 parity 의 load-bearing 보정

옵션 B-narrow(indent /2 만 제거, spacing /2 유지)를 구현·검증한 결과:

- 단위 테스트 전부 통과, 3-11월/3-09월 indent ir-diff 0건(정답화), sample16 무영향.
- **그러나** 정답(더 깊은) indent 가 본문 줄바꿈을 늘려 미주 영역 레이아웃을 이동시켜:
  - `issue_1082` 3-09'24 미주 페이지-높이 초과 29.3 → **113.5px** (페이지 수는 23=한컴 PDF 일치).
  - **한컴 PDF parity 테스트 3건 회귀**(모두 모의고사군):
    - `issue_1139_page20_starts_after_question29_tail` (3-09'22): "PDF 기준 19쪽에 pi=1020/1021" 단언 깨짐.
    - `issue_1284_..._page21_question23_title_stays_in_left_tail` (3-09'24-미주사이20).
    - `issue_1256_..._page10_question12_keeps_between_notes_gap` (3-09'22, 한컴 PDF 10쪽 7mm 갭).

→ **#1042 indent /2 는 단순 버그가 아니라, rhwp 미주 페이지네이션을 한컴 PDF 에 맞추는
경험적 보정(load-bearing)** 이었다. rhwp 의 줄높이/줄바꿈/미주 엔진이 한컴과 미세하게 달라
(#1184/#1256/#1284 의 정밀 튜닝 영역), 절반 indent 가 본문 높이를 줄여 한컴 PDF 페이지 분할과
정합시켜 왔다. indent 를 IR 상 정답(full)으로 되돌리면 **IR 정확성↔시각(페이지네이션) parity 가
충돌**한다.

### 결론
- #1472 의 "indent 2× 어긋남" 은 독립적으로 고칠 수 없다. IR 정답 indent 적용은 모의고사군
  전체의 한컴-PDF 페이지네이션 parity 를 회귀시킨다.
- 프로젝트 목표가 "한컴 동일 조판"(시각 parity)이므로, 단순 indent /2 제거는 **순효과 음(-)** 일 수 있다.
- 실험 변경 전부 revert(소스/테스트 clean). 방향 재결정 필요.

### 권고
1. **재설계 필요**: indent IR 정확성과 페이지네이션 보정을 분리(=indent 값을 훼손하지 않고
   미주/본문 높이 보정을 엔진 측에서 수행). #1184 미주 2D vpos 아키텍처와 함께 다뤄야 함(대형).
2. 또는 **upstream #1472 에 본 교정 진단 공유** 후 "indent /2 가 페이지네이션 parity 에 load-bearing"
   임을 명시하고, 엔진 개선 전에는 standalone 수정 보류로 합의.
3. 단기 우회(미권장): 모의고사군처럼 미주 parity 가 검증된 파일에서만 영향이 큼 — 일반 HWP5(미주
   적은 문서)는 indent /2 제거가 안전할 가능성. 단 변환본/미주 회귀 가드 없이는 위험.

## 6. 재현 명령 (디버그 코드 없이도 일부 재현)
```
rhwp ir-diff samples/3-11월_..._.hwpx samples/3-11월_..._.hwp | grep indent   # -3216vs-1608
# raw 확인은 parse_para_shape 에 임시 eprintln 필요(본 조사에서 사용, revert 완료)
```
