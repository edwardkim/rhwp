# 양쪽 정렬의 자연 대시와 낱말 공백 예산

원 입력 `issue3307_outline_number.hwpx`의 독립 한컴2020 Print8쪽은 자연 대시를 유지하고
같은 줄의 낱말 간격을 줄인다. 수정 전에는 탄력 대시의 잘린 폭을 남은 양수 여백만큼만 복원하여
줄 오른쪽 끝은 맞아도 `에 따른 소극행정`과 뒤의 리더 원점이 달랐다.

실제 일반 양쪽 정렬 분기 `justified_dash_restoration`에서 자연 폭 복원에 필요한 부족을
기존 음수 여백의 공백20% 한도로 수용한다. 글자 간 압축까지 필요하거나 탭·다른 정렬이면
기존 배분을 유지한다. 각 런의 자연 대시 폭·필드/선두 공백 소유를 그대로 사용한다.
문서명·쪽별 좌표·글꼴·DPI·판정 문턱을 바꾸지 않았다. 마지막 줄의 기존 #5830 경로도 변경하지 않았다.

Native9쪽과 fresh WASM9쪽 모두100.00%이며 누락·중복·90% 미달·정본 보호 표시값 하락은0이다.
Native8쪽99.94275→100.00000%다. 첫 대시 원점595.57333→591.86133 CSSpx로,
독립 Print591.67867 CSSpx와 맞는다. 마지막 낱말 글자 원점과 함께 ±2 CSSpx로 보호한다.
원점 외에는 자연 대시 전진·공백20% 한도·실제 원본 셀/문단의 유일한 소유를 관계로 검사한다.
9쪽 전쪽과 두 경로8쪽의 직접 확대 판독을 마친 뒤2개 검사를 등록했다.
수정 전 실제2 FAIL→수정 후 최소 글꼴 standalone2 PASS다. 같은 최종 소스의 정식 suite는
새2건·기존PII 대시2건·쪽번호3건 모두7 PASS/0 FAIL(1015건은 선택 밖)다.

첫 후보는 대상이 문단 끝이라고 판단해 마지막 줄 분기에 적용했으나9쪽PNG가 모두 같아
제외했다. standalone가 `release-test/librhwp.rlib`의 오래된 별칭을 링크한 진단도 무효로 분리했다.
실제 새 Cargo 산출물 `release-test/deps/librhwp.rlib`를 보존·hash 고정한 뒤 재링크해 위PASS를 확인했다.
이전에 골랐던 #5830 역사적 Print는 수정 전부터 Native64쪽/PDF65쪽으로 달라 정상 대조군으로
수용하지 않았고, 그34·35쪽의 미달을 본 보정의 새 회귀로 쓰거나 새golden으로 등록하지 않았다.
기존 PII 대시·낱말 간격 회귀와 앞 단계 쪽번호 회귀도 위 최종 실행으로 유지됨을 확인했다.

[Native review](after-native-p008-review.png) · [overlay](after-native-p008-overlay.png) ·
[fresh WASM review](after-wasm-p008-review.png) · [overlay](after-wasm-p008-overlay.png).

PDF는 동일 입력·96DPI의 Linux PDFium PNG를 SHA 검증 후 재사용했다. 실제 재사용 provenance를
고정한 wrapper의 정본비교이며 Mac에서 PDFium을 새로 실행했다고 기록하지 않는다.
Mac WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
루트pkg·Studio public·frozen의 JS/WASM hash와 실제 브라우저4 PASS는 receipt에 연결했다.
전체 통합93조건·다른 보호 문서·최종 전체 회귀·실제 소비 검증 완료를 뜻하지 않는다.

Native 영향 대조12쪽과 sample10 WASM9쪽은 앞 단계와 PNG가 바이트 동일하다.
Native lib·선택3개suite Clippy(`-D warnings`), fmt, 현재 base의 manifest48/48·문서 링크도 통과했다.
