//! 최소 CFB (Compound File Binary) v3 빌더
//!
//! `cfb` 크레이트의 `CompoundFile::create()`가 `SystemTime::now()`를 호출하여
//! `wasm32-unknown-unknown` 타겟에서 panic이 발생하므로,
//! SystemTime을 사용하지 않는 자체 CFB 빌더를 구현한다.
//!
//! CFB v3 사양:
//! - 섹터 크기: 512바이트
//! - 미니 섹터 크기: 64바이트
//! - 미니 스트림 컷오프: 4096바이트 (표준값)

const SECTOR_SIZE: usize = 512;
const MINI_SECTOR_SIZE: usize = 64;
const MINI_STREAM_CUTOFF: usize = 4096;
const DIR_ENTRY_SIZE: usize = 128;
const ENTRIES_PER_DIR_SECTOR: usize = SECTOR_SIZE / DIR_ENTRY_SIZE; // 4
const FAT_ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / 4; // 128
const HEADER_DIFAT_COUNT: usize = 109;

const ENDOFCHAIN: u32 = 0xFFFFFFFE;
const FREESECT: u32 = 0xFFFFFFFF;
const FATSECT: u32 = 0xFFFFFFFD;
const NOSTREAM: u32 = 0xFFFFFFFF;

/// CFB 시그니처 (Magic Number)
const CFB_SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

struct DirEntry {
    name: String,
    obj_type: u8,
    data: Vec<u8>,
    parent: usize,
    children: Vec<usize>,
    left: u32,
    right: u32,
    child: u32,
    start_sector: u32,
    is_mini: bool,
}

impl DirEntry {
    fn new(name: &str, obj_type: u8, parent: usize) -> Self {
        DirEntry {
            name: name.to_string(),
            obj_type,
            data: Vec::new(),
            parent,
            children: Vec::new(),
            left: NOSTREAM,
            right: NOSTREAM,
            child: NOSTREAM,
            // Storage(1)는 start_sector=0 (MS-CFB 스펙: "SHOULD be set to all zeroes")
            // Root(5), Stream(2)은 ENDOFCHAIN → 나중에 실제 값으로 교체
            start_sector: if obj_type == 1 { 0 } else { ENDOFCHAIN },
            is_mini: false,
        }
    }
}

/// 명명된 스트림 목록으로 CFB v3 바이너리를 생성한다.
///
/// # 인자
/// - `named_streams`: `(경로, 데이터)` 쌍. 경로는 `/FileHeader`, `/BodyText/Section0` 형식.
///
/// # 반환
/// CFB v3 바이너리 바이트.
pub fn build_cfb(named_streams: &[(&str, &[u8])]) -> Result<Vec<u8>, String> {
    // 1. 엔트리 목록 구축
    let mut entries = build_entries(named_streams);

    // 2. 디렉토리 트리 구축
    build_tree(&mut entries, 0);

    // 3. 미니 스트림 구축 (< 4096 바이트 스트림)
    let mut mini_stream = Vec::new();
    let mut mini_fat: Vec<u32> = Vec::new();

    for entry in entries.iter_mut() {
        if entry.obj_type == 2 && !entry.data.is_empty() && entry.data.len() < MINI_STREAM_CUTOFF {
            entry.is_mini = true;
            let start_mini = mini_fat.len();
            entry.start_sector = start_mini as u32;

            let num_mini = (entry.data.len() + MINI_SECTOR_SIZE - 1) / MINI_SECTOR_SIZE;
            for i in 0..num_mini {
                mini_fat.push(if i + 1 < num_mini {
                    (start_mini + i + 1) as u32
                } else {
                    ENDOFCHAIN
                });
            }

            mini_stream.extend_from_slice(&entry.data);
            let pad = (MINI_SECTOR_SIZE - (entry.data.len() % MINI_SECTOR_SIZE)) % MINI_SECTOR_SIZE;
            mini_stream.resize(mini_stream.len() + pad, 0);
        }
    }

    // Root Entry에 미니 스트림 컨테이너 저장
    let mini_stream_size = mini_stream.len();
    if !mini_stream.is_empty() {
        entries[0].data = mini_stream;
    }

    // 4. 정규 섹터 할당
    let dir_sectors = (entries.len() + ENTRIES_PER_DIR_SECTOR - 1) / ENTRIES_PER_DIR_SECTOR;
    let mut next_sector = dir_sectors as u32;

    // 큰 스트림 (>= 4096 바이트) → 정규 섹터
    for entry in entries.iter_mut() {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            entry.start_sector = next_sector;
            let num = (entry.data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
            next_sector += num as u32;
        }
    }

    // Root Entry 미니 스트림 컨테이너 → 정규 섹터
    if mini_stream_size > 0 {
        entries[0].start_sector = next_sector;
        let num = (entries[0].data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
        next_sector += num as u32;
    }

    // 미니 FAT 섹터
    let mini_fat_start;
    let mini_fat_sector_count;
    if !mini_fat.is_empty() {
        mini_fat_start = next_sector;
        mini_fat_sector_count =
            ((mini_fat.len() + FAT_ENTRIES_PER_SECTOR - 1) / FAT_ENTRIES_PER_SECTOR) as u32;
        next_sector += mini_fat_sector_count;
    } else {
        mini_fat_start = ENDOFCHAIN;
        mini_fat_sector_count = 0;
    }

    // FAT 섹터 수 계산 (고정점 반복)
    let non_fat_sectors = next_sector;
    let mut fat_count = 1u32;
    loop {
        let total = non_fat_sectors + fat_count;
        let needed = ((total as usize) + FAT_ENTRIES_PER_SECTOR - 1) / FAT_ENTRIES_PER_SECTOR;
        if needed as u32 <= fat_count {
            break;
        }
        fat_count = needed as u32;
    }

    let fat_start = non_fat_sectors;
    let total_sectors = non_fat_sectors + fat_count;

    // 5. FAT 구축
    let mut fat = vec![FREESECT; total_sectors as usize];

    // 디렉토리 체인
    for i in 0..dir_sectors {
        fat[i] = if i + 1 < dir_sectors {
            (i + 1) as u32
        } else {
            ENDOFCHAIN
        };
    }

    // 큰 스트림 체인
    for entry in entries.iter() {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            let start = entry.start_sector as usize;
            let num = (entry.data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
            for i in 0..num {
                fat[start + i] = if i + 1 < num {
                    (start + i + 1) as u32
                } else {
                    ENDOFCHAIN
                };
            }
        }
    }

    // Root Entry (미니 스트림 컨테이너) 체인
    if entries[0].start_sector != ENDOFCHAIN && !entries[0].data.is_empty() {
        let start = entries[0].start_sector as usize;
        let num = (entries[0].data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
        for i in 0..num {
            fat[start + i] = if i + 1 < num {
                (start + i + 1) as u32
            } else {
                ENDOFCHAIN
            };
        }
    }

    // 미니 FAT 체인
    if mini_fat_start != ENDOFCHAIN {
        let start = mini_fat_start as usize;
        for i in 0..mini_fat_sector_count as usize {
            fat[start + i] = if i + 1 < mini_fat_sector_count as usize {
                (start + i + 1) as u32
            } else {
                ENDOFCHAIN
            };
        }
    }

    // FAT 섹터 마커
    for i in 0..fat_count as usize {
        fat[fat_start as usize + i] = FATSECT;
    }

    // 6. 바이너리 조립
    let file_size = 512 + total_sectors as usize * SECTOR_SIZE;
    let mut output = vec![0u8; file_size];

    // 헤더 작성
    write_header(
        &mut output,
        fat_count,
        fat_start,
        mini_fat_start,
        mini_fat_sector_count,
    );

    // 디렉토리 엔트리 작성
    for (i, entry) in entries.iter().enumerate() {
        let sector_idx = i / ENTRIES_PER_DIR_SECTOR;
        let entry_in_sector = i % ENTRIES_PER_DIR_SECTOR;
        let offset = 512 + sector_idx * SECTOR_SIZE + entry_in_sector * DIR_ENTRY_SIZE;
        write_dir_entry(&mut output, offset, entry);
    }

    // 큰 스트림 데이터 작성
    for entry in &entries {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            let start_offset = 512 + entry.start_sector as usize * SECTOR_SIZE;
            output[start_offset..start_offset + entry.data.len()].copy_from_slice(&entry.data);
        }
    }

    // Root Entry 데이터 (미니 스트림 컨테이너) 작성
    if entries[0].start_sector != ENDOFCHAIN && !entries[0].data.is_empty() {
        let start_offset = 512 + entries[0].start_sector as usize * SECTOR_SIZE;
        output[start_offset..start_offset + entries[0].data.len()]
            .copy_from_slice(&entries[0].data);
    }

    // 미니 FAT 작성
    if mini_fat_start != ENDOFCHAIN {
        for (i, &mf) in mini_fat.iter().enumerate() {
            let sector_idx = i / FAT_ENTRIES_PER_SECTOR;
            let entry_in_sector = i % FAT_ENTRIES_PER_SECTOR;
            let offset =
                512 + (mini_fat_start as usize + sector_idx) * SECTOR_SIZE + entry_in_sector * 4;
            output[offset..offset + 4].copy_from_slice(&mf.to_le_bytes());
        }
    }

    // FAT 작성
    for (i, &fat_entry) in fat.iter().enumerate() {
        let fat_sector_idx = i / FAT_ENTRIES_PER_SECTOR;
        let entry_in_sector = i % FAT_ENTRIES_PER_SECTOR;
        let offset =
            512 + (fat_start as usize + fat_sector_idx) * SECTOR_SIZE + entry_in_sector * 4;
        output[offset..offset + 4].copy_from_slice(&fat_entry.to_le_bytes());
    }

    Ok(output)
}

/// 경로 목록에서 엔트리 목록을 구축한다.
fn build_entries(named_streams: &[(&str, &[u8])]) -> Vec<DirEntry> {
    let mut entries = Vec::new();

    // Root Entry
    entries.push(DirEntry::new("Root Entry", 5, 0));

    for &(path, data) in named_streams {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut parent_idx = 0;

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;

            let existing = entries
                .iter()
                .position(|e| e.parent == parent_idx && e.name == *part);

            if let Some(idx) = existing {
                if is_last {
                    entries[idx].data = data.to_vec();
                }
                parent_idx = idx;
            } else {
                let new_idx = entries.len();
                let obj_type = if is_last { 2 } else { 1 };
                let mut entry = DirEntry::new(part, obj_type, parent_idx);
                if is_last {
                    entry.data = data.to_vec();
                }
                entries[parent_idx].children.push(new_idx);
                entries.push(entry);
                parent_idx = new_idx;
            }
        }
    }

    entries
}

/// 각 스토리지의 자식을 정렬된 균형 이진 트리로 구축한다.
fn build_tree(entries: &mut Vec<DirEntry>, idx: usize) {
    let children = entries[idx].children.clone();
    if children.is_empty() {
        entries[idx].child = NOSTREAM;
        return;
    }

    // CFB 사양에 따라 이름 비교: 길이 우선, 같은 길이면 대소문자 무시
    let mut sorted = children.clone();
    sorted.sort_by(|&a, &b| {
        let na = entries[a].name.to_uppercase();
        let nb = entries[b].name.to_uppercase();
        na.len().cmp(&nb.len()).then(na.cmp(&nb))
    });

    let root = build_balanced_tree(entries, &sorted);
    entries[idx].child = root;

    // 하위 스토리지에 대해 재귀
    for &child_idx in &children {
        if entries[child_idx].obj_type == 1 {
            build_tree(entries, child_idx);
        }
    }
}

/// 정렬된 인덱스 배열로 균형 이진 트리를 구축한다.
fn build_balanced_tree(entries: &mut Vec<DirEntry>, sorted: &[usize]) -> u32 {
    if sorted.is_empty() {
        return NOSTREAM;
    }
    let mid = sorted.len() / 2;
    let root = sorted[mid] as u32;

    let left = build_balanced_tree(entries, &sorted[..mid]);
    let right = if mid + 1 < sorted.len() {
        build_balanced_tree(entries, &sorted[mid + 1..])
    } else {
        NOSTREAM
    };

    entries[root as usize].left = left;
    entries[root as usize].right = right;
    root
}

/// CFB v3 헤더 (512바이트) 작성
fn write_header(
    output: &mut [u8],
    fat_count: u32,
    fat_start: u32,
    mini_fat_start: u32,
    mini_fat_sector_count: u32,
) {
    // 시그니처
    output[0..8].copy_from_slice(&CFB_SIGNATURE);

    // CLSID (16바이트 zero) — 이미 0

    // Minor version: 0x003E
    output[24..26].copy_from_slice(&0x003Eu16.to_le_bytes());
    // Major version: 0x0003 (v3)
    output[26..28].copy_from_slice(&0x0003u16.to_le_bytes());
    // Byte order: 0xFFFE (little-endian)
    output[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
    // Sector shift: 9 (512 bytes)
    output[30..32].copy_from_slice(&9u16.to_le_bytes());
    // Mini sector shift: 6 (64 bytes)
    output[32..34].copy_from_slice(&6u16.to_le_bytes());

    // Reserved (6 bytes) — 이미 0

    // Total directory sectors: 0 (v3에서는 미사용)
    // output[40..44] — 이미 0

    // Total FAT sectors
    output[44..48].copy_from_slice(&fat_count.to_le_bytes());

    // First directory sector SID: 0 (항상 섹터 0부터)
    output[48..52].copy_from_slice(&0u32.to_le_bytes());

    // Transaction signature: 0
    // output[52..56] — 이미 0

    // Mini stream cutoff: 4096 (표준값)
    output[56..60].copy_from_slice(&(MINI_STREAM_CUTOFF as u32).to_le_bytes());

    // First mini FAT sector
    output[60..64].copy_from_slice(&mini_fat_start.to_le_bytes());
    // Total mini FAT sectors
    output[64..68].copy_from_slice(&mini_fat_sector_count.to_le_bytes());

    // First DIFAT sector: ENDOFCHAIN (109개 이내)
    output[68..72].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
    // Total DIFAT sectors: 0
    // output[72..76] — 이미 0

    // DIFAT 배열 (109개 엔트리, 각 4바이트)
    let difat_start = 76;
    for i in 0..HEADER_DIFAT_COUNT {
        let offset = difat_start + i * 4;
        if (i as u32) < fat_count {
            let sid = fat_start + i as u32;
            output[offset..offset + 4].copy_from_slice(&sid.to_le_bytes());
        } else {
            output[offset..offset + 4].copy_from_slice(&FREESECT.to_le_bytes());
        }
    }
}

/// 디렉토리 엔트리 (128바이트) 작성
fn write_dir_entry(output: &mut [u8], offset: usize, entry: &DirEntry) {
    let buf = &mut output[offset..offset + DIR_ENTRY_SIZE];

    // 이름 (UTF-16LE, null 종료, 최대 32 UTF-16 코드 유닛)
    let name_utf16: Vec<u16> = entry.name.encode_utf16().collect();
    let name_len = name_utf16.len().min(31); // 최대 31자 + null
    for i in 0..name_len {
        let pos = i * 2;
        buf[pos..pos + 2].copy_from_slice(&name_utf16[i].to_le_bytes());
    }
    // null 종료
    let null_pos = name_len * 2;
    buf[null_pos..null_pos + 2].copy_from_slice(&0u16.to_le_bytes());

    // 이름 길이 (바이트, null 포함)
    let name_byte_len = ((name_len + 1) * 2) as u16;
    buf[64..66].copy_from_slice(&name_byte_len.to_le_bytes());

    // Object type
    buf[66] = entry.obj_type;

    // Color flag: 1 = black (유효한 red-black 트리)
    buf[67] = 1;

    // Left sibling
    buf[68..72].copy_from_slice(&entry.left.to_le_bytes());
    // Right sibling
    buf[72..76].copy_from_slice(&entry.right.to_le_bytes());
    // Child
    buf[76..80].copy_from_slice(&entry.child.to_le_bytes());

    // CLSID (16 bytes) — 이미 0
    // State bits — 이미 0

    // Creation/Modified time (FILETIME, 8 bytes each)
    // Root Entry(5)와 Storage(1)에 고정 타임스탬프 설정
    // WASM에서 SystemTime::now()를 사용할 수 없으므로 고정값 사용
    // 2024-01-01 00:00:00 UTC ≈ 0x01DA5E8B_80000000
    if entry.obj_type == 5 || entry.obj_type == 1 {
        let filetime: u64 = 0x01DA_5E8B_8000_0000;
        let ft_bytes = filetime.to_le_bytes();
        buf[100..108].copy_from_slice(&ft_bytes); // Creation time
        buf[108..116].copy_from_slice(&ft_bytes); // Modified time
    }

    // Start sector
    buf[116..120].copy_from_slice(&entry.start_sector.to_le_bytes());

    // Stream size (lower 32 bits)
    // type 2 (스트림): 원본 데이터 크기
    // type 5 (루트): 미니 스트림 컨테이너 크기
    let size = if entry.obj_type == 2 || entry.obj_type == 5 {
        entry.data.len() as u32
    } else {
        0
    };
    buf[120..124].copy_from_slice(&size.to_le_bytes());

    // Stream size (upper 32 bits, v3: must be 0)
    // buf[124..128] — 이미 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cfb_signature() {
        let streams = vec![("/TestStream", b"Hello" as &[u8])];
        let bytes = build_cfb(&streams).unwrap();

        assert!(bytes.len() >= 512);
        assert_eq!(&bytes[0..8], &CFB_SIGNATURE);
    }

    #[test]
    fn test_build_cfb_empty() {
        let streams: Vec<(&str, &[u8])> = Vec::new();
        let bytes = build_cfb(&streams).unwrap();

        assert_eq!(&bytes[0..8], &CFB_SIGNATURE);
    }

    #[test]
    fn test_build_cfb_readable_by_cfb_crate() {
        let fh = vec![0xAAu8; 256];
        let di = vec![0xBBu8; 100];
        let streams = vec![("/FileHeader", fh.as_slice()), ("/DocInfo", di.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        // cfb 크레이트로 읽기
        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut fh_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/FileHeader").unwrap(), &mut fh_read)
            .unwrap();
        assert_eq!(fh_read.len(), 256);
        assert!(fh_read.iter().all(|&b| b == 0xAA));

        let mut di_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/DocInfo").unwrap(), &mut di_read)
            .unwrap();
        assert_eq!(di_read.len(), 100);
        assert!(di_read.iter().all(|&b| b == 0xBB));
    }

    #[test]
    fn test_build_cfb_with_storages() {
        let d1 = vec![0x01u8; 256];
        let d2 = vec![0x02u8; 500];
        let d3 = vec![0x03u8; 2000];
        let d4 = vec![0x04u8; 1500];
        let streams = vec![
            ("/FileHeader", d1.as_slice()),
            ("/DocInfo", d2.as_slice()),
            ("/BodyText/Section0", d3.as_slice()),
            ("/BodyText/Section1", d4.as_slice()),
        ];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut s0 = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BodyText/Section0").unwrap(), &mut s0)
            .unwrap();
        assert_eq!(s0.len(), 2000);
        assert!(s0.iter().all(|&b| b == 0x03));

        let mut s1 = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BodyText/Section1").unwrap(), &mut s1)
            .unwrap();
        assert_eq!(s1.len(), 1500);
        assert!(s1.iter().all(|&b| b == 0x04));
    }

    #[test]
    fn test_build_cfb_large_stream() {
        // 10KB 스트림 (다중 섹터, >= 4096이므로 정규 섹터)
        let data = vec![0x55u8; 10240];
        let streams = vec![("/BigStream", data.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut read_data = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BigStream").unwrap(), &mut read_data)
            .unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_build_cfb_mixed_sizes() {
        // 미니 스트림(< 4096)과 정규 스트림(>= 4096) 혼합
        let small = vec![0x11u8; 100];
        let large = vec![0x22u8; 5000];
        let streams = vec![("/Small", small.as_slice()), ("/Large", large.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut s_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/Small").unwrap(), &mut s_read).unwrap();
        assert_eq!(s_read, small);

        let mut l_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/Large").unwrap(), &mut l_read).unwrap();
        assert_eq!(l_read, large);
    }
}
