macro_rules! despawn_all {
    ($commands:expr, $collection:expr) => {
        for entity in &$collection {
            $commands.entity(entity).despawn();
        }
    };
}

pub(crate) use despawn_all;