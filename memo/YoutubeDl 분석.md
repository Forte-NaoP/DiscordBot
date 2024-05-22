실행 순서
```rust
YoutubeDl::into()
    YoutubeDl::create_async // (from Compose trait)
        HttpRequest::create_async // (from Compose trait)
            HttpRequest::create_stream
                reqwest::Client::get -> reqwest::Response
                reqwest::Response::bytes_stream -> Box<tokio_io::StreamReader> 
                Box<tokio_io::StreamReader> -> songbird::HttpStream
            songbird::HttpStream::stream -> AsyncAdapterStream
        songbird::HttpStream -> songbird::AudioStream
    return songbird::AudioStream
```
create_stream에서 resume를 이용해서 계속 파일을 받아오는 것 같음.
연속된 네트워크 스트림에서 특정 부분만 잘라내는건 어려워보이므로 파일로 저장 후 songbird::File을 이용해서 songbird::Input을 생성


--- 
24-05-22

`util::youtube_dl.rs` 주의점

- `start`와 `duration`의 범위 처리가 되어있지않음


