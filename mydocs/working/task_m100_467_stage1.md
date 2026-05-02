# Task #467 단계 1 보고서 — 조사 결과 + 보류 권고

**이슈**: #467 — 확장 바탕쪽 다른 apply_to 조합 (active=Odd + ext=Both 등) 동작 미검증
**브랜치**: `local/task467`
**상태**: **보류 권고** (단계 1 조사 후 작업지시자 결정 요청)

---

## 1. 조사 결과

### 1.1 다른 apply_to 조합 샘플 발견 — `samples/exam_science.hwp`

```
바탕쪽: 5개
  [0] Both, !ext, !overlap, ext_flags=0x0000          → 비확장 Both
  [1] Both, !ext, !overlap, ext_flags=0x0004          → 비확장 Both (다른 인코딩)
  [2] Odd,  !ext,  overlap, ext_flags=0x0005          → 비확장 Odd + overlap=true
  [3] Even,  ext, !overlap, ext_flags=0x0006          → 확장 Even (replace)
  [4] Both,  ext,  overlap, ext_flags=0x0007          → 확장 Both + overlap=true
```

### 1.2 페이지 4 (마지막 짝수) master 적용 추적

`document_core/queries/rendering.rs:990~1041` 로직:
- is_odd=false, is_last=true
- mp_even 검색: 비확장 Even 없음 → None
- mp_both 검색: [0] (첫 번째 비확장 Both)
- selected = mp_both → active = mp[0]
- ext 처리:
  - replace_exts = [3] (Even ext, replace)
  - overlap_exts = [4] (Both ext, overlap)
  - active = mp[3] (replace 적용)
  - active_apply = Even
  - mp[4].apply_to = Both ≠ Even → remaining_overlap_exts = [4]
  - **extra_master_pages = [mp[4]]**

→ 페이지 4 = active(mp[3] Even ext) + extra(mp[4] Both ext overlap). **다른 apply_to 조합 케이스 활성**.

### 1.3 SVG vs PDF 시각 비교 — 페이지 4

| 영역 | PDF | SVG (현재) |
|---|---|---|
| 좌측 상단 | **"4(화1)"** 큰 페이지 번호 + 부제목 | 누락 |
| 우측 상단 | "과학탐구 영역" 헤더 | 표시 |
| 하단 | "32-32" 페이지 번호 + 가로선 | 표시 |

PDF 좌측 상단 "4(화1)" 컨텐츠가 SVG에 누락. master[3] (Even ext) 또는 master[4] (Both ext overlap) 의 컨트롤이 그려지지 않은 것으로 추정.

### 1.4 master 컨트롤 분석

```
mp[3] Even ext:
  ctrl[0]: 도형 (페이지 배경/테두리 등)
  ctrl[1]: 표 1x1, wrap=BehindText, cells=["31|32"]   ← 좌측 페이지 번호 셀

mp[4] Both ext overlap:
  ctrl[0]: 도형
  ctrl[1]: 표 1x1, wrap=BehindText, cells=["32|32"]   ← 좌측 페이지 번호 셀
  ctrl[2]: 표 1x1, wrap=TopAndBottom, "확인 사항|◦ 답안지의 해당란에..."  ← 하단 안내
```

→ "31|32" / "32|32" 는 좌측 페이지 번호 (현재페이지|전체페이지) 형식. PDF 의 "4(화1)" 와 다름. PDF 의 "4" 는 큰 폰트 페이지 번호이고 "(화1)" 은 과목 명 부제목.

→ master 간 매핑이 더 복잡할 가능성. 또는 dump 가 보여주지 않는 추가 컨트롤.

## 2. 본 task 의 한계

### 2.1 직접 검증 어려움

- PDF/SVG 직접 좌표 비교는 페이지 크기 차이로 신뢰도 낮음 (#491 보류 사유와 동일)
- 한컴 2010/2020 환경에서 직접 렌더링 비교 불가 (개발 환경 제약)
- master[3]/master[4] 내부 컨트롤이 dump 에 일부만 노출됨 (ctrl 의 자세한 텍스트/이미지 미확인)

### 2.2 결함 본질 식별 어려움

PDF 좌측 상단 "4(화1)" 누락이:
- (a) #467 다른 apply_to 조합 처리 결함인지
- (b) master 컨트롤 자체 렌더링 결함인지 (BehindText 표 처리 등)
- (c) 또 다른 layout 결함인지

확정 어려움.

### 2.3 #467 의 광범위성

이슈 본문 "다른 apply_to 조합 샘플 발굴 + 한컴 PDF 출력 동작 비교" 명시. **본질적으로 조사 task** 이며 단순 코드 수정으로 해결 어려움.

## 3. 보류 권고

### 후보

| 후보 | 내용 | 위험 |
|---|---|---|
| (A) | "31\|32" 형식의 좌측 페이지 번호 표 렌더링 결함으로 좁혀 별도 task 분리 | 새 task 생성, 본 task 와 분리 |
| (B) | extra_master_pages 렌더링 차단 (apply_to 다른 ext 무시) | PDF 와 일치 가능성 모름, 회귀 위험 |
| (C) | 더 깊은 조사 (master 컨트롤 모두 추적 + 한컴 PDF 환경 수동 검증) | 시간 큼, 신뢰성 낮음 |
| **(D)** | **보류** — 향후 layout 리팩터링 또는 한컴 환경 검증 가능 시점에 종합 해결 | 결함 잔존, 가장 안전 |

### (D) 보류 근거

1. 본 task 는 **조사 task** 이며 코드 변경이 명확하지 않음
2. PDF/SVG 직접 비교 신뢰도 낮음 (#491 보류 사유 동일)
3. 식별된 SVG 결함 ("4(화1)" 누락) 이 본 task 범위(다른 apply_to 조합 처리) 와 직접 연관 모호
4. 한컴 2010/2020 환경 검증 어려움 — 메모리 `feedback_pdf_not_authoritative.md` 적용 (PDF 200dpi 는 보조 ref)
5. 메모리 `feedback_essential_fix_regression_risk.md`: master 처리 변경은 회귀 위험 큼

## 4. 후속 처리 제안

- **본 issue (#467) 보류 결정** — close 안 함, open 유지
- 페이지 4 좌측 상단 "4(화1)" 누락은 별도 결함으로 식별 시 **새 issue 생성** 권고 (master 컨트롤 BehindText 표 렌더링 또는 다른 결함)
- 향후 한컴 환경 직접 검증 가능 시점에 종합 처리

## 5. 산출물

- 코드 변경: 없음 (조사만 진행)
- 본 보고서

## 6. 작업지시자 결정 요청

자동승인 모드이지만 #496 학습 (회귀 위험 큰 layout 본질 정정) 패턴 — 본 task 도 명확한 코드 수정 방향이 없어 보류 결정에 명시적 승인 필요.
