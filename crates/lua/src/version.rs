use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version([u64; 3]);

impl FromStr for Version {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let invalid = || format!("`{text}` is not a version of one to three numbers");
        let mut parts = [0; 3];
        for (index, part) in text.split('.').enumerate() {
            if index == 3 || part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            parts[index] = part.parse().map_err(|_| invalid())?;
        }
        Ok(Version(parts))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comparison {
    AtLeast,
    Above,
    AtMost,
    Below,
    Exactly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement(Vec<(Comparison, Version)>);

impl FromStr for Requirement {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let mut terms = Vec::new();
        for term in text.split(',') {
            let term = term.trim();
            let (comparison, rest) = [
                (">=", Comparison::AtLeast),
                ("<=", Comparison::AtMost),
                (">", Comparison::Above),
                ("<", Comparison::Below),
                ("=", Comparison::Exactly),
            ]
            .into_iter()
            .find_map(|(operator, comparison)| {
                term.strip_prefix(operator).map(|rest| (comparison, rest))
            })
            .ok_or_else(|| format!("`{term}` in `{text}` does not start with >=, >, <=, < or ="))?;
            terms.push((comparison, rest.trim().parse()?));
        }
        Ok(Requirement(terms))
    }
}

impl Requirement {
    pub fn is_met_by(&self, version: Version) -> bool {
        self.0.iter().all(|&(comparison, bound)| match comparison {
            Comparison::AtLeast => version >= bound,
            Comparison::Above => version > bound,
            Comparison::AtMost => version <= bound,
            Comparison::Below => version < bound,
            Comparison::Exactly => version == bound,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn met(requirement: &str, version: &str) -> bool {
        requirement
            .parse::<Requirement>()
            .unwrap()
            .is_met_by(version.parse().unwrap())
    }

    #[test]
    fn lower_bound_met() {
        assert!(met(">= 0.1", "0.1"));
        assert!(met(">=0.1", "0.1.0"));
        assert!(!met("> 0.1", "0.1"));
    }

    #[test]
    fn range_not_met() {
        assert!(!met(">= 0.2, < 1", "0.1.4"));
        assert!(met(">= 0.2, < 1", "0.9.9"));
        assert!(!met(">= 0.2, < 1", "1"));
        assert!(met("<= 1.2", "1.2.0"));
        assert!(met("= 2", "2.0.0"));
    }

    #[test]
    fn missing_parts_read_as_zero() {
        assert_eq!("1".parse::<Version>(), "1.0.0".parse());
        assert_eq!("1.2".parse::<Version>(), "1.2.0".parse());
        assert!("0.10".parse::<Version>().unwrap() > "0.9".parse().unwrap());
    }

    #[test]
    fn malformed_input_is_refused() {
        for version in ["", "1.", ".1", "1.2.3.4", "a", "1.-2", "1 .2", "v1"] {
            assert!(version.parse::<Version>().is_err(), "{version:?}");
        }
        for requirement in ["", "0.1", ">= ", "=> 1", ">= 1,", "~ 1", ">= 1 < 2"] {
            assert!(
                requirement.parse::<Requirement>().is_err(),
                "{requirement:?}"
            );
        }
    }
}
