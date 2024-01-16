use super::*;

pub fn assert_eq_lines<S1: AsRef<str>, S2: AsRef<str>>(actual: S1, expected: S2) {
    let (real, goal) = (actual.as_ref(), expected.as_ref());
    dbg!(real);
    dbg!(goal);
    for (la, lb) in actual.as_ref().lines().zip(expected.as_ref().lines()) {
        assert_eq!(la, lb, "Expected {la} to be {lb}.")
    }
}

pub fn concat(strings: &[&str]) -> String {
    let mut string = String::new();
    for s in strings {
        if !string.is_empty() {
            string.push('\n');
        }
        string.push_str(s.as_ref());
    }
    string
}

#[test]
fn test_capitalise() {
    assert_eq!(capitalise("tree at hill"), "Tree at Hill");
    assert_eq!(capitalise("sword of killing"), "Sword of Killing");
    assert_eq!(capitalise("the big town"), "The Big Town");
    assert_eq!(capitalise("magic is a resource"), "Magic is a Resource");
}
