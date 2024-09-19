#[cfg(test)]
mod tests {
    use bevy::utils::hashbrown::HashMap;

    use crate::creature::{DamageResult, Ipseity, Soul};

    #[test]
    fn ipseity_damage() {
        let mut ipseity =
            Ipseity::new(&[(Soul::Saintly, 2), (Soul::Ordered, 4), (Soul::Artistic, 3)]);
        assert_eq!(ipseity.harvest_random_souls(3), DamageResult::Survived);
        assert_eq!(ipseity.get_active_soul_count(), 6);
    }

    #[test]
    fn ipseity_repression() {
        let mut ipseity = Ipseity {
            active: HashMap::new(),
            forefront: [Some(Soul::Saintly), None, Some(Soul::Saintly), None],
            repressed: HashMap::new(),
        };
        assert_eq!(ipseity.harvest_random_souls(1), DamageResult::Survived);
        assert_eq!(ipseity.harvest_random_souls(2), DamageResult::Drained);
        assert_eq!(*ipseity.repressed.get(&Soul::Saintly).unwrap(), 2);
    }
}
