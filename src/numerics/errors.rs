macro_rules! define_numerics {
    ($($code:expr => $name:ident ,)+) => {
        $(
            #[allow(unused)]
            pub const $name: &'static str = $code;
        )+
    };
}

define_numerics! {
    // NICK
    "431" => ERR_NONICKNAMEGIVEN,
    "433" => ERR_NICKNAMEINUSE,
    // USER
    "461" => ERR_NEEDMOREPARAMS,
    "462" => ERR_ALREADYREGISTRED,
}
