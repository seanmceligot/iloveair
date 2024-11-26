use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyBool(bool);

impl PrettyBool {
    pub fn new(b: bool) -> Self {
        PrettyBool(b)
    }
}

impl From<bool> for PrettyBool {
    fn from(b: bool) -> Self {
        PrettyBool(b)
    }
}
pub static TRUE: &str = "✔️";
pub static FALSE: &str = "❌";

impl fmt::Display for PrettyBool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 {
            write!(f, "{}", TRUE)
        } else {
            write!(f, "{}", FALSE)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_bool_from_bool() {
        let b: bool = true;
        let pb: PrettyBool = PrettyBool::from(b);
        assert_eq!(pb.to_string(), TRUE.to_string());
    }

    #[test]
    fn test_pretty_bool_into_string() {
        let pb: PrettyBool = PrettyBool::new(true);
        assert_eq!(pb.to_string(), TRUE.to_string());
    }

    #[test]
    fn test_pretty_bool_display() {
        let pb: PrettyBool = PrettyBool::new(true);
        assert_eq!(format!("{}", pb), TRUE.to_string());
    }

    #[test]
    fn test_pretty_bool_false() {
        let pb: PrettyBool = PrettyBool::new(false);
        assert_eq!(pb.to_string(), FALSE.to_string());
    }
}
