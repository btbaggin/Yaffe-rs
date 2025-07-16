#[macro_export]
#[allow(unused_macros)]
macro_rules! yaffe_plugin {
    ($vis:vis struct $name:ident {
        $($value:ident: $ty:ty = $default:expr,)+
    }) => {
        $vis struct $name {
            parent_id: Option<String>,
            $($value: $ty,)+
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> Box<dyn YaffePlugin> { Box::new($name {
            parent_id: None,
            $($value: $default,)+
        }) }
    };
}
