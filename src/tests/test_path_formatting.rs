#[test]
fn test_shorten() {
    use crate::{CWDPath, CWDPathPart};

    let mut path = CWDPath::from_str("/tmp");
    path.shorten(0);
    assert_eq!(
        path.parts,
        vec![CWDPathPart::Root, CWDPathPart::Normal("tmp".to_string())]
    )
}
