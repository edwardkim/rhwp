# Stage 1 완료보고서 — Task #1608

**단계**: RED 테스트 + HWP3 변환본 영향 측정 · **브랜치**: `local/task1608`

## 1. RED 테스트 작성

`tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs`:
네이티브 HWPX(`samples/hwpx/143E433F503322BD33.hwpx`) 파싱 후 모든 섹션
`page_def.pagination_bottom_tolerance == 0` 단언.

수정 전 실행 결과 — **RED 확인**:
```
left: 1600   right: 0
섹션 0: 네이티브 HWPX(head 1.4)에 부당한 HWP3 tolerance(1600 HU)가 부여됨 (#1608)
```

## 2. 판별자 부재 재확인 (version.xml)

| 속성 | 143E… (네이티브) | hwp3-sample-hwpx (변환본) |
|------|------|------|
| head version | **1.4** | **1.4** (동일) |
| major / minor | 5 / 1 | 5 / 1 (동일) |
| application | Hancom Office Hangul | Hancom Office Hangul (동일) |
| appVersion | 11 (한글2022) | 10 |

→ `head version == "1.4"` 판정이 네이티브를 오탐지함이 in-repo 샘플로 직접 재현됨.
appVersion 차이는 저장 앱 버전일 뿐 HWP3 지표 아님 → 단일 메타 판별자 없음(조사 정합).

## 3. HWP3 변환본 영향 측정 (완전제거 vs 가드 결정)

| 변환본 | 제거 전 | 제거 후 |
|--------|--------|--------|
| hwp3-sample-hwpx.hwpx (이슈 인용 "진짜 변환본") | 16쪽 | **16쪽 (변동 없음)** |

tolerance 제거는 가용 높이를 줄이므로 페이지수는 같거나 증가만 가능. 핵심 변환본이
16→16 유지 = **회귀 없음**. SVG 렌더 스폿체크도 정상(16쪽, 크래시 없음, 431 KB).

## 4. 결정

→ **완전 제거 확정**(보수 가드 불필요). Stage 2는 `is_hwp3_origin` + tolerance 블록 제거.
