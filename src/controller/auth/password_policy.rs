// We are doing the module trick to isolate the const flags, which, bear with me, are useless.
// We are not restricting any letters or symbols for the password, just enforcing some policies.
// Although I am not sure if that is a good idea.

// TODO: Port that to client-side.
const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 128;
const SPECIAL_CHARACTERS: &str = "!@#$%^&*()-+";
const REQUIRE_SPECIAL_CHARACTERS: bool = true;
const REQUIRE_UPPERCASE: bool = true;
const REQUIRE_DIGIT: bool = true;
const REQUIRE_LOWERCASE: bool = true;

pub fn validate_password_policy(password: &str) -> bool {
    // NOTE: That is not the length, that is the size in bytes as this is how the len function works, it may behave unexpectedly with the grapheme rich symbols,
    // leave that be.

    let size = password.len();

    if !(MIN_LENGTH..MAX_LENGTH).contains(&size) {
        return false;
    }

    let (mut has_uppercase, mut has_lowercase, mut has_digit, mut has_special) = (
        !REQUIRE_UPPERCASE,
        !REQUIRE_LOWERCASE,
        !REQUIRE_DIGIT,
        !REQUIRE_SPECIAL_CHARACTERS,
    );

    for char in password.chars() {
        if !has_uppercase && char.is_uppercase() {
            has_uppercase = true;
        } else if !has_lowercase && char.is_lowercase() {
            has_lowercase = true;
        } else if !has_digit && char.is_ascii_digit() {
            has_digit = true;
        } else if !has_special && SPECIAL_CHARACTERS.contains(char) {
            has_special = true;
        }

        // Early exit if all requirements are met, size is already early satisfied .
        if has_uppercase && has_lowercase && has_digit && has_special {
            return true;
        }
    }

    return has_uppercase && has_lowercase && has_digit && has_special;
}

#[cfg(test)]
mod tests {
    use crate::controller::auth::password_policy;

    use super::validate_password_policy;

    #[test]
    fn test_password_policy() {
        let valid = "Password1!";
        println!(
            "{}",
            valid.repeat(password_policy::MAX_LENGTH.div_ceil(valid.len()))
        );

        assert!(validate_password_policy(valid));
        assert!(!validate_password_policy("weakpass"));
        assert!(!validate_password_policy("Short1!"));
        assert!(!validate_password_policy("NoSpecialChar1"));
        assert!(!validate_password_policy("NOLOWERCASE1!"));
        assert!(!validate_password_policy("nouppercase1!"));
        assert!(!validate_password_policy("NoDigit!"));
        assert!(!validate_password_policy(
            // 128 / 10 = 12.8 => 13 * 10 => 130 > 128
            &valid.repeat(password_policy::MAX_LENGTH.div_ceil(valid.len()))
        ))
    }
}
