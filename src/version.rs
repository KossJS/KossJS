const VERSION:  &[u8] = b"0.1.0-dev.9\0";

pub fn get_version() ->  &'static [u8] {
    return  VERSION;
}
