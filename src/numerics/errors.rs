macro_rules! define_numerics {
    ($($code:expr => $name:ident ,)+) => {
        $(
            #[allow(unused)]
            pub const $name: &'static str = $code;
        )+
    };
}

define_numerics! {
    "431" => ERR_NONICKNAMEGIVEN,
    "433" => ERR_NICKNAMEINUSE,
}