use super::*;

#[test]
fn test_difference() {
    assert_eq!(
        &format!(
            "{:?}",
            IterOps::difference(vec!("a", "b", "4"), vec!("c", "b", "4"))
        ),
        "[\"a\", \"c\"]"
    );
}


#[test]
fn test_intersect() {
    assert_eq!(
        &format!(
            "{:?}",
            IterOps::intersect(vec!("a", "b", "4"), vec!("c", "b", "4"))
        ),
        "[\"b\", \"4\"]"
    );
}
