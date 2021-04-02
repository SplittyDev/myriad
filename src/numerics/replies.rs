macro_rules! define_numerics {
    ($($code:expr => $name:ident ,)+) => {
        $(
            #[allow(unused)]
            pub const $name: &'static str = $code;
        )+
    };
}

define_numerics! {
    // Welcome
    "001" => RPL_WELCOME,
    "002" => RPL_YOURHOST,
    "003" => RPL_CREATED,
    "004" => RPL_MYINFO,
    "005" => RPL_ISUPPORT,
}