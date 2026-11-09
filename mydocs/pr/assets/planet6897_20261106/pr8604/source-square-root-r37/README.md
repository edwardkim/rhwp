# 표시 family 선택과 원 HFT 기호 전진

이 폴더의 `before-*`는 보정 전 실제 측정이다. 보정 후 검증·최종 수용 증거로 사용하지 않는다.

- 입력: `samples/issue8596/policy_committee_agenda.hwp`, SHA-256
  `b3b7c0d557a26d7080957fe5d8472153a6783b9dab029acae4e3b0c29186e4ad`.
- 기존 독립 Print: `pdf/issue8596_policy_committee_agenda-2020.pdf`, SHA-256
  `b15da297882bf644e53e5b895fdb79d119dd03314e24f0fef660001904f4cd4f`.
  파일 이름과 달리 실제 PDF 작성자는 Hwp 2022다. 같은 조판 세대인 compat 2022로 검사했다.
- 2026-11-07, 96 DPI, Print, 색상 경계 보정 전 2px raw 실루엣.
- Native 제품 `ef77d740e1`, fresh WASM 제품 `3f2a7c96ee`. 실제 최적화 WASM SHA-256
  `5783f16d661696184aed7c8f1d41a0f057ffa4b0d3d8f97234e277b411c106b9`.
- 같은 원본·독립 Print의 전체 14쪽: Native 최저 96.52%, WASM 최저 98.44%, 미달 0쪽.
- 실제 11쪽: Native 97.59%, WASM 99.96%.
  [보정 전 Native TSV](before-native.tsv) · [보정 전 fresh WASM TSV](before-wasm.tsv).
  [같은 쪽 Native review](before-native-review-011.png) · [overlay](before-native-overlay-011.png).

원본 PDF의 T6/ToUnicode 04는 U+221A를 HFT285에 연결한다. 이 CharProc의 d1 전진은
1000/1000 em이며 현재 Native 공급 환경의 0.5em 추정과 다르다. 명시적 selectedPrograms가
표시 family를 영어 이름으로 바꾼 경우를 공개 글꼴 결정 API에서 재현했다.
글꼴 프로그램·윤곽은 배포하지 않으며, 회귀는 공급자의 메타데이터만 선언해 시스템 글꼴
설치와 무관하게 원본 글자 크기·원문/셀 소유·저장 바이트 보존을 검사한다.

[원인·선행 Git·소비 경로·보정 범위](../../../../archives/pr_8604_review_impl.md#native-표시-family-변경-시-체크-기호의-원본-폭).
현재 코드 보정과 신규 관계형 검사는 검증 진행 중이다. 전체 Rust 실패를 함께 분석한 뒤
보정 후 Native/fresh WASM 및 집중·전체 회귀 결과를 추가한다.
