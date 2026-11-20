//! Starting configuration for each library profile. These are only defaults chosen in the setup
//! wizard: every value is ordinary configuration that an administrator can edit afterwards.
//! An item category with no policy row for a member category means "not lendable" (e.g. Reference).

pub const PROFILES: [&str; 8] = [
    "school",
    "college",
    "university",
    "public",
    "special",
    "research",
    "institutional",
    "private",
];

pub struct MemberCategoryDef {
    pub name: &'static str,
    pub membership_days: i64,
}

pub struct PolicyDef {
    pub member: &'static str,
    pub item: &'static str,
    pub loan_days: i64,
    pub max_loans: i64,
    pub max_renewals: i64,
    pub renewal_days: i64,
}

pub struct ProfileDefaults {
    pub member_categories: Vec<MemberCategoryDef>,
    /// Lendable item categories (each gets policies) followed by non-lendable ones.
    pub lendable_items: Vec<&'static str>,
    pub reference_items: Vec<&'static str>,
    pub collections: Vec<&'static str>,
    pub locations: Vec<&'static str>,
    pub policies: Vec<PolicyDef>,
}

fn mc(name: &'static str, membership_days: i64) -> MemberCategoryDef {
    MemberCategoryDef {
        name,
        membership_days,
    }
}

fn pol(
    member: &'static str,
    loan_days: i64,
    max_loans: i64,
    max_renewals: i64,
    renewal_days: i64,
) -> PolicyDef {
    PolicyDef {
        member,
        item: "General",
        loan_days,
        max_loans,
        max_renewals,
        renewal_days,
    }
}

/// Returns the starting configuration for a profile code, or `None` for an unknown profile.
pub fn defaults_for(profile: &str) -> Option<ProfileDefaults> {
    let (members, policies): (Vec<MemberCategoryDef>, Vec<PolicyDef>) = match profile {
        "school" => (
            vec![mc("Student", 365), mc("Teacher", 365), mc("Staff", 365)],
            vec![
                pol("Student", 14, 3, 1, 7),
                pol("Teacher", 30, 10, 2, 14),
                pol("Staff", 14, 3, 1, 7),
            ],
        ),
        "college" => (
            vec![mc("Student", 365), mc("Faculty", 365), mc("Staff", 365)],
            vec![
                pol("Student", 14, 3, 1, 7),
                pol("Faculty", 30, 10, 2, 14),
                pol("Staff", 14, 3, 1, 7),
            ],
        ),
        "university" => (
            vec![
                mc("Undergraduate", 365),
                mc("Postgraduate", 365),
                mc("Researcher", 365),
                mc("Faculty", 365),
            ],
            vec![
                pol("Undergraduate", 21, 5, 2, 7),
                pol("Postgraduate", 30, 10, 2, 14),
                pol("Researcher", 60, 15, 3, 14),
                pol("Faculty", 90, 20, 3, 30),
            ],
        ),
        "public" => (
            vec![mc("Adult", 365), mc("Child", 365)],
            vec![pol("Adult", 14, 5, 2, 14), pol("Child", 14, 3, 1, 14)],
        ),
        "special" => (
            vec![mc("Member", 365), mc("Staff", 365)],
            vec![pol("Member", 14, 5, 2, 14), pol("Staff", 30, 10, 2, 14)],
        ),
        "research" => (
            vec![mc("Researcher", 365), mc("Staff", 365)],
            vec![
                pol("Researcher", 30, 10, 2, 14),
                pol("Staff", 30, 10, 2, 14),
            ],
        ),
        "institutional" => (
            vec![mc("Staff", 365), mc("Guest", 90)],
            vec![pol("Staff", 21, 5, 2, 14), pol("Guest", 14, 2, 0, 0)],
        ),
        "private" => (vec![mc("Reader", 3650)], vec![pol("Reader", 30, 10, 2, 14)]),
        _ => return None,
    };
    Some(ProfileDefaults {
        member_categories: members,
        lendable_items: vec!["General"],
        reference_items: vec!["Reference"],
        collections: vec!["General Collection", "Reference Collection"],
        locations: vec!["Main Library"],
        policies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_profile_has_defaults_and_unknown_has_none() {
        for p in PROFILES {
            assert!(defaults_for(p).is_some(), "{p}");
        }
        assert!(defaults_for("zoo").is_none());
    }

    #[test]
    fn policies_reference_real_categories_and_satisfy_database_rules() {
        for p in PROFILES {
            let d = defaults_for(p).unwrap();
            assert!(
                !d.policies.is_empty(),
                "{p} needs at least one lendable policy"
            );
            for pol in &d.policies {
                assert!(
                    d.member_categories.iter().any(|m| m.name == pol.member),
                    "{p}: {}",
                    pol.member
                );
                assert!(d.lendable_items.contains(&pol.item), "{p}: {}", pol.item);
                assert!((1..=365).contains(&pol.loan_days) && pol.max_loans >= 1);
                // Mirrors the CHECK constraint: renewals need a renewal period.
                assert!(
                    pol.max_renewals == 0 || pol.renewal_days > 0,
                    "{p}: {}",
                    pol.member
                );
            }
            let names: Vec<_> = d
                .member_categories
                .iter()
                .map(|m| m.name.to_lowercase())
                .collect();
            let mut dedup = names.clone();
            dedup.sort();
            dedup.dedup();
            assert_eq!(
                names.len(),
                dedup.len(),
                "{p}: duplicate member category names"
            );
        }
    }
}
