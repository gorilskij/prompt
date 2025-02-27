#[test]
fn test_shorten() {
    use crate::{CWDPath, CWDPathPart};

    let mut path = CWDPath::from("/tmp");
    path.shorten(0);
    assert_eq!(
        path.parts,
        vec![CWDPathPart::RootDir, CWDPathPart::Normal("tmp".to_string())]
    )
}
