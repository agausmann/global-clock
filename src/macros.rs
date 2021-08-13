#[macro_export]
macro_rules! asset_path {
    ($path:literal) => {
        concat!(env!("ASSETS_ROOT"), "/", $path)
    };
}

#[cfg(feature = "bundled-assets")]
mod bundling {
    #[macro_export]
    macro_rules! asset_bytes {
        ($path:literal) => {
            std::borrow::Cow::Borrowed(include_bytes!($crate::asset_path!($path)))
        };
    }

    #[macro_export]
    macro_rules! asset_str {
        ($path:literal) => {
            std::borrow::Cow::Borrowed(include_str!($crate::asset_path!($path)))
        };
    }
}

#[cfg(not(feature = "bundled-assets"))]
mod not_bundling {
    #[macro_export]
    macro_rules! asset_bytes {
        ($path:literal) => {
            std::borrow::Cow::Owned(
                std::fs::read($crate::asset_path!($path)).expect("failed to read asset from disk"),
            )
        };
    }

    #[macro_export]
    macro_rules! asset_str {
        ($path:literal) => {
            std::borrow::Cow::Owned(
                std::fs::read_to_string($crate::asset_path!($path))
                    .expect("failed to read asset from disk"),
            )
        };
    }
}
