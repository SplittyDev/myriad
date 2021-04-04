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
    // LUSERS
    "251" => RPL_LUSERCLIENT,
    "252" => RPL_LUSEROP,
    // Channels
    "332" => RPL_TOPIC,
    "353" => RPL_NAMREPLY,
    // MOTD
    "375" => RPL_MOTDSTART,
    "372" => RPL_MOTD,
    "376" => RPL_ENDOFMOTD,
}
