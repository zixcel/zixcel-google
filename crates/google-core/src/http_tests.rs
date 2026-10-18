use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
fn exchange(status: u16, headers: &str, body: Vec<u8>) -> Result<Vec<u8>> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let headers = headers.to_owned();
    let thread = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = [0; 4096];
        let n = stream.read(&mut request).unwrap();
        assert!(
            String::from_utf8_lossy(&request[..n]).contains("authorization: Bearer SYNTHETIC_ONLY")
        );
        let _ = write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n",
            body.len()
        );
        let _ = stream.write_all(&body);
    });
    let mut transport = GoogleHttp::new("SYNTHETIC_ONLY").unwrap();
    // This private unit-test client has no public endpoint override or release feature.
    transport.client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let result = transport.get_url(&format!("http://{address}/"));
    thread.join().unwrap();
    result
}
#[test]
fn http_statuses_and_redirect_are_not_retried_or_converted_to_key_state() {
    for (status, error) in [
        (401, Error::Unauthorized),
        (403, Error::Forbidden),
        (404, Error::Unavailable),
        (429, Error::RateLimited),
        (500, Error::HttpStatus(500)),
        (302, Error::HttpStatus(302)),
    ] {
        assert_eq!(
            exchange(
                status,
                "Location: http://127.0.0.1:1/steal\r\n",
                b"SYNTHETIC_PRIVATE_RESPONSE".to_vec()
            )
            .unwrap_err(),
            error
        );
    }
    assert_eq!(exchange(200, "", b"{}".to_vec()).unwrap(), b"{}");
}
#[test]
fn http_content_length_budget_is_enforced() {
    assert_eq!(
        exchange(200, "", vec![b' '; MAXIMUM + 1]).unwrap_err(),
        Error::ResponseTooLarge
    );
}
#[test]
fn invalid_tokens_and_arbitrary_paths_are_rejected_before_network() {
    for token in ["", "bad\r\nheader", "bearer space"] {
        assert!(matches!(
            GoogleHttp::new(token),
            Err(Error::CredentialInvalid)
        ));
    }
    for path in [
        "https://other.invalid/",
        "/v1/projects/../../other",
        "/v1/projects/sample-project/serviceAccounts/other@evil.invalid",
        "/v1/projects/sample-project/serviceAccounts/reader-account@sample-project.iam.gserviceaccount.com/keys?other=true",
    ] {
        assert_eq!(ReadRequest::iam(path).unwrap_err(), Error::InvalidInput);
    }
}

#[test]
fn quota_identity_is_explicit_and_request_budget_precedes_network() {
    let mut http = GoogleHttp::new("SYNTHETIC_ONLY").unwrap();
    assert!(http.quota_project.is_none());
    for project in [
        "../sample-project",
        "sample-project\r\nheader",
        "https://other.invalid",
    ] {
        assert_eq!(
            http.set_quota_project(project).unwrap_err(),
            Error::InvalidInput
        );
    }
    http.set_quota_project("sample-project").unwrap();
    assert_eq!(
        http.quota_project.as_ref().unwrap().to_str().unwrap(),
        "sample-project"
    );
    http.remaining = 0;
    let request=ReadRequest::iam("/v1/projects/sample-project/serviceAccounts/reader-account@sample-project.iam.gserviceaccount.com").unwrap();
    assert_eq!(http.read(&request).unwrap_err(), Error::Deadline);
}
