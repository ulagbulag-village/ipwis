use avusen::function::Function;

ipwis_modules_codec_api::module_wrap!(
    extern impl Ipwis for "ipwis-modules-ipwis" {
        fn call(&self, func: &Function) -> Result<String>;
    }
);
