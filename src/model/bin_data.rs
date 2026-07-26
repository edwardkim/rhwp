//! 바이너리 데이터 (BinData, 이미지/OLE 참조)

/// 바이너리 데이터 아이템 (HWPTAG_BIN_DATA)
#[derive(Debug, Clone, Default)]
pub struct BinData {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성 비트 플래그
    pub attr: u16,
    /// 데이터 타입
    pub data_type: BinDataType,
    /// 압축 방식
    pub compression: BinDataCompression,
    /// 접근 상태
    pub status: BinDataStatus,
    /// 연결 파일 절대 경로 (LINK 타입)
    pub abs_path: Option<String>,
    /// 연결 파일 상대 경로 (LINK 타입)
    pub rel_path: Option<String>,
    /// BinData 스토리지 내 ID (EMBEDDING/STORAGE 타입)
    pub storage_id: u16,
    /// 확장자 (EMBEDDING 타입: jpg, bmp, png 등)
    pub extension: Option<String>,
}

/// 바이너리 데이터 타입
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataType {
    #[default]
    /// 외부 파일 참조
    Link,
    /// 파일 포함
    Embedding,
    /// OLE 포함
    Storage,
}

/// 바이너리 데이터 압축 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataCompression {
    #[default]
    /// 스토리지 디폴트
    Default,
    /// 무조건 압축
    Compress,
    /// 무조건 비압축
    NoCompress,
}

/// 바이너리 데이터 접근 상태
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataStatus {
    #[default]
    /// 아직 접근하지 않음
    NotAccessed,
    /// 접근 성공
    Success,
    /// 접근 실패
    Error,
    /// 접근 실패했으나 무시됨
    Ignored,
}

/// BinData 스토리지에서 로드된 실제 데이터
#[derive(Debug, Clone)]
pub struct BinDataContent {
    /// 스토리지 ID
    pub id: u16,
    /// 바이너리 데이터
    pub data: BinDataBytes,
    /// 파일 확장자
    pub extension: String,
}

/// 지연 로딩 대상 BinData 를 원본 컨테이너에서 다시 읽어오는 주체.
///
/// [Task #2263] 압축 해제된 이미지 바이트를 IR 에 상주시키지 않기 위해,
/// 원본 컨테이너(HWPX ZIP / HWP5 CFB)를 보유한 파서 측이 이 트레이트를
/// 구현하고, 실제 바이트가 필요한 시점에만 압축을 푼다.
pub trait BinDataResolver:
    std::fmt::Debug + Send + Sync + std::panic::RefUnwindSafe + std::panic::UnwindSafe
{
    /// `key` 가 가리키는 BinData 의 바이트를 압축 해제하여 반환한다.
    ///
    /// 원본이 손상되었거나 엔트리가 없으면 빈 벡터를 반환한다
    /// (파싱 시점의 placeholder 의미를 그대로 유지한다).
    fn resolve(&self, key: &str) -> Vec<u8>;

    /// 최대 `max_bytes` 바이트까지만 materialize하여 반환한다.
    ///
    /// 기본 구현은 안전하게 실패한다. 컨테이너별 리졸버가 압축 해제 경계에서
    /// 상한을 보장할 수 있을 때만 이 메서드를 구현해야 한다.
    fn resolve_limited(&self, _key: &str, _max_bytes: usize) -> Option<Vec<u8>> {
        None
    }
}

/// BinData 바이트의 보관 방식.
///
/// [Task #2263] 파싱 시점에 모든 내장 이미지를 압축 해제해 상주시키면
/// 원본 파일 크기의 수십 배에 달하는 메모리를 쓰게 된다. `Lazy` 는 원본
/// 컨테이너만 보유하고 실제 요청 시점에 해당 항목만 압축을 푼다.
#[derive(Debug, Clone)]
pub enum BinDataBytes {
    /// 메모리에 이미 올라온 바이트 (직렬화기가 새로 추가한 이미지, HML/HWP3 등).
    ///
    /// `Arc` 인 이유는 이 값이 **여러 벌 복제되는 자리**에 놓이기 때문이다 — undo
    /// 스냅샷은 `Document` 를 통째로 클론하고(`snapshot_store`), 레이어 트리는 편집마다
    /// 다시 빌드된다. `Vec` 이면 4MB 사진 한 장이 스냅샷 98개에서 약 392MB 가 된다
    /// (실측 0.032 → 0.173 ms/스냅샷, Task #3315).
    Loaded(std::sync::Arc<[u8]>),
    /// 원본 컨테이너에서 요청 시 압축 해제
    Lazy {
        /// 원본 컨테이너를 보유한 리졸버 (문서 내 모든 항목이 공유)
        resolver: std::sync::Arc<dyn BinDataResolver>,
        /// 리졸버가 해석하는 항목 키 (HWPX: ZIP 엔트리 경로, HWP5: 스토리지 스트림명)
        key: String,
    },
}

impl BinDataBytes {
    /// 바이트를 얻는다. `Lazy` 인 경우 이 시점에 압축을 푼다.
    ///
    /// `Loaded` 는 같은 할당을 공유해 돌려주므로 호출을 반복해도 복제가 없다 —
    /// 호출부가 `Arc` 를 들기 때문에 종전의 "호출부가 어차피 복사해 보유하니 캐시가
    /// 이중 상주" 라는 전제가 성립하지 않는다 (Task #3315).
    ///
    /// `Lazy` 는 여전히 호출마다 압축을 푼다. 압축 해제 결과를 여기 붙들면 화면에
    /// 없는 이미지까지 상주하므로(#2263 이 `Lazy` 를 도입한 이유) 그 판단은 이 함수가
    /// 아니라 캐시를 갖는 쪽에서 해야 한다. 리졸버가 `Vec` 을 돌려주므로 `Arc` 로 옮기는
    /// 복사 1회가 따라붙는다 — 압축 해제 비용에 묻히지만 `Loaded` 처럼 공짜는 아니다.
    pub fn load(&self) -> std::sync::Arc<[u8]> {
        match self {
            BinDataBytes::Loaded(v) => std::sync::Arc::clone(v),
            BinDataBytes::Lazy { resolver, key } => resolver.resolve(key).into(),
        }
    }

    /// 최대 `max_bytes` 바이트까지만 로드한다.
    ///
    /// `Loaded` 는 복제 전에 길이를 확인하고, `Lazy` 는 리졸버가 제공하는
    /// bounded read/decompression 경로만 사용한다.
    pub fn load_limited(&self, max_bytes: usize) -> Option<std::sync::Arc<[u8]>> {
        match self {
            BinDataBytes::Loaded(v) if v.len() <= max_bytes => Some(std::sync::Arc::clone(v)),
            BinDataBytes::Loaded(_) => None,
            BinDataBytes::Lazy { resolver, key } => resolver
                .resolve_limited(key, max_bytes)
                .filter(|bytes| bytes.len() <= max_bytes)
                .map(Into::into),
        }
    }

    /// 바이트 길이. `Lazy` 인 경우 압축 해제가 발생하므로 렌더 경로의
    /// 반복 호출은 피하고 `load()` 결과를 재사용한다.
    pub fn len(&self) -> usize {
        match self {
            BinDataBytes::Loaded(v) => v.len(),
            // 길이만 알면 되므로 `load()` 를 쓰지 않는다 — 그 경로는 압축 해제 결과를
            // `Arc` 로 한 번 더 복사한다 (Task #3315).
            BinDataBytes::Lazy { resolver, key } => resolver.resolve(key).len(),
        }
    }

    /// 빈 항목인지 판정한다.
    ///
    /// `Lazy` 는 "원본 컨테이너에 엔트리가 있을 것"이라는 기대일 뿐 보장이 아니다.
    /// 매니페스트에는 있으나 실제 엔트리가 없거나(엔트리 누락) 읽기에 실패하는
    /// 경우([#1917] 상한 초과 등) 리졸버가 빈 바이트를 반환하므로, 여기서
    /// 실제로 해석해 봐야 placeholder 의미가 보존된다.
    pub fn is_empty(&self) -> bool {
        match self {
            BinDataBytes::Loaded(v) => v.is_empty(),
            BinDataBytes::Lazy { resolver, key } => resolver.resolve(key).is_empty(),
        }
    }
}

impl Default for BinDataBytes {
    fn default() -> Self {
        BinDataBytes::Loaded(std::sync::Arc::from(Vec::new()))
    }
}

impl From<Vec<u8>> for BinDataBytes {
    fn from(v: Vec<u8>) -> Self {
        BinDataBytes::Loaded(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_bin_data_default() {
        let bd = BinData::default();
        assert_eq!(bd.data_type, BinDataType::Link);
        assert_eq!(bd.compression, BinDataCompression::Default);
        assert_eq!(bd.status, BinDataStatus::NotAccessed);
    }

    #[test]
    fn test_bin_data_embedding() {
        let bd = BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("jpg".to_string()),
            ..Default::default()
        };
        assert_eq!(bd.data_type, BinDataType::Embedding);
        assert_eq!(bd.extension.as_deref(), Some("jpg"));
    }

    #[test]
    fn limited_lazy_load_never_falls_back_to_unbounded_resolution() {
        #[derive(Debug)]
        struct LimitedOnlyResolver {
            requested_limit: AtomicUsize,
        }

        impl BinDataResolver for LimitedOnlyResolver {
            fn resolve(&self, key: &str) -> Vec<u8> {
                panic!("bounded load must not call unbounded resolver: {key}")
            }

            fn resolve_limited(&self, _key: &str, max_bytes: usize) -> Option<Vec<u8>> {
                self.requested_limit.store(max_bytes, Ordering::SeqCst);
                Some(vec![0; max_bytes + 1])
            }
        }

        let resolver = std::sync::Arc::new(LimitedOnlyResolver {
            requested_limit: AtomicUsize::new(0),
        });
        let bytes = BinDataBytes::Lazy {
            resolver: resolver.clone(),
            key: "compressed-font".to_string(),
        };

        assert!(bytes.load_limited(16).is_none());
        assert_eq!(resolver.requested_limit.load(Ordering::SeqCst), 16);
    }
}
