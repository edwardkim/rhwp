# #8183 3단계 — 실제 소비 검증과 사용자 가이드

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183) ·
[수행 계획](../plans/task_m100_8183.md) · [2단계 결과](task_m100_8183_stage2.md)

2026-10-23 작업지시자가 「3단계 진행 승인」으로 실제 CLI·fresh WASM/Studio 검증,
시각·성능 확인과 빌드·사용자 안내 작성을 승인했다. 기준 source는 `b799b1f47`이다.
원격 게시·push·PR 생성은 이번 승인에 포함하지 않는다.

상태: **진행 중. 아래 항목은 완료 검증 결과가 아니다.**

## 검증 범위

- default/JBIG2/JPEG 2000/양쪽의 실제 Native·WASM 산출물과 공통 지원 정보 일치.
- CLI 가져오기·저장·재열기, Rust 공개 API, Studio Worker의 오류·취소·다음 요청.
- 정상 입력과 mask·palette·alpha·공유 사전, 전체 페이지 TSV와 직접 판독.
- 동일 조건의 시간·메모리 관측·산출물 크기, 기존 대조군과 필수 lint·회귀.
- 실제 실행 결과에 맞춘 선택 빌드·설치·진단 가이드.

임시 증적은 `output/pr-review/issue8183/stage3/`에 둔다. 공용 target과 8개 Cargo 잡을
사용하고 Cargo 빌드는 순차 실행한다. 시작 시 가용 디스크는 17 GiB였다.
