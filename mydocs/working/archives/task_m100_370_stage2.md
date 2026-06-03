# Task #370 Stage 2 완료 보고서

## 통합 검증

```
./target/release/rhwp export-svg samples/hwpx/form-002.hwpx -p 9 -o /tmp/task370/
```

- 출력: `/tmp/task370/form-002_010.svg` (1122줄, 1개 페이지)
- 폴백 체인 적용 확인: `Noto Serif CJK KR` 936회 등장 (모든 한글 세리프 `<text>` 요소).
- 볼드 대상 문구(`(기술적 측면)`, `(시장적 측면)`, `(정책·사회적 측면)`, `연구개발기간`, `정부지원연구개발비`, `주관연구개발기관`, `정부납부…`)에 `font-weight="bold"` 정상 부여 (Stage 0 분석 결과 그대로 유지).

## 시각 확인

`/tmp/task370/form-002_010.svg`를 브라우저에서 열면, 시스템에 설치된 `Noto Serif CJK KR Bold`가 매칭되어 볼드 변형(굵은 글자)이 올바르게 렌더된다. 기존(미설치 패밀리 → generic serif → 자동 한글 폴백)에서는 weight 매칭이 실패해 Regular로 렌더되던 상태가 해소됨.

## 영향 범위 검증

- `cargo test` 전체 1055건 통과.
- 골든 스냅샷의 차이는 폰트 체인 문자열 추가 부분으로만 한정됨을 `diff`로 확인.
