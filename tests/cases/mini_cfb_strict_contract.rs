//! `mini_cfb` strict MS-CFB container contracts.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::serializer::mini_cfb;

const SECTOR_SIZE: usize = 512;
const MINI_SECTOR_SIZE: usize = 64;
const DIR_ENTRY_SIZE: usize = 128;
const DIR_ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / DIR_ENTRY_SIZE;
const FAT_ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / 4;
const FREESECT: u32 = 0xFFFF_FFFF;
const NOSTREAM: u32 = 0xFFFF_FFFF;

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn directory_entry_offset(bytes: &[u8], index: usize) -> usize {
    let first_dir_sector = read_u32_at(bytes, 48) as usize;
    SECTOR_SIZE
        + (first_dir_sector + index / DIR_ENTRIES_PER_SECTOR) * SECTOR_SIZE
        + (index % DIR_ENTRIES_PER_SECTOR) * DIR_ENTRY_SIZE
}

fn assert_valid_red_black_subtree(
    bytes: &[u8],
    index: u32,
    entry_count: usize,
    parent_is_red: bool,
    visited: &mut [bool],
) -> usize {
    if index == NOSTREAM {
        return 1; // NIL leaves are black.
    }

    let index = index as usize;
    assert!(index < entry_count, "directory index out of range: {index}");
    assert!(
        !visited[index],
        "directory tree contains a cycle at {index}"
    );
    visited[index] = true;

    let offset = directory_entry_offset(bytes, index);
    let color = bytes[offset + 67];
    assert!(color <= 1, "invalid directory color {color} at {index}");
    let is_red = color == 0;
    assert!(
        !(parent_is_red && is_red),
        "adjacent red directory nodes at {index}"
    );

    let left = read_u32_at(bytes, offset + 68);
    let right = read_u32_at(bytes, offset + 72);
    let left_height = assert_valid_red_black_subtree(bytes, left, entry_count, is_red, visited);
    let right_height = assert_valid_red_black_subtree(bytes, right, entry_count, is_red, visited);
    assert_eq!(
        left_height, right_height,
        "directory black-height mismatch at {index}"
    );
    left_height + usize::from(!is_red)
}

#[test]
fn unused_container_slots_use_required_sentinels() {
    let owned = [
        ("/SmallA", vec![0x11; 100]),
        ("/SmallB", vec![0x22; 200]),
        ("/SmallC", vec![0x33; 300]),
        ("/Large", vec![0x44; 5_000]),
    ];
    let streams: Vec<_> = owned
        .iter()
        .map(|(path, data)| (*path, data.as_slice()))
        .collect();
    let entry_count = 1 + streams.len(); // Root Entry + four top-level streams
    let bytes = mini_cfb::build_cfb(&streams).unwrap();

    let total_sectors = (bytes.len() - SECTOR_SIZE) / SECTOR_SIZE;
    let fat_count = read_u32_at(&bytes, 44) as usize;
    assert_eq!(fat_count, 1, "fixture must use exactly one FAT sector");
    let fat_sector = read_u32_at(&bytes, 76) as usize;
    let fat_base = SECTOR_SIZE + fat_sector * SECTOR_SIZE;
    for index in total_sectors..FAT_ENTRIES_PER_SECTOR {
        assert_eq!(
            read_u32_at(&bytes, fat_base + index * 4),
            FREESECT,
            "unused FAT entry {index}"
        );
    }

    let mini_fat_start = read_u32_at(&bytes, 60) as usize;
    let mini_fat_count = read_u32_at(&bytes, 64) as usize;
    assert_eq!(
        mini_fat_count, 1,
        "fixture must use exactly one MiniFAT sector"
    );
    let first_dir_sector = read_u32_at(&bytes, 48) as usize;
    let root_offset = SECTOR_SIZE + first_dir_sector * SECTOR_SIZE;
    let used_mini_entries = read_u32_at(&bytes, root_offset + 120) as usize / MINI_SECTOR_SIZE;
    let mini_fat_base = SECTOR_SIZE + mini_fat_start * SECTOR_SIZE;
    for index in used_mini_entries..FAT_ENTRIES_PER_SECTOR {
        assert_eq!(
            read_u32_at(&bytes, mini_fat_base + index * 4),
            FREESECT,
            "unused MiniFAT entry {index}"
        );
    }

    let dir_capacity = entry_count.div_ceil(DIR_ENTRIES_PER_SECTOR) * DIR_ENTRIES_PER_SECTOR;
    for index in entry_count..dir_capacity {
        let offset = SECTOR_SIZE
            + (first_dir_sector + index / DIR_ENTRIES_PER_SECTOR) * SECTOR_SIZE
            + (index % DIR_ENTRIES_PER_SECTOR) * DIR_ENTRY_SIZE;
        assert_eq!(bytes[offset + 66], 0, "unused directory object type");
        assert_eq!(read_u32_at(&bytes, offset + 68), NOSTREAM);
        assert_eq!(read_u32_at(&bytes, offset + 72), NOSTREAM);
        assert_eq!(read_u32_at(&bytes, offset + 76), NOSTREAM);
    }
}

#[test]
fn directory_tree_obeys_red_black_invariants() {
    for stream_count in 1..=128usize {
        let owned: Vec<_> = (0..stream_count)
            .map(|index| (format!("/Stream{index:04}"), Vec::<u8>::new()))
            .collect();
        let streams: Vec<_> = owned
            .iter()
            .map(|(path, data)| (path.as_str(), data.as_slice()))
            .collect();
        let bytes = mini_cfb::build_cfb(&streams).unwrap();
        let entry_count = 1 + stream_count;

        let root_offset = directory_entry_offset(&bytes, 0);
        let tree_root = read_u32_at(&bytes, root_offset + 76);
        assert_ne!(tree_root, NOSTREAM);
        let tree_root_offset = directory_entry_offset(&bytes, tree_root as usize);
        assert_eq!(
            bytes[tree_root_offset + 67],
            1,
            "directory tree root must be black for {stream_count} streams"
        );

        let mut visited = vec![false; entry_count];
        visited[0] = true; // Root Entry owns the child tree but is not in it.
        assert_valid_red_black_subtree(&bytes, tree_root, entry_count, false, &mut visited);
        assert!(
            visited.iter().all(|seen| *seen),
            "directory tree omitted an entry for {stream_count} streams"
        );
    }
}
