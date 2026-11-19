# seo-rii 통합 체리픽 충돌 해결 기록

기준 `c526dd402bcf55d09cb8d0993537698a25d4740a`, 적용 `64ba35858d4f511bdd993f24eae9337880a7f2b2`.
#9158 → #9159 → #9161 → #9162 → #9163 → #9166 순서의 비병합 고유 커밋 14개를 반영했다.

| 원 PR | 경계 | 해결 및 검증 |
| --- | --- | --- |
| #9163 | `qualify_linux_texture.py`, `qualify_linux_tile_group.py` | 메모리 압박 모듈과 SDK의 하위 프로세스 감독을 함께 import했다. |
| #9163 | `check_linux_desktop_sdk.py` | 단일·두 타일 실행에 압박 횟수를 전달하고, 기존 `supervise_children=True`를 유지했다. 최종 Flutter 전체 tree가 #9166 원 head의 현재 누적 tree와 동일하다. |
| #9166 | `src/renderer/skia/renderer.rs` | #9177의 실제 `TextStyle`과 명시 글리프·원본 TrueType 공급 선택을 그대로 전달하고, 후보가 모두 결손일 때 Native paint와 진단에 같은 설치 글꼴 검색을 연결했다. |
| #9166 | `tests/cases/skia_native_glyph_replay.rs` | #9177의 TTF/TTC 컨테이너 검사와 #9166의 Linux Fontconfig 자식 프로세스 검사를 기존 Native CI 모듈에 함께 보존했다. |

원 PR의 기존 HWP는 가족관계등록 예시 2쪽이다. 한컴 2022 데스크톱에서 Hancom PDF로 인쇄한 동일 입력의 PDF를 사용했다.
통합 Mac에서 Native는 99.66%·100.00%, fresh WASM은 99.67%·100.00%였다. 글자 누락을 재현하도록 특수 공급한 원 작성자의 Linux 조건과 Mac 공개 예시 문서 결과는 서로 다른 환경이다.
