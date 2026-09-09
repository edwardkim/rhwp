# Task #2099 Stage 1 — U+F53A 옛한글 렌더 복구

## 배경

Issue #2099는 `한글문서파일형식_5.0_revision1.3` 문서에서 `글`의 아래아 점이 보이지 않거나 영문 간격이 불안정한 현상이다.

로컬 대응 자료:

- `samples/hwpspec.hwp`: 178쪽, HWP5 배포용 원본.
- `pdf/hwpspec-2024.pdf`: 178쪽, 기존 기준 PDF.
- `samples/pua-test.hwp` / `samples/pua-test.pdf`: PUA 회귀 검증 fixture.

## 확인

- `samples/hwpspec.hwp`에는 U+F53A ``가 실제 문서 단어 `글`의 첫 글자로 사용된다.
- 현재 SVG 출력은 U+F53A를 그대로 내보낸다. Linux/브라우저 공개 폰트 환경에서는 한컴 PUA 글리프가 없어 아래아가 비거나 깨질 수 있다.
- 이전 #615 처리에서 U+F53A를 `pua_oldhangul` 매핑에서 제거했지만, 보존된 `samples/pua-test.pdf` 시각 확인 결과 U+F53A 위치도 ``으로 보인다. 따라서 “빈 공백” 전제는 #2099 기준에서 정정이 필요하다.

## 수정 방침

- U+F53A를 KTUG Hanyang PUA 매핑의 `ᄒᆞᆫ` 자모 시퀀스로 다시 포함한다.
- 매핑 크기와 회귀 테스트를 #2099 기준으로 갱신한다.
- 기존 PUA bullet 영역은 계속 매핑 충돌 금지 대상으로 유지한다.

## 검증 계획

- `cargo test --lib pua_oldhangul`
- `cargo test --test issue_2099_araea_pua`
- `target/debug/rhwp export-svg samples/hwpspec.hwp -p 0`
- SVG에 원본 ``가 남지 않고 `ᄒᆞᆫ`이 포함되는지 확인.
- `samples/pua-test.hwp`에서도 U+F53A가 `ᄒᆞᆫ`으로 렌더 확장되는지 확인.
