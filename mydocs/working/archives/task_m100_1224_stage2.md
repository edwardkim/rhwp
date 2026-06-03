# Stage 2 완료 보고서 — Task #1224: 폴백 체인 우선순위 조정

- **이슈**: #1224 (M100)
- **브랜치**: `feature/issue-1224-font-fidelity`
- **단계**: Stage 2 / 4
- **작성일**: 2026-06-01

## 변경 내용

`src/renderer/mod.rs::generic_fallback` 의 sans-serif 체인 2곳(빈 family 분기 + 일반 sans
분기)에 **`'Noto Sans KR Light'` 를 무거운 `'Noto Sans KR'` 직전에 삽입**.

> **후속 보완**: Stage 2 당시에는 Stage 1 측정 결과에 따라 `Light(wght 300)`로 진행했으나,
> Stage 3의 실제 rsvg 페이지밀도 검증에서 `ExtraLight(wght 200)`가 한컴 돋움/PDF 기준에
> 더 근접함을 확인해 최종 구현은 `Noto Sans KR ExtraLight`로 재확정했다. 이 문서는
> Stage 2 시점의 변경 기록을 보존한다.

변경 전:
```
…,'Apple SD Gothic Neo','Noto Sans KR','Pretendard',…
```
변경 후:
```
…,'Apple SD Gothic Neo','Noto Sans KR Light','Noto Sans KR','Pretendard',…
```

`test_generic_fallback` 의 sans 기대 문자열도 동일 갱신.

## 설계 근거

- 시스템 고딕(맑은 고딕/Apple SD Gothic Neo)은 **체인 앞 그대로 유지** → Windows/macOS
  플랫폼 렌더 무영향.
- Light 는 시스템 고딕 부재 환경(Linux/CI)에서만 매칭 — 기존에 무거운 `Noto Sans KR`
  (= Noto Sans CJK KR Regular, 밀도 0.378)로 폴백되던 자리를 **밀도 0.260 의 Light** 로 교체.
- serif·mono·PUA(함초롬) 체인 무변경. bold 런은 weight 별도 처리(영향 없음).

## 적용 범위 한계 (Stage 3 에서 보완)

- 이 변경은 **`Noto Sans KR Light` 폰트가 설치/번들된 환경**에서만 시각 효과 발생.
- 현재 CI/Linux 시스템엔 미설치 → 본 단계만으로는 bare `export-svg` 렌더 무변화
  (fontconfig 가 `Noto Sans KR Light` 미해석 시 다음 후보로 진행).
- **결정적 충실도는 Stage 3(폰트 자산 번들 + 임베딩)** 가 담당.

## 검증

- `cargo test --lib generic_fallback` → 통과.
- `cargo test --lib renderer::` → **593 passed, 0 failed**.
- 잔존 구 체인 문자열 검색 → 없음(svg.rs 는 `super::generic_fallback` 단일 호출).

## 다음 단계

Stage 3: Noto Sans KR Light(한글 서브셋) OFL 자산을 저장소에 번들하고, `find_font_file`/
`known_font_filenames` 에 돋움/고딕 계열 대체 후보로 등록 → `--embed-fonts` 시 Light 글리프
서브셋 임베딩으로 CI/Linux 결정적 충실도 확보.
