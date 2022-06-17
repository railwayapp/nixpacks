macro_rules! static_asset_list {
    ($($name:expr => $value:expr),*) => {
        {
            let mut assets = $crate::nixpacks::app::StaticAssets::new();
            $(
                assets.insert($name.to_string(), $value.to_string());
            )*
            assets
        }
    }
}
