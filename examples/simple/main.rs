use bizerror::BizError;

#[derive(BizError)]
pub enum ApiError {
    #[bizerror(2001, "FromUtf8Error Error")]
    FromUtf8Error(std::string::FromUtf8Error),
}

// fn from_utf8_error() -> Result<String, ApiError> {
//     let bytes = vec![0, 159];
//     let s = String::from_utf8(bytes).map_biz()?;

//     Ok(s)
// }

fn main() {
    println!("Hello, world!");
}
