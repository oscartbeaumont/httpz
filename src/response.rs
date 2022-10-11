use http::Response;

use crate::Error;

/// TODO
pub trait HttpResponse {
    /// TODO
    fn into_response(self) -> Result<Response<Vec<u8>>, Error>;
}

impl HttpResponse for Response<Vec<u8>> {
    fn into_response(self) -> Result<Response<Vec<u8>>, Error> {
        Ok(self)
    }
}

#[cfg(feature = "cookies")]
impl HttpResponse for (Response<Vec<u8>>, cookie::CookieJar) {
    fn into_response(self) -> Result<Response<Vec<u8>>, Error> {
        use http::header::SET_COOKIE;

        let (mut res, jar) = self;
        let headers = res.headers_mut();
        for cookie in jar.delta() {
            if let Ok(header_value) = cookie.encoded().to_string().parse() {
                headers.append(SET_COOKIE, header_value);
            }
        }

        Ok(res)
    }
}

impl<TResp> HttpResponse for Result<TResp, Error>
where
    TResp: HttpResponse,
{
    fn into_response(self) -> Result<Response<Vec<u8>>, Error> {
        self?.into_response()
    }
}
