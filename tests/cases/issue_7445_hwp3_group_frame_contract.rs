//! Synthetic V3 field contracts through the public drawing reader; no pixel oracle.
use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwp3::drawing::{
    parse_drawing_object_tree, Hwp3DrawingObjectCommonHeader, Hwp3DrawingObjectRotationAttr,
};
use std::{collections::HashMap, io::Cursor};

fn rectangle(h: Hwp3DrawingObjectCommonHeader, depth: usize) -> ShapeObject {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&24u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    for _ in 0..depth {
        bytes.extend_from_slice(&88u32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&[0; 84]);
        bytes.extend_from_slice(&[0; 8]);
    }
    bytes.extend_from_slice(
        &(if h.rotation_attr.is_some() {
            120u32
        } else {
            88u32
        })
        .to_le_bytes(),
    );
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for value in h
        .relative_pos
        .into_iter()
        .chain(h.object_size)
        .chain(h.absolute_pos)
    {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&[0; 16]);
    bytes.extend_from_slice(&[0; 40]);
    bytes.extend_from_slice(
        &(if h.rotation_attr.is_some() {
            1u32 << 17
        } else {
            0u32
        })
        .to_le_bytes(),
    );
    if let Some(rotation) = h.rotation_attr {
        for value in [rotation.center_x, rotation.center_y]
            .into_iter()
            .chain(rotation.parallelogram)
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes.extend_from_slice(&[0; 8]);
    let mut cursor = Cursor::new(bytes.as_slice());
    let mut shape = parse_drawing_object_tree(
        &mut cursor,
        &mut Vec::new(),
        &mut Vec::new(),
        &mut Vec::new(),
        &mut Vec::new(),
        &mut HashMap::new(),
    )
    .expect("synthetic V3 drawing tree");
    assert_eq!(
        cursor.position() as usize,
        bytes.len(),
        "all records consumed once"
    );
    for _ in 0..depth {
        let ShapeObject::Group(mut group) = shape else {
            panic!("source container")
        };
        assert_eq!(group.children.len(), 1);
        shape = group.children.remove(0);
    }
    shape
}
#[test]
fn grouped_child_uses_frame_absolute_position_and_preserves_relative_metadata() {
    // V3 표68: relative는 부모 그룹 기준, absolute는 공통 틀 기준이다.
    let shape = rectangle(
        Hwp3DrawingObjectCommonHeader {
            relative_pos: [5, 6],
            absolute_pos: [25, 36],
            object_size: [100, 50],
            ..Default::default()
        },
        2,
    );

    let attr = shape.shape_attr();
    assert_eq!((attr.offset_x, attr.offset_y), (20, 24));
    assert_eq!((attr.render_tx, attr.render_ty), (100.0, 144.0));
    assert_eq!(
        u16::from_le_bytes(attr.raw_rendering[..2].try_into().unwrap()),
        3
    );
}

#[test]
fn grouped_rectangle_keeps_all_three_source_parallelogram_vertices() {
    // V3 표70의 세 점은 앞 변과 옆 변을 정한다. 각도 하나로 줄이면 전단이 사라진다.
    let mut shape = rectangle(
        Hwp3DrawingObjectCommonHeader {
            relative_pos: [5, 6],
            absolute_pos: [25, 36],
            object_size: [100, 50],
            rotation_attr: Some(Hwp3DrawingObjectRotationAttr {
                parallelogram: [10, 5, 90, 15, 100, 45],
                ..Default::default()
            }),
            ..Default::default()
        },
        1,
    );

    let a = shape.shape_attr();
    for ((x, y), (ex, ey)) in [
        ((0.0, 0.0), (140.0, 164.0)),
        ((400.0, 0.0), (460.0, 204.0)),
        ((400.0, 200.0), (500.0, 324.0)),
    ] {
        assert!((a.render_sx * x + a.render_b * y + a.render_tx - ex).abs() < 1e-9);
        assert!((a.render_c * x + a.render_sy * y + a.render_ty - ey).abs() < 1e-9);
    }
}

#[test]
fn standalone_shape_retains_its_separate_flow_anchor_contract() {
    let mut shape = rectangle(
        Hwp3DrawingObjectCommonHeader {
            relative_pos: [5, 6],
            absolute_pos: [25, 36],
            object_size: [100, 50],
            ..Default::default()
        },
        0,
    );

    assert_eq!(
        (shape.shape_attr().render_tx, shape.shape_attr().render_ty),
        (0.0, 0.0)
    );
    assert!(shape.shape_attr().raw_rendering.is_empty());
}
