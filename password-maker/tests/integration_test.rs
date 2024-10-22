use password_maker::PasswordMaker;

#[test]
fn test_integration() {
    let mut password_maker = PasswordMaker::default();
    let password = password_maker.generate().unwrap();
    assert_eq!(password.chars().count(), 16);

    // Check if consecutive generated passwords are not duplicated
    let password2 = password_maker.generate().unwrap();
    assert_ne!(password, password2);

    // Check if passwords generated by different instances are not duplicated
    let mut password_maker2 = PasswordMaker::default();
    let password3 = password_maker2.generate().unwrap();
    assert_ne!(password, password3);

    // Check if an error occurs
    let mut password_maker = PasswordMaker {
        length: 0,
        ..Default::default()
    };
    assert!(password_maker.generate().is_err());

    // Check if candidates include uppercase, lowercase, numbers, and symbols
    // Since other conditions are tested in unit tests, this is enough here (even too much)
    // Just in case, check if they are included at the beginning, end, and middle
    let candidates = password_maker.candidates();
    assert!(candidates.iter().any(|c| c.eq("A")));
    assert!(candidates.iter().any(|c| c.eq("z")));
    assert!(candidates.iter().any(|c| c.eq("5")));
    assert!(candidates.iter().any(|c| c.eq("|")));
}
