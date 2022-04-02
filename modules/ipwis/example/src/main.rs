use avusen::{
    account::{Decoder, Encoder},
    function::Function,
};
use ipwis_modules_ipwis_api::{Ipwis, IpwisImpl};

pub fn main() {
    let func = Function {
        caller: Encoder {
            public_key: Default::default(),
            signature: Default::default(),
        },
        program: avusen::source::Source::Ipfs {
            author: Decoder {
                private_key: Default::default(),
                encoder: Encoder {
                    public_key: Default::default(),
                    signature: Default::default(),
                },
            },
            host: Some("hello world!".to_string()),
            path: Default::default(),
        },
        inputs: Default::default(),
        outputs: Default::default(),
    };

    let result = IpwisImpl.call(&func).unwrap();
    assert_eq!(&result, "hello world!");
}
